use crate::output;
use crate::output::{ComparedStdout, LabeledOutput, Output, SimpleStdout};
use std::time::{Duration, Instant};

/// The simplest possible timed function that just runs some `FnMut` closure and returns the time it took
/// ```
/// use std::time::Duration;
/// use tiny_bench::run_timed;
/// let time = run_timed(|| std::thread::sleep(Duration::from_micros(5)));
/// assert!(time.as_micros() >= 5);
/// ```
pub fn run_timed<T, F: FnMut() -> T>(mut closure: F) -> Duration {
    let start = Instant::now();
    (closure)();
    Instant::now().duration_since(start)
}

/// Runs some closure `n` times and returns the data gathered
/// ```
/// use std::time::Duration;
/// use tiny_bench::run_timed_times;
/// let data = run_timed_times(100, || std::thread::sleep(Duration::from_micros(1)));
/// data.pretty_print();
/// ```
pub fn run_timed_times<T, F: FnMut() -> T>(iterations: usize, mut closure: F) -> TimingData {
    let mut elapsed = Duration::ZERO;
    let mut min_nanos = u128::MAX;
    let mut max_nanos = 0;
    for _ in 0..iterations {
        let start = Instant::now();
        closure();
        let run_elapsed = Instant::now().duration_since(start);
        let run_elapsed_nanos = run_elapsed.as_nanos();
        if run_elapsed_nanos < min_nanos {
            min_nanos = run_elapsed_nanos;
        }
        if run_elapsed_nanos > max_nanos {
            max_nanos = run_elapsed_nanos;
        }
        elapsed += run_elapsed;
    }
    TimingData {
        iterations: iterations as u128,
        min_nanos,
        max_nanos,
        elapsed: elapsed.as_nanos(),
    }
}

/// Drains an iterator and calls the closure with the yielded value, timing the closure's execution.
/// ```
/// use std::time::Duration;
/// use tiny_bench::run_timed_from_iterator;
/// let it = (0..100);
/// let mut v = Vec::with_capacity(100);
/// let mut counted_iterations = 0;
/// let data = run_timed_from_iterator(it, |i| {
///     v.push(i);
///     counted_iterations += 1;
/// });
/// assert_eq!(100, v.len());
/// assert_eq!(100, counted_iterations);
/// data.pretty_print();
/// ```
pub fn run_timed_from_iterator<T, R, F: FnMut(R) -> T, It>(
    iterator: It,
    mut closure: F,
) -> TimingData
where
    It: Iterator<Item = R>,
{
    let mut elapsed = Duration::ZERO;
    let mut min_nanos = u128::MAX;
    let mut max_nanos = 0;
    let mut iterations = 0;
    for v in iterator {
        let start = Instant::now();
        closure(v);
        let run_elapsed = Instant::now().duration_since(start);
        let run_elapsed_nanos = run_elapsed.as_nanos();
        if run_elapsed_nanos < min_nanos {
            min_nanos = run_elapsed_nanos;
        }
        if run_elapsed_nanos > max_nanos {
            max_nanos = run_elapsed_nanos;
        }
        elapsed += run_elapsed;
        iterations += 1;
    }
    TimingData {
        iterations,
        min_nanos,
        max_nanos,
        elapsed: elapsed.as_nanos(),
    }
}

/// Data collected after a timed run
#[derive(Copy, Clone, Debug)]
#[cfg(feature = "timer")]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub struct TimingData {
    /// The last amount of time elapsed for an iteration
    pub min_nanos: u128,
    /// The most amount of time elapsed for an iteration
    pub max_nanos: u128,
    /// The total elapsed time for all iterations combined
    pub elapsed: u128,
    /// How many iterations were ran
    pub iterations: u128,
}

#[cfg(feature = "timer")]
impl TimingData {
    /// Print the data with pretty colors to stdout
    pub fn pretty_print(&self) {
        output::print_timer_header("anonymous", self);
        output::print_elapsed(
            self.min_nanos as f64,
            self.elapsed as f64 / self.iterations as f64,
            self.max_nanos as f64,
        );
    }
}

