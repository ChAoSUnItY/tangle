use globals::get_source;
use parser::Parser;

mod defs;
mod globals;
mod lexer;
pub mod parser;

fn main() {
    let source = get_source();

    {
        let mut parser = Parser::new(include_str!("../example.c"));
        parser.read_global_statements();
    }
}

#[cfg(test)]
mod test {
    use std::{fs, process::Command};

    use test_case::test_case;

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

    #[test_case("alias.c"; "Test alias expansion")]
    #[test_case("macro.c"; "Test macro expansion")]
    fn test_cpp_result_eq_(file_path: &'static str) {
        let full_file_path = format!("test_suite/{}", file_path);
        let input = fs::read_to_string(&full_file_path)
            .expect("Unable to read file");
        let mut parser = Parser::new(&input);
        let parser_output = parser.read_global_statements();
        let output = Command::new("cpp")
            .arg(full_file_path)
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
