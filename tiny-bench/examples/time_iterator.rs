use std::time::Duration;
use tiny_bench::Timeable;

pub fn main() {
    let v = (0..100)
        .inspect(|_a| {
            my_expensive_call();
        })
        .timed()
        .max();
    assert_eq!(99, v.unwrap());
}

fn my_expensive_call() {
    std::thread::sleep(Duration::from_millis(5));
}
