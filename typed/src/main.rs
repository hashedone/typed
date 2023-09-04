use std::io::stdin;

use nom_locate::LocatedSpan;

pub mod ast;
mod mir;

fn main() {
    let source: Vec<_> = stdin().lines().filter_map(Result::ok).collect();
    let source: String = source.join("\n");
    let source = LocatedSpan::new(source);

    let ast = match ast::Ast::parse_verbose(&source) {
        Ok(ast) => ast,
        Err(err) => {
            println!("{err:?}");
            return;
        }
    };
    //    println!("{:#?}", ast);
    ast.print_tree(&mut std::io::stdout()).unwrap();
}
