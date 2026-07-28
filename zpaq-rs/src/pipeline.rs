use crossbeam_channel::{bounded, unbounded};
use indicatif::ProgressBar;
use rayon::ThreadPoolBuilder;
use std::collections::BTreeMap;
use std::ffi::CString;
use std::io::{Read, Write};
use std::os::raw::{c_char, c_int, c_void};

use crate::ffi::*;

#[derive(Debug, Clone)]
pub struct CompressConfig {
    pub level: String,
    pub block_size: usize,
    pub threads: usize,
}

#[derive(Debug, Clone)]
pub struct DecompressConfig {
    pub threads: usize,
}

pub struct RawBlock {
    pub id: usize,
    pub data: Vec<u8>,
}

pub struct CompressedBlock {
    pub id: usize,
    pub data: Vec<u8>,
}

struct SliceReadState<'a> {
    data: &'a [u8],
    pos: usize,
}

struct VecWriteState {
    data: Vec<u8>,
}

unsafe extern "C" fn slice_read_cb(
    user_data: *mut c_void,
    buf: *mut c_char,
    size: c_int,
) -> c_int {
    let state = &mut *(user_data as *mut SliceReadState);
    let remaining = state.data.len() - state.pos;
    let to_read = (size as usize).min(remaining);
    if to_read > 0 {
        std::ptr::copy_nonoverlapping(
            state.data[state.pos..].as_ptr(),
            buf as *mut u8,
            to_read,
        );
        state.pos += to_read;
    }
    to_read as c_int
}

unsafe extern "C" fn vec_write_cb(
    user_data: *mut c_void,
    buf: *const c_char,
    size: c_int,
) -> c_int {
    let state = &mut *(user_data as *mut VecWriteState);
    let bytes = std::slice::from_raw_parts(buf as *const u8, size as usize);
    state.data.extend_from_slice(bytes);
    size
}

pub fn compress_block_buffer(data: &[u8], level: &str) -> Result<Vec<u8>, String> {
    let mut read_state = SliceReadState { data, pos: 0 };
    let mut write_state = VecWriteState {
        data: Vec::with_capacity(data.len() / 2 + 1024),
    };
    let c_method = CString::new(level).map_err(|e| e.to_string())?;

    let res = unsafe {
        zpaq_compress_stream(
            Some(slice_read_cb),
            &mut read_state as *mut _ as *mut c_void,
            Some(vec_write_cb),
            &mut write_state as *mut _ as *mut c_void,
            c_method.as_ptr(),
        )
    };

    if res == 0 {
        Ok(write_state.data)
    } else {
        Err(format!("zpaq_compress_stream failed with code {}", res))
    }
}

pub fn decompress_block_buffer(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut read_state = SliceReadState { data, pos: 0 };
    let mut write_state = VecWriteState {
        data: Vec::with_capacity(data.len() * 4 + 1024),
    };

    let res = unsafe {
        zpaq_decompress_stream(
            Some(slice_read_cb),
            &mut read_state as *mut _ as *mut c_void,
            Some(vec_write_cb),
            &mut write_state as *mut _ as *mut c_void,
        )
    };

    if res == 0 {
        Ok(write_state.data)
    } else {
        Err(format!("zpaq_decompress_stream failed with code {}", res))
    }
}

pub fn find_block_offsets(data: &[u8]) -> Vec<usize> {
    let mut offsets = Vec::new();
    let mut h1: u32 = 0x3D49B113;
    let mut h2: u32 = 0x29EB7F93;
    let mut h3: u32 = 0x2614BE13;
    let mut h4: u32 = 0x3828EB13;

    for (i, &b) in data.iter().enumerate() {
        let c = b as u32;
        h1 = h1.wrapping_mul(12).wrapping_add(c);
        h2 = h2.wrapping_mul(20).wrapping_add(c);
        h3 = h3.wrapping_mul(28).wrapping_add(c);
        h4 = h4.wrapping_mul(44).wrapping_add(c);

        if h1 == 0xB16B88F1 && h2 == 0xFF5376F1 && h3 == 0x72AC5BF1 && h4 == 0x2F909AF1 {
            if i >= 15 {
                offsets.push(i - 15);
            }
        }
    }
    offsets
}

