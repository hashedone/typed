use std::io::stdin;

mod parser;
mod mir;

fn main() {
    let source: Vec<_> = stdin().lines().filter_map(Result::ok).collect();
    let source: String = source.join("\n");

    let ast = parser::Ast::parse(&source).unwrap();
    println!("{:#?}", ast);
}
