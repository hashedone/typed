use std::io::stdin;

pub mod ast;
mod mir;

fn main() {
    let source: Vec<_> = stdin().lines().filter_map(Result::ok).collect();
    let source: String = source.join("\n");

    let ast = ast::Ast::parse(&source).unwrap();
    println!("{:#?}", ast);
}
