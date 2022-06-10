use tiny_bench::black_box;

fn main() {
    let label = "compare_functions";
    tiny_bench::bench_labeled(label, my_slow_function);
    tiny_bench::bench_labeled(label, my_faster_function);
}

fn my_slow_function() {
    let mut num_iters = 0;
    for _ in 0..10_000 {
        num_iters += black_box(1);
    }
    assert_eq!(10_000, black_box(num_iters));
}

fn my_faster_function() {
    let mut num_iters = 0;
    for _ in 0..5_000 {
        num_iters += black_box(1);
    }
    assert_eq!(5_000, black_box(num_iters));
}
