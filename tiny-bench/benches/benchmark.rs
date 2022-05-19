use tiny_bench::black_box;

fn main() {
    bench_test_one();
    bench_test_two();
}

pub fn bench_test_one() {
    tiny_bench::bench(|| {
        let mut v: Vec<i32> = Vec::with_capacity(10_000);
        for i in 0..10_000 {
            v.push(black_box(i));
        }
        let mut sum = 0;
        for i in v {
            sum += black_box(i);
        }
        assert!(sum >= black_box(1));
    })
}

pub fn bench_test_two() {
    tiny_bench::bench_with_setup(
        || Vec::with_capacity(10_000),
        |mut v| {
            for i in 0..10_000 {
                v.push(black_box(i));
            }
            let mut sum = 0;
            for i in v {
                sum += black_box(i);
            }
            assert!(sum >= black_box(1));
        },
    )
}
