use std::ops::Range;
use std::path::PathBuf;

use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
use clap::Parser;
use color_eyre::Result;
use eyre::Context;
use tracing::info;
use tracing_subscriber::EnvFilter;

use parser::ast::Ast;
use parser::error::Error;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Input file name
    #[arg(short, long)]
    input: PathBuf,
}

fn create_report_unexpected<'a>(
    source_name: &'a str,
    offset: usize,
    context: &'a str,
    recovery_point: usize,
) -> Report<'a, (&'a str, Range<usize>)> {
    let config = ariadne::Config::default()
        .with_label_attach(ariadne::LabelAttach::Start)
        .with_tab_width(2);

    let highlight = Color::Fixed(81);

    Report::build(ReportKind::Error, source_name, offset)
        .with_code(1)
        .with_config(config)
        .with_message("Unexpected token")
        .with_label(
            Label::new((source_name, offset..recovery_point))
                .with_message(format!("While parsing {}", context.fg(highlight))),
        )
        .finish()
}

fn create_report(
    source_name: &str,
    error: parser::error::RecoveredError,
) -> Report<(&str, Range<usize>)> {
    match error.error {
        Error::Unexpected { offset, context } => {
            create_report_unexpected(source_name, offset, context, error.recovery_point)
        }
    }
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
