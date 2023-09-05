use std::io::Write;

use crate::parser::{
    binding::Binding, expression::Expression, fn_appl::FnAppl, fn_decl::FnDecl, root::Root,
};

fn print_expr(
    w: &mut impl Write,
    expr: &Expression<'_>,
    indent: usize,
) -> Result<(), std::io::Error> {
    match expr {
        Expression::Literal(lit) => write!(w, "{:indent$}LIT: {lit}\n", ""),
        Expression::Variable(v) => write!(w, "{:indent$}VAR: {v}\n", ""),
        Expression::FnDecl(decl) => print_fn_decl(w, decl, indent),
        Expression::FnAppl(appl) => print_fn_appl(w, appl, indent),
    }
}

fn print_fn_decl(
    w: &mut impl Write,
    decl: &FnDecl<'_>,
    indent: usize,
) -> Result<(), std::io::Error> {
    let args = decl.args.join(" ");
    write!(w, "{:indent$}fn ({args}):\n", "")?;

    for binding in &decl.bindings {
        print_binding(w, binding, indent + 1)?;
    }

    print_expr(w, &decl.expr, indent + 1)
}

fn print_fn_appl(
    w: &mut impl Write,
    appl: &FnAppl<'_>,
    indent: usize,
) -> Result<(), std::io::Error> {
    print_expr(w, &appl.func, indent)?;
    write!(w, "{:indent$} <-\n", "", indent = indent)?;
    for arg in &appl.args {
        print_expr(w, arg, indent + 1)?;
    }

    Ok(())
}

fn print_binding(
    w: &mut impl Write,
    bind: &Binding<'_>,
    indent: usize,
) -> Result<(), std::io::Error> {
    write!(w, "{:indent$}let {} =\n", "", bind.name)?;
    print_expr(w, &bind.expr, indent + 1)
}

pub fn print_parsed_tree(w: &mut impl Write, root: &Root<'_>) -> Result<(), std::io::Error> {
    for binding in &root.bindings {
        print_binding(w, binding, 0)?;
    }

    print_expr(w, &root.expr, 0)
}
