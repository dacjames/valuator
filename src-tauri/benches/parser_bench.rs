
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use finmod::{Parser, cell::Value};
use rust_decimal::Decimal;

fn bench_parse(c: &mut Criterion) {
  let mut p1 = Parser::new("1+1+23+17*(78+892/039)1+1+23+17*(78+892039)1+1+23+17*(78+8/92039)1+1+23+17*(78+892039)*'1+1+23+17*(78+892/039)1+1+23+17*(78+892039)1+1+23+17*(78+8/92039)1+1+23+17*(78+892039)'");
  c.bench_function("parser", |b|b.iter(||black_box({
    p1.parse()
  })));
  
  let mut p2 = Parser::new("3*7*(1+1)/2");
  c.bench_function("calc", |b|b.iter(||black_box({
    let node = p2.parse().unwrap();
    let res = node.eval(&p2);
  })));
}

criterion_group!(benches, bench_parse);
criterion_main!(benches);
