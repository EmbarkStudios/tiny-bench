use std::time::Duration;
#[cfg(bench)]
use tiny_bench::{black_box, BenchmarkConfig};

fn main() {
    #[cfg(bench)]
    bench_test_one();
    #[cfg(bench)]
    bench_test_two();
    #[cfg(bench)]
    bench_test_three();
    #[cfg(bench)]
    bench_test_four();
}

#[cfg(bench)]
fn bench_test_one() {
    tiny_bench::bench_labeled("test one", || {
        let mut v: Vec<i32> = Vec::with_capacity(10_000);
        for i in 0..black_box(10_000) {
            v.push(black_box(i));
        }
        let mut sum = 0;
        for i in black_box(v) {
            sum += black_box(i);
        }
        assert!(sum >= black_box(1));
    });
}

#[cfg(bench)]
fn bench_test_two() {
    tiny_bench::bench_with_setup_labeled(
        "test two",
        || {
            std::thread::sleep(Duration::from_micros(1));
            let mut v: Vec<i32> = Vec::with_capacity(10_000);
            for i in 0..10_000 {
                v.push(black_box(i));
            }
            v
        },
        |v| {
            let mut sum = 0;
            for i in black_box(v) {
                sum += black_box(i);
            }
            assert!(sum >= black_box(1));
        },
    );
}

#[cfg(bench)]
fn bench_test_three() {
    tiny_bench::bench_labeled("test three, empty", || {});
}

#[cfg(bench)]
fn bench_test_four() {
    tiny_bench::bench_with_configuration_labeled(
        "test four, max_it",
        &BenchmarkConfig {
            max_iterations: Some(5000),
            ..BenchmarkConfig::default()
        },
        || {},
    );
}
