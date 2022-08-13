use clap::{ArgGroup, Parser};
use std::path::Path;
use std::time::Instant;

mod ciso;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
#[clap(group(ArgGroup::new("action").required(true).args(&["compress", "decompress"])))]
struct CliArgs {
    #[clap(short, long)]
    compress: bool,

    #[clap(short, long)]
    decompress: bool,

    #[clap(short, long, default_value_t = 5, value_parser = clap::value_parser!(u8).range(1..9))]
    level: u8,

    input_file: String,
    output_file: String,
}

fn main() {
    let cli_args = CliArgs::parse();

    let start = Instant::now();

    ciso::decompress_ciso(
        &Path::new(&cli_args.input_file),
        &Path::new(&cli_args.output_file),
    )
    .expect("Error decompressing CISO");

    let t = start.elapsed().as_micros() as f64 / 1000.0;
    println!("Duration: {:.3}ms", t);
}
