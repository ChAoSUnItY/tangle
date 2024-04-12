use std::{fs::File, io::Write};

use defs::Alias;
use globals::get_source;

use crate::lexer::{Lexer, TokenType};

mod globals;
mod lexer;
mod defs;

fn main() {
    let source = get_source();

    File::create("shecc/out.c")
        .unwrap()
        .write_all(source.as_bytes())
        .unwrap();

    let mut lexer = Lexer::new("^ ASL 10 20 MUL ADD");
    lexer.add_alias("ASL", "MUL");
    lexer.add_alias("MUL", "&& ||");

    loop {
        let token = lexer.next_token();

        if token == TokenType::TEof {
            break;
        }

        println!("{:?}", token);
    }
}
