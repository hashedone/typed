use std::path::PathBuf;

use clap::Parser;
use color_eyre::Result;
use eyre::Context;
use nom_locate::LocatedSpan;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Input file name
    #[arg(short, long)]
    input: PathBuf,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    dotenv::dotenv().wrap_err("Dotenv setup")?;

    let args = Args::parse();

    let filter = EnvFilter::from_default_env();
    tracing_subscriber::fmt().with_env_filter(filter).init();
    info!("Logging initialized");

    let source = std::fs::read_to_string(args.input).wrap_err("Reading input file")?;
    let _source = LocatedSpan::new(source);

    Ok(())
}
