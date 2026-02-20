// Criterion benchmarks for FlameWasm
//
// Run with: cargo bench

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use flame_core::{parse, validate};
use flame_cranelift::compile;
use wat::parse_str;

fn bench_compile(c: &mut Criterion) {
    // A non-trivial fibonacci module
    let wat = r#"
    (module
      (func $fib (export "fib") (param $n i32) (result i32)
        (if (i32.lt_s (local.get $n) (i32.const 2))
          (then (return (local.get $n))))
        (i32.add
          (call $fib (i32.sub (local.get $n) (i32.const 1)))
          (call $fib (i32.sub (local.get $n) (i32.const 2)))))
    )"#;
    let wasm = parse_str(wat).expect("WAT parse");
    let module = parse(&wasm).expect("parse");
    validate(&module).expect("validate");

    c.bench_function("compile_fibonacci", |b| {
        b.iter(|| compile(black_box(&module)).expect("compile"))
    });
}

fn bench_parse(c: &mut Criterion) {
    let wat = r#"
    (module
      (func (export "add") (param i32 i32) (result i32)
        (i32.add (local.get 0) (local.get 1)))
    )"#;
    let wasm = parse_str(wat).expect("WAT parse");

    c.bench_function("parse_simple_module", |b| {
        b.iter(|| parse(black_box(&wasm)).expect("parse"))
    });
}

criterion_group!(benches, bench_compile, bench_parse);
criterion_main!(benches);
