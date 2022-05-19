use tiny_bench::run_timed_from_iterator;

fn main() {
    let generator = 0..100;
    let mut spooky_calculation = 0;
    let results = run_timed_from_iterator(generator, |i| {
        spooky_calculation += i;
    });
    results.pretty_print();
    assert_eq!(4950, spooky_calculation);
}
