use std::io::stdin;

use nom_locate::LocatedSpan;
use tree_printer::print_parsed_tree;

mod mir;
pub mod parser;
mod tree_printer;

fn main() {
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

    print_parsed_tree(&mut std::io::stdout(), ast.root()).unwrap();
}
