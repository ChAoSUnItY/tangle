use tangle::parser::Parser;

fn main() {
    let mut parser = Parser::new("../main.c", include_str!("../main.c"));
    let result = parser.read_global_statements();
    println!("{result}");
}
