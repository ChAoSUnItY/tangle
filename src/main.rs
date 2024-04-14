use std::{fs::File, io::Write};

use globals::get_source;
use parser::Parser;

mod defs;
mod globals;
mod lexer;
mod parser;

fn main() {
    let source = get_source();

    File::create("shecc/out.c")
        .unwrap()
        .write_all(source.as_bytes())
        .unwrap();

    let mut parser = Parser::new("#define SUB - \"STR\" +\n#define ADD SUB - 1 *\nwhile ADD while + (");
    parser.read_global_statements();
}
