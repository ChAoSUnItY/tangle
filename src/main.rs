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

    // {
    //     let mut parser =
    //         Parser::new("#define SUB - \"STR\" +\n#define ADD SUB - 1 *\nwhile ADD while + (");
    //     parser.read_global_statements();
    // }

    {
        let mut parser = Parser::new(include_str!("../example.c"));
        parser.read_global_statements();
    }
}

#[cfg(test)]
mod test {
    use std::process::Command;

    use crate::parser::Parser;

    #[test]
    fn test_cpp_result_eq() {
        let input = include_str!("../example.c");
        let mut parser = Parser::new(input);
        let parser_output = parser.read_global_statements();
        let output = Command::new("cpp")
            .arg("example.c")
            .output()
            .expect("Failed to execute command");
        let output = std::str::from_utf8(output.stdout.as_slice())
            .expect("Failed to convert output to String");
        let output = output
            .split("\n")
            .filter(|line| !line.starts_with("# "))
            .collect::<String>()
            .replace(" ", "");

        assert_eq!(parser_output, output);
    }
}
