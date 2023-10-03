use std::path::{Path, PathBuf};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[arg(short, long)]
    site: PathBuf,
    #[arg(short, long)]
    templates: PathBuf,
    #[arg(short, long)]
    output: PathBuf
}

fn main() {
    let args = Args::parse();
}