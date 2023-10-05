#![warn(clippy::pedantic)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cast_sign_loss)]
#![deny(missing_docs)]
//! `tiny-bench`, a tiny benchmarking library.
//! The crate is divided into two sections, benchmarking and timing.
//! Benchmarking provides tools to measure code execution, show statistics about that execution,
//! and compare those statistics to previous runs.
//! Timing provides tools to time code. Timing how long a closure runs, or how long an iterator runs.

#[cfg(feature = "bench")]
pub(crate) mod benching;

#[cfg(feature = "bench")]
pub use benching::{
    bench, bench_labeled, bench_with_configuration, bench_with_configuration_labeled,
    bench_with_setup, bench_with_setup_configuration, bench_with_setup_configuration_labeled,
    bench_with_setup_labeled,
};
#[cfg(feature = "bench")]
pub use output::analysis::criterion::BenchmarkConfig;
#[cfg(feature = "bench")]
pub use std::hint::black_box;

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
