// Execution benchmarks: measure function call overhead and hot-loop perf

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use flame_core::{parse, validate};
use flame_cranelift::compile;
use flame_runtime::{imports::Imports, instance::Instance, val::Val};
use wat::parse_str;

fn bench_call_fib(c: &mut Criterion) {
    let wat = r#"
    (module
      (func $fib (export "fib") (param $n i32) (result i32)
        (if (i32.lt_s (local.get $n) (i32.const 2))
          (then (return (local.get $n))))
        (i32.add
          (call $fib (i32.sub (local.get $n) (i32.const 1)))
          (call $fib (i32.sub (local.get $n) (i32.const 2)))))
    )"#;
    let wasm = parse_str(wat).unwrap();
    let module = parse(&wasm).unwrap();
    validate(&module).unwrap();
    let compiled = compile(&module).unwrap();
    let imports = Imports::new();
    let instance = Instance::new(&module, compiled, &imports).unwrap();

    c.bench_function("fib(20)", |b| {
        b.iter(|| instance.call("fib", &[Val::I32(black_box(20))]).unwrap())
    });
}

criterion_group!(benches, bench_call_fib);
criterion_main!(benches);
