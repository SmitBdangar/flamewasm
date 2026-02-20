// Memory benchmarks: measure grow latency and sequential access throughput

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use flame_runtime::memory::{LinearMemory, PAGE_SIZE};

fn bench_memory_grow(c: &mut Criterion) {
    c.bench_function("memory_grow_1_page", |b| {
        b.iter(|| {
            let mut mem = LinearMemory::new(1, Some(64));
            black_box(mem.grow(black_box(1)))
        })
    });
}

fn bench_memory_sequential_write(c: &mut Criterion) {
    let mut mem = LinearMemory::new(16, None);
    c.bench_function("sequential_i32_store_1_page", |b| {
        b.iter(|| {
            let slice = mem.as_mut_slice();
            for i in (0..PAGE_SIZE).step_by(4) {
                slice[i..i + 4].copy_from_slice(&black_box(0xDEADBEEFu32).to_le_bytes());
            }
        })
    });
}

criterion_group!(benches, bench_memory_grow, bench_memory_sequential_write);
criterion_main!(benches);