/// A trait for allowing iterators to be used as timers
pub trait Timeable<It, T>: Sized
where
    It: Iterator<Item = T>,
{
    /// Time this iterator with an anonymous label
    /// ```
    /// use tiny_bench::Timeable;
    /// let v: Vec<i32> = (0..100)
    ///     .timed()
    ///     .collect();
    /// // Prints results when the iterator has been drained
    /// assert_eq!(100, v.len());
    /// ```
    fn timed(self) -> TimedIterator<It, T, SimpleStdout> {
        self.timed_labeled("anonymous")
    }

    /// Time this iterator with a specified label
    /// ```
    /// use tiny_bench::Timeable;
    /// let v: Vec<i32> = (0..100)
    ///     .timed_labeled("my_iterator_test")
    ///     .collect();
    /// // Prints results when the iterator has been drained
    /// assert_eq!(100, v.len());
    /// ```
    fn timed_labeled(self, label: &'static str) -> TimedIterator<It, T, SimpleStdout>;

    /// Time this iterator with an anonymous label and persist the result so that other anonymous
    /// time results will be compared with it when they run next
    fn timed_persisted(self) -> TimedIterator<It, T, ComparedStdout> {
        self.timed_persisted_labeled("anonymous")
    }

    /// Time this iterator with a custom label to separate different runs for comparison
    fn timed_persisted_labeled(self, label: &'static str) -> TimedIterator<It, T, ComparedStdout>;
}

impl<It, T> Timeable<It, T> for It
where
    It: Iterator<Item = T>,
{
    fn timed_labeled(self, label: &'static str) -> TimedIterator<It, T, SimpleStdout> {
        TimedIterator::new(self, LabeledOutput::new(label, SimpleStdout))
    }

    fn timed_persisted_labeled(self, label: &'static str) -> TimedIterator<It, T, ComparedStdout> {
        TimedIterator::new(self, LabeledOutput::new(label, ComparedStdout))
    }
}

/// An iterator that wraps another iterator and times each call to `next`
pub struct TimedIterator<It, T, O>
where
    It: Iterator<Item = T>,
{
    inner: It,
    iterations: u128,
    min_nanos: u128,
    max_nanos: u128,
    elapsed: Duration,
    out: LabeledOutput<O>,
}

impl<It, T, O> TimedIterator<It, T, O>
where
    It: Iterator<Item = T>,
{
    fn new(inner: It, out: LabeledOutput<O>) -> Self {
        TimedIterator {
            inner,
            iterations: 0,
            min_nanos: u128::MAX,
            max_nanos: 0,
            elapsed: Duration::ZERO,
            out,
        }
    }
}

impl<It, T, O> Iterator for TimedIterator<It, T, O>
where
    It: Iterator<Item = T>,
    O: Output,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let start = Instant::now();
        let maybe_item = self.inner.next();
        let run_elapsed = Instant::now().duration_since(start);
        if let Some(item) = maybe_item {
            let run_elapsed_nanos = run_elapsed.as_nanos();
            if run_elapsed_nanos < self.min_nanos {
                self.min_nanos = run_elapsed_nanos;
            }
            if run_elapsed_nanos > self.max_nanos {
                self.max_nanos = run_elapsed_nanos;
            }
            self.elapsed += run_elapsed;
            self.iterations += 1;
            Some(item)
        } else {
            self.out.dump(TimingData {
                min_nanos: self.min_nanos,
                max_nanos: self.max_nanos,
                elapsed: self.elapsed.as_nanos(),
                iterations: self.iterations,
            });
            None
        }
    }
}

#[cfg(test)]
#[cfg(feature = "timer")]
mod tests {
    use crate::timing::Timeable;

    #[test]
    fn time_iterator() {
        let _v: Vec<i32> = (0..100).timed().chain(0..10_000).timed().collect();
    }

    #[test]
    fn time_persisted_iterator() {
        for _ in 0..2 {
            let _v: Vec<i32> = (0..1_000_000).timed_persisted().collect();
        }
    }

    #[test]
    fn time_persisted_labled() {
        for _ in 0..2 {
            let _v: Vec<i32> = (0..1_000_000).timed_persisted_labeled("my_test").collect();
        }
    }
}
