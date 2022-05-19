#![warn(clippy::pedantic)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cast_sign_loss)]

#[cfg(feature = "bench")]
pub(crate) mod benching;

#[cfg(feature = "bench")]
pub use benching::{
    bench, bench_labeled, bench_with_configuration, bench_with_configuration_labeled,
    bench_with_setup, bench_with_setup_configuration, bench_with_setup_configuration_labeled,
    bench_with_setup_labeled,
};
#[cfg(feature = "bench")]
pub use output::analysis::criterion::{black_box, BenchmarkConfig};

#[cfg(any(feature = "bench", feature = "timer"))]
mod error;
#[cfg(any(feature = "bench", feature = "timer"))]
pub(crate) mod output;
#[cfg(feature = "timer")]
pub(crate) mod timing;
#[cfg(feature = "timer")]
pub use timing::{
    run_timed, run_timed_from_iterator, run_timed_times, Timeable, TimedIterator, TimingData,
};