/// Lock-free Multi-Producer Single-Consumer (MPSC) streaming block compression pipeline.
pub fn compress_pipeline<R: Read + Send + 'static, W: Write + Send>(
    mut reader: R,
    mut writer: W,
    config: CompressConfig,
    progress: Option<ProgressBar>,
) -> Result<(u64, u64), String> {
    let num_threads = if config.threads == 0 {
        std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4)
    } else {
        config.threads
    };

    let pool = ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .map_err(|e| e.to_string())?;

    let (tx_in, rx_in) = bounded::<RawBlock>(num_threads * 2);
    let (tx_out, rx_out) = unbounded::<CompressedBlock>();

    let level = config.level.clone();
    let block_size = config.block_size;

    let reader_handle = std::thread::spawn(move || -> Result<u64, String> {
        let mut block_id = 0;
        let mut total_read = 0u64;
        loop {
            let mut buf = vec![0u8; block_size];
            let mut nread = 0;
            while nread < block_size {
                let n = reader.read(&mut buf[nread..]).map_err(|e| e.to_string())?;
                if n == 0 {
                    break;
                }
                nread += n;
            }
            if nread == 0 {
                break;
            }
            buf.truncate(nread);
            total_read += nread as u64;

            if tx_in.send(RawBlock { id: block_id, data: buf }).is_err() {
                break;
            }
            block_id += 1;
        }
        Ok(total_read)
    });

    pool.scope(|scope| {
        for _ in 0..num_threads {
            let rx_in = rx_in.clone();
            let tx_out = tx_out.clone();
            let level = level.clone();
            scope.spawn(move |_| {
                while let Ok(block) = rx_in.recv() {
                    let compressed = compress_block_buffer(&block.data, &level)
                        .expect("Block compression failed");
                    if tx_out.send(CompressedBlock { id: block.id, data: compressed }).is_err() {
                        break;
                    }
                }
            });
        }
    });

    drop(tx_out);

    let mut pending: BTreeMap<usize, Vec<u8>> = BTreeMap::new();
    let mut next_id = 0;
    let mut total_written = 0u64;

    while let Ok(cblock) = rx_out.recv() {
        pending.insert(cblock.id, cblock.data);
        while let Some(data) = pending.remove(&next_id) {
            writer.write_all(&data).map_err(|e| e.to_string())?;
            total_written += data.len() as u64;
            if let Some(ref pb) = progress {
                pb.inc(data.len() as u64);
            }
            next_id += 1;
        }
    }

    writer.flush().map_err(|e| e.to_string())?;

    let total_read = reader_handle.join().unwrap()?;
    if let Some(ref pb) = progress {
        pb.finish_with_message("Compression complete");
    }

    Ok((total_read, total_written))
}

/// Lock-free Multi-Producer Single-Consumer (MPSC) streaming block decompression pipeline.
pub fn decompress_pipeline<R: Read + Send + 'static, W: Write + Send>(
    mut reader: R,
    mut writer: W,
    config: DecompressConfig,
    progress: Option<ProgressBar>,
) -> Result<(u64, u64), String> {
    let num_threads = if config.threads == 0 {
        std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4)
    } else {
        config.threads
    };

    let pool = ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .map_err(|e| e.to_string())?;

    let (tx_in, rx_in) = bounded::<CompressedBlock>(num_threads * 2);
    let (tx_out, rx_out) = unbounded::<RawBlock>();

    let reader_handle = std::thread::spawn(move || -> Result<u64, String> {
        let mut scan_buf = Vec::new();
        let mut chunk_buf = vec![0u8; 64 * 1024];
        let mut block_id = 0;
        let mut total_read = 0u64;

        loop {
            let n = reader.read(&mut chunk_buf).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            total_read += n as u64;
            scan_buf.extend_from_slice(&chunk_buf[..n]);

            loop {
                let offsets = find_block_offsets(&scan_buf);
                if offsets.len() >= 2 {
                    let end_off = offsets[1];
                    let block_bytes = scan_buf[..end_off].to_vec();
                    scan_buf.drain(..end_off);
                    if tx_in.send(CompressedBlock { id: block_id, data: block_bytes }).is_err() {
                        return Ok(total_read);
                    }
                    block_id += 1;
                } else {
                    break;
                }
            }
        }

        if !scan_buf.is_empty() {
            let _ = tx_in.send(CompressedBlock { id: block_id, data: scan_buf });
        }

        Ok(total_read)
    });

    pool.scope(|scope| {
        for _ in 0..num_threads {
            let rx_in = rx_in.clone();
            let tx_out = tx_out.clone();
            scope.spawn(move |_| {
                while let Ok(block) = rx_in.recv() {
                    let decompressed = decompress_block_buffer(&block.data)
                        .expect("Block decompression failed");
                    if tx_out.send(RawBlock { id: block.id, data: decompressed }).is_err() {
                        break;
                    }
                }
            });
        }
    });

    drop(tx_out);

    let mut pending: BTreeMap<usize, Vec<u8>> = BTreeMap::new();
    let mut next_id = 0;
    let mut total_written = 0u64;

    while let Ok(rblock) = rx_out.recv() {
        pending.insert(rblock.id, rblock.data);
        while let Some(data) = pending.remove(&next_id) {
            writer.write_all(&data).map_err(|e| e.to_string())?;
            total_written += data.len() as u64;
            if let Some(ref pb) = progress {
                pb.inc(data.len() as u64);
            }
            next_id += 1;
        }
    }

    writer.flush().map_err(|e| e.to_string())?;

    let total_read = reader_handle.join().unwrap()?;
    if let Some(ref pb) = progress {
        pb.finish_with_message("Decompression complete");
    }

    Ok((total_read, total_written))
}
