use clap::{Args, Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::PathBuf;
use zpaq_rs::pipeline::{compress_pipeline, decompress_pipeline, CompressConfig, DecompressConfig};

#[derive(Parser)]
#[command(
    name = "zpaq-rs",
    author = "High-Performance Lock-Free ZPAQ Team",
    version = "0.1.0",
    about = "Lock-free Multi-Producer Single-Consumer (MPSC) streaming ZPAQ compression utility"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compress input file/stream to output ZPAQ stream
    Compress(CompressArgs),
    /// Decompress input ZPAQ file/stream to output stream
    Decompress(DecompressArgs),
}

#[derive(Args)]
struct CompressArgs {
    /// Compression level (e.g. "1", "2", "3", "14,128,0")
    #[arg(short = 'l', long = "level", default_value = "1")]
    level: String,

    /// Block size (e.g. "16MB", "64M", "1048576")
    #[arg(short = 'b', long = "block-size", default_value = "16MB")]
    block_size: String,

    /// Number of worker threads (0 = auto-detect CPU cores)
    #[arg(short = 't', long = "threads", default_value_t = 0)]
    threads: usize,

    /// Input file path (defaults to stdin if omitted or "-")
    #[arg(short = 'i', long = "input")]
    input: Option<PathBuf>,

    /// Output file path (defaults to stdout if omitted or "-")
    #[arg(short = 'o', long = "output")]
    output: Option<PathBuf>,
}

#[derive(Args)]
struct DecompressArgs {
    /// Number of worker threads (0 = auto-detect CPU cores)
    #[arg(short = 't', long = "threads", default_value_t = 0)]
    threads: usize,

    /// Input file path (defaults to stdin if omitted or "-")
    #[arg(short = 'i', long = "input")]
    input: Option<PathBuf>,

    /// Output file path (defaults to stdout if omitted or "-")
    #[arg(short = 'o', long = "output")]
    output: Option<PathBuf>,
}

fn parse_block_size(s: &str) -> Result<usize, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("Block size cannot be empty".to_string());
    }

    let (num_str, mult) = if s.ends_with("GB") || s.ends_with("gb") {
        (&s[..s.len() - 2], 1024 * 1024 * 1024)
    } else if s.ends_with('G') || s.ends_with('g') {
        (&s[..s.len() - 1], 1024 * 1024 * 1024)
    } else if s.ends_with("MB") || s.ends_with("mb") {
        (&s[..s.len() - 2], 1024 * 1024)
    } else if s.ends_with('M') || s.ends_with('m') {
        (&s[..s.len() - 1], 1024 * 1024)
    } else if s.ends_with("KB") || s.ends_with("kb") {
        (&s[..s.len() - 2], 1024)
    } else if s.ends_with('K') || s.ends_with('k') {
        (&s[..s.len() - 1], 1024)
    } else if s.ends_with('B') || s.ends_with('b') {
        (&s[..s.len() - 1], 1)
    } else {
        (s, 1)
    };

    let num: usize = num_str
        .trim()
        .parse()
        .map_err(|e| format!("Invalid block size string '{}': {}", num_str, e))?;

    Ok(num * mult)
}

fn is_stdin_path(p: Option<&PathBuf>) -> bool {
    match p {
        None => true,
        Some(path) => path.to_str() == Some("-"),
    }
}

fn is_stdout_path(p: Option<&PathBuf>) -> bool {
    match p {
        None => true,
        Some(path) => path.to_str() == Some("-"),
    }
}

fn create_progress_bar(total_bytes: Option<u64>, action_msg: &str) -> ProgressBar {
    match total_bytes {
        Some(len) => {
            let pb = ProgressBar::new(len);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta}) {msg}")
                    .unwrap()
                    .progress_chars("#>-"),
            );
            pb.set_message(action_msg.to_string());
            pb
        }
        None => {
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} [{elapsed_precise}] {bytes} processed {msg}")
                    .unwrap(),
            );
            pb.set_message(action_msg.to_string());
            pb
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Compress(args) => {
            let block_size = parse_block_size(&args.block_size)?;

            let (reader, total_len): (Box<dyn Read + Send + 'static>, Option<u64>) =
                if is_stdin_path(args.input.as_ref()) {
                    (Box::new(BufReader::new(io::stdin())), None)
                } else {
                    let path = args.input.as_ref().unwrap();
                    let file = File::open(path)?;
                    let len = file.metadata().ok().map(|m| m.len());
                    (Box::new(BufReader::new(file)), len)
                };

            let writer: Box<dyn Write + Send> = if is_stdout_path(args.output.as_ref()) {
                Box::new(BufWriter::new(io::stdout()))
            } else {
                let path = args.output.as_ref().unwrap();
                let file = File::create(path)?;
                Box::new(BufWriter::new(file))
            };

            let pb = create_progress_bar(total_len, "Compressing");

            let config = CompressConfig {
                level: args.level,
                block_size,
                threads: args.threads,
            };

            let (read_bytes, comp_bytes) =
                compress_pipeline(reader, writer, config, Some(pb))?;

            eprintln!(
                "Compression finished: {} -> {} bytes ({:.2}% ratio)",
                read_bytes,
                comp_bytes,
                if read_bytes > 0 {
                    (comp_bytes as f64 / read_bytes as f64) * 100.0
                } else {
                    0.0
                }
            );
        }
        Commands::Decompress(args) => {
            let (reader, total_len): (Box<dyn Read + Send + 'static>, Option<u64>) =
                if is_stdin_path(args.input.as_ref()) {
                    (Box::new(BufReader::new(io::stdin())), None)
                } else {
                    let path = args.input.as_ref().unwrap();
                    let file = File::open(path)?;
                    let len = file.metadata().ok().map(|m| m.len());
                    (Box::new(BufReader::new(file)), len)
                };

            let writer: Box<dyn Write + Send> = if is_stdout_path(args.output.as_ref()) {
                Box::new(BufWriter::new(io::stdout()))
            } else {
                let path = args.output.as_ref().unwrap();
                let file = File::create(path)?;
                Box::new(BufWriter::new(file))
            };

            let pb = create_progress_bar(total_len, "Decompressing");

            let config = DecompressConfig {
                threads: args.threads,
            };

            let (read_bytes, decomp_bytes) =
                decompress_pipeline(reader, writer, config, Some(pb))?;

            eprintln!(
                "Decompression finished: {} -> {} bytes",
                read_bytes, decomp_bytes
            );
        }
    }

    Ok(())
}
