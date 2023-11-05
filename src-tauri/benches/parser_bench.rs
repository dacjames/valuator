
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use finmod::Parser;

fn bench_parse(c: &mut Criterion) {
    let mut p = Parser::new("1+1+23+17*(78+892/039)1+1+23+17*(78+892039)1+1+23+17*(78+8/92039)1+1+23+17*(78+892039)*'1+1+23+17*(78+892/039)1+1+23+17*(78+892039)1+1+23+17*(78+8/92039)1+1+23+17*(78+892039)'");
    c.bench_function("parser", |b|b.iter(||black_box({
        p.parse()
    })));
}



criterion_group!(benches, bench_parse);
criterion_main!(benches);
