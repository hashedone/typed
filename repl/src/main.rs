use rustyline::{Editor, error::ReadlineError};
use parser;

fn main() {
    let mut rl = Editor::<()>::new();

    rl.load_history("history").ok();

    loop {
        match rl.readline("> ") {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                let ast = parser::parse(line.as_str());
                println!("= {:?}", ast);
            }
            Err(ReadlineError::Interrupted) |
            Err(ReadlineError::Eof) => { break }
            Err(err) => {
                println!("Error: {:?}", err);
            }
        }
    }

    rl.save_history("history").ok();
}
