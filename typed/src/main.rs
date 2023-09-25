use std::io::stdin;

use nom_locate::LocatedSpan;
use tracing::info;
use tracing_subscriber::filter::EnvFilter;

mod mir;
pub mod parser;
pub mod reduced;
mod tree_printer;

fn main() {
    dotenv::dotenv().unwrap();
    let filter = EnvFilter::from_default_env();
    tracing_subscriber::fmt().with_env_filter(filter).init();
    info!("Logging initialized");
    let source: Vec<_> = stdin().lines().filter_map(Result::ok).collect();
    let source: String = source.join("\n");
    let source = LocatedSpan::new(source);

    let ast = match parser::Ast::parse_verbose(&source) {
        Ok(ast) => ast,
        Err(err) => {
            println!("{err:?}");
            return;
        }
    };

    // print_parsed_tree(&mut std::io::stdout(), ast.root()).unwrap();
    let reduced = reduced::Ast::new(ast.root).unwrap();
    println!("{reduced}");
}
