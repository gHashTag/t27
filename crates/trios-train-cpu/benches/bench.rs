//! Criterion benchmarks for trios-train-cpu

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use trios_train_cpu::{matmul, gelu, layer_norm, softmax, LayerDims};

fn bench_matmul(c: &mut Criterion) {
    let mut group = c.benchmark_group("matmul");

    for size in [64, 128, 256, 512].iter() {
        let m = *size;
        let k = *size;
        let n = *size;

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            let a = vec![1.0f32; m * k];
            let b_mat = vec![1.0f32; k * n];
            let mut c = vec![0.0f32; m * n];

            b.iter(|| {
                matmul(black_box(&a), black_box(&b_mat), black_box(&mut c), m, k, n);
                black_box(&c);
            });
        });
    }

    group.finish();
}

fn bench_matmul_igla_dimensions(c: &mut Criterion) {
    let dims = LayerDims::default();
    let batch_size = 4;
    let seq_len = 128;

    let m = batch_size * seq_len;
    let k = dims.d_model;
    let n = dims.d_model;

    let a = vec![1.0f32; m * k];
    let b_mat = vec![1.0f32; k * n];
    let mut c = vec![0.0f32; m * n];

    c.bench_function("matmul_igla_forward", |b| {
        b.iter(|| {
            matmul(black_box(&a), black_box(&b_mat), black_box(&mut c), m, k, n);
            black_box(&c);
        });
    });
}

fn bench_gelu(c: &mut Criterion) {
    let mut group = c.benchmark_group("gelu");

    for size in [128, 512, 2048, 8192].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut x = vec![1.0f32; size];

            b.iter(|| {
                let mut x_copy = x.clone();
                gelu(black_box(&mut x_copy));
                black_box(&x_copy);
            });
        });
    }

    group.finish();
}

fn bench_layer_norm(c: &mut Criterion) {
    let mut group = c.benchmark_group("layer_norm");

    for size in [128, 512, 2048, 8192].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut x = vec![1.0f32; size];

            b.iter(|| {
                let mut x_copy = x.clone();
                layer_norm(black_box(&mut x_copy), 1e-5);
                black_box(&x_copy);
            });
        });
    }

    group.finish();
}

fn bench_softmax(c: &mut Criterion) {
    let mut group = c.benchmark_group("softmax");

    for vocab_size in [1000, 5000, 10000, 32000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(vocab_size),
            vocab_size,
            |b, &vocab_size| {
                let mut x = vec![1.0f32; vocab_size];

                b.iter(|| {
                    let mut x_copy = x.clone();
                    softmax(black_box(&mut x_copy));
                    black_box(&x_copy);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_matmul,
    bench_matmul_igla_dimensions,
    bench_gelu,
    bench_layer_norm,
    bench_softmax
);
criterion_main!(benches);
