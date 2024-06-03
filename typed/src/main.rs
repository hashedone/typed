use std::ops::Range;
use std::path::PathBuf;

use ariadne::{ColorGenerator, Fmt, Label, Report, ReportKind, Source};
use clap::Parser;
use color_eyre::Result;
use eyre::Context;
use itertools::Itertools;
use tracing::info;
use tracing_subscriber::EnvFilter;

use typed_parser as parser;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Input file name
    #[arg(short, long)]
    input: PathBuf,
}

fn create_report<'a>(
    source_name: &'a str,
    error: parser::Error,
) -> Report<'a, (&'a str, Range<usize>)> {
    let mut report = Report::build(ReportKind::Error, source_name, error.span().start)
        .with_message("Parse error");

    let mut colors = ColorGenerator::new();
    let expected = error.expected();

    if expected.len() > 0 {
        let color = colors.next();
        let label = Label::new((source_name, error.span().start..error.span().end));

        let label = match error.found() {
            Some(found) => label.with_message(format!("Unexpected {}", found.fg(color))),
            None => label.with_message(format!("Unexpected {}", "EOF".fg(color))),
        };

        let label = label.with_color(color);

        report = report.with_label(label);

        let expected = expected.map(|e| e.fg(color).to_string()).join(", ");

        report = report.with_note(format!("Expected one of: {}", expected));
    }

    report.finish()
}

fn main() -> Result<()> {
    color_eyre::install()?;
    dotenv::dotenv().wrap_err("Dotenv setup")?;

    let args = Args::parse();

    let filter = EnvFilter::from_default_env();
    tracing_subscriber::fmt().with_env_filter(filter).init();
    info!("Logging initialized");

    let source = std::fs::read_to_string(&args.input).wrap_err("Reading input file")?;

    let module = parser::parse(&source);

    println!("Module: {:#?}", module.output());

    let input_name = args.input.as_os_str().to_str().unwrap_or("");
    for err in module.into_errors() {
        create_report(input_name, err).eprint((input_name, Source::from(&source)))?;
    }

    Ok(())
}
