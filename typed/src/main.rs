use std::{ops::Range, path::PathBuf};

use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
use clap::Parser;
use color_eyre::Result;
use eyre::Context;
use tracing::info;
use tracing_subscriber::EnvFilter;

use parser::ast::Ast;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Input file name
    #[arg(short, long)]
    input: PathBuf,
}

fn create_report<'a>(
    source_name: &'a str,
    error: parser::error::Error<'a>,
) -> Report<'a, (&'a str, Range<usize>)> {
    let config = ariadne::Config::default()
        .with_label_attach(ariadne::LabelAttach::Start)
        .with_tab_width(2);
    let highlight = Color::Fixed(81);

    Report::build(ReportKind::Error, source_name, error.offset)
        .with_code(1)
        .with_config(config)
        .with_message(error.to_string())
        .with_label(
            Label::new((source_name, error.context_span))
                .with_message(format!("While parsing {}", error.context.fg(highlight))),
        )
        .finish()
}

fn main() -> Result<()> {
    color_eyre::install()?;
    dotenv::dotenv().wrap_err("Dotenv setup")?;

    let args = Args::parse();

    let filter = EnvFilter::from_default_env();
    tracing_subscriber::fmt().with_env_filter(filter).init();
    info!("Logging initialized");

    let source = std::fs::read_to_string(&args.input).wrap_err("Reading input file")?;

    let ast = Ast::parse(&source);

    ast.format(&mut std::io::stdout().lock())?;
    println!();

    let input_name = args.input.as_os_str().to_str().unwrap_or("");
    for err in ast.errors {
        create_report(input_name, err).eprint((input_name, Source::from(&source)))?;
    }

    Ok(())
}
