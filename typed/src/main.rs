use std::path::PathBuf;

use clap::Parser;
use color_eyre::Result;
use eyre::Context;
use nom::error::convert_error;
use tracing::info;
use tracing_subscriber::EnvFilter;

use parser::RawAST;

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

    let source = std::fs::read_to_string(&args.input).wrap_err("Reading input file")?;

    let module = match RawAST::parse(&source) {
        Err(err) => {
            let err: Vec<_> = err
                .errors
                .into_iter()
                .map(|(input, error)| (*input.fragment(), error))
                .collect();

            print!(
                "While compiling {}:\n{}",
                args.input.to_string_lossy(),
                convert_error(source.as_str(), nom::error::VerboseError { errors: err })
            );

            return Ok(());
        }
        Ok(module) => module,
    };

    println!("Compiled module:");
    module.format(&mut std::io::stdout().lock()).unwrap();

    Ok(())
}
