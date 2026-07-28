pub mod ffi;
pub mod pipeline;

#[cfg(test)]
mod tests {
    use super::ffi::*;
    use super::pipeline::*;
    use std::ffi::CString;
    use std::os::raw::{c_char, c_int, c_void};
    use std::io::Cursor;

    struct ReadState<'a> {
        data: &'a [u8],
        pos: usize,
    }

    struct WriteState {
        data: Vec<u8>,
    }

    unsafe extern "C" fn read_callback(
        user_data: *mut c_void,
        buf: *mut c_char,
        size: c_int,
    ) -> c_int {
        let state = &mut *(user_data as *mut ReadState);
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

    unsafe extern "C" fn write_callback(
        user_data: *mut c_void,
        buf: *const c_char,
        size: c_int,
    ) -> c_int {
        let state = &mut *(user_data as *mut WriteState);
        let bytes = std::slice::from_raw_parts(buf as *const u8, size as usize);
        state.data.extend_from_slice(bytes);
        size
    }

    #[test]
    fn test_compress_decompress_stream() {
        let input_data = b"Hello ZPAQ Compression and Decompression stream test!";

        let mut read_state = ReadState {
            data: input_data,
            pos: 0,
        };
        let mut compressed = WriteState { data: Vec::new() };

        let method = CString::new("1").unwrap();
        let res = unsafe {
            zpaq_compress_stream(
                Some(read_callback),
                &mut read_state as *mut ReadState as *mut c_void,
                Some(write_callback),
                &mut compressed as *mut WriteState as *mut c_void,
                method.as_ptr(),
            )
        };
        assert_eq!(res, 0);
        assert!(!compressed.data.is_empty());

        let mut comp_read_state = ReadState {
            data: &compressed.data,
            pos: 0,
        };
        let mut decompressed = WriteState { data: Vec::new() };

        let res = unsafe {
            zpaq_decompress_stream(
                Some(read_callback),
                &mut comp_read_state as *mut ReadState as *mut c_void,
                Some(write_callback),
                &mut decompressed as *mut WriteState as *mut c_void,
            )
        };
        assert_eq!(res, 0);
        assert_eq!(decompressed.data.as_slice(), input_data);
    }

    #[test]
    fn test_pipeline_compress_decompress() {
        let original_data: Vec<u8> = (0..100000).map(|i| (i % 256) as u8).collect();

        let reader = Cursor::new(original_data.clone());
        let mut compressed_buf = Vec::new();

        let config = CompressConfig {
            level: "1".to_string(),
            block_size: 16384,
            threads: 4,
        };

        let (read_bytes, comp_bytes) = compress_pipeline(reader, &mut compressed_buf, config, None).unwrap();
        assert_eq!(read_bytes, original_data.len() as u64);
        assert!(comp_bytes > 0);

        let decomp_reader = Cursor::new(compressed_buf);
        let mut decompressed_buf = Vec::new();

        let decomp_config = DecompressConfig { threads: 4 };

        let (_, decomp_bytes) = decompress_pipeline(decomp_reader, &mut decompressed_buf, decomp_config, None).unwrap();
        assert_eq!(decomp_bytes, original_data.len() as u64);
        assert_eq!(decompressed_buf, original_data);
    }
}
