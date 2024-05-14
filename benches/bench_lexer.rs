use criterion::{criterion_group, criterion_main, Criterion};
use tangle::parser::Parser;

fn parse() {
    let input = include_str!("../example.c");
    let mut parser = Parser::new(input);
    let _ = parser.read_global_statements();
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("lexer", |b| b.iter(|| parse()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
