use clap::{ArgGroup, Parser};

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

    println!("Args: {:#?}", cli_args);
}
