use tiny_bench::BenchmarkConfig;

fn main() {
    let v = vec![10, 5, 3, 8, 7, 5];
    tiny_bench::bench_with_configuration(&BenchmarkConfig::default(), || {
        let sorted = bad_sort(v.clone());
        assert_eq!(vec![3, 5, 5, 7, 8, 10], sorted);
    })
}

fn bad_sort(mut v: Vec<u32>) -> Vec<u32> {
    let mut sorted = Vec::with_capacity(v.len());
    while !v.is_empty() {
        let mut min_val = u32::MAX;
        let mut min_index = 0;
        for i in 0..v.len() {
            if v[i] < min_val {
                min_index = i;
                min_val = v[i];
            }
        }
        sorted.push(min_val);
        v.remove(min_index);
    }
    sorted
}
