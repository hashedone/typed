use std::io::stdin;

use color_eyre::Result;
use eyre::Context;
use nom_locate::LocatedSpan;
use tracing::info;
use tracing_subscriber::EnvFilter;

fn main() -> Result<()> {
    color_eyre::install()?;
    dotenv::dotenv().wrap_err("Dotenv setup")?;
    let filter = EnvFilter::from_default_env();
    tracing_subscriber::fmt().with_env_filter(filter).init();
    info!("Logging initialized");

    let source: Vec<_> = stdin().lines().map_while(Result::ok).collect();
    let source: String = source.join("\n");
    let _source = LocatedSpan::new(source);

    Ok(())
}
