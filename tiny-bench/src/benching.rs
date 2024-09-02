use crate::output::analysis::criterion::calculate_iterations;
use crate::output::{
    fallback_to_anonymous_on_invalid_label, fmt_num, fmt_time, wrap_bold_green,
    wrap_high_intensity_white, Output,
};
use crate::{black_box, BenchmarkConfig};
use std::time::{Duration, Instant};

/// Will run the closure and print statistics from the benchmarking to stdout.
/// Will persist results under the anonymous label which is shared, making comparisons impossible
/// if running more than one (different) benchmark on the same project, ie. benching two different
/// functions
/// ```no_run
/// use tiny_bench::bench;
/// bench(|| {
///     // Some code that should be benched
/// })
/// ```
pub fn bench<T, F: FnMut() -> T>(closure: F) {
    bench_with_configuration(&BenchmarkConfig::default(), closure);
}

/// Will run the closure with a label, running with a label enables comparisons for subsequent runs.
/// ```no_run
/// use tiny_bench::bench_labeled;
/// bench_labeled("my_benchmark", || {
///     // Some code that should be benched
/// })
/// ```
pub fn bench_labeled<T, F: FnMut() -> T>(label: &'static str, closure: F) {
    bench_with_configuration_labeled(label, &BenchmarkConfig::default(), closure);
}

/// Will run the benchmark with the supplied configuration
/// ```no_run
/// use std::time::Duration;
/// use tiny_bench::{bench_with_configuration, BenchmarkConfig};
/// bench_with_configuration(&BenchmarkConfig {
///     measurement_time: Duration::from_secs(10),
///     ..BenchmarkConfig::default()
/// }, || {
///     // Some code that should be benched
/// })
/// ```
pub fn bench_with_configuration<T, F: FnMut() -> T>(cfg: &BenchmarkConfig, closure: F) {
    bench_with_configuration_labeled("anonymous", cfg, closure);
}

/// Will run the benchmark with the supplied configuration and a label
/// ```no_run
/// use tiny_bench::{bench_with_configuration_labeled, BenchmarkConfig};
/// bench_with_configuration_labeled("my_benchmark", &BenchmarkConfig::default(), || {
///     // Some code that should be benched
/// })
/// ```
pub fn bench_with_configuration_labeled<T, F: FnMut() -> T>(
    label: &'static str,
    cfg: &BenchmarkConfig,
    mut closure: F,
) {
    let label = fallback_to_anonymous_on_invalid_label(label);
    println!(
        "{} warming up for {}",
        wrap_bold_green(label),
        wrap_high_intensity_white(&fmt_time(cfg.warm_up_time.as_nanos() as f64))
    );
    let wu = run_warm_up(&mut closure, cfg.warm_up_time);
    let mean_execution_time = wu.elapsed.as_nanos() as f64 / wu.iterations as f64;
    let sample_size = cfg.num_samples as u64;
    let (iters, total_iters) =
        calculate_iters_and_total_iters(cfg, mean_execution_time, sample_size);

    println!(
        "{} mean warm up execution time {} running {} iterations",
        wrap_bold_green(label),
        wrap_high_intensity_white(&fmt_time(mean_execution_time)),
        wrap_high_intensity_white(&fmt_num(total_iters as f64))
    );
    let sampling_data = run(iters, closure);
    if cfg.dump_results_to_disk {
        crate::output::ComparedStdout.dump_sampling_data(label, &sampling_data, cfg, total_iters);
    } else {
        crate::output::SimpleStdout.dump_sampling_data(label, &sampling_data, cfg, total_iters);
    }
}

fn calculate_iters_and_total_iters(
    cfg: &BenchmarkConfig,
    mut mean_execution_time: f64,
    sample_size: u64,
) -> (Vec<u64>, u128) {
    if let Some(max_it) = cfg.max_iterations {
        (vec![max_it], u128::from(max_it))
    } else {
        // This can be arbitrarily small, causing an absurd amount of iterations.
        // Raise it to 1 nano -> max 5B iterations
        mean_execution_time = mean_execution_time.max(1.0);
        let iters = calculate_iterations(mean_execution_time, sample_size, cfg.measurement_time);
        let mut total_iters = 0u128;
        for count in iters.iter().copied() {
            total_iters = total_iters.saturating_add(u128::from(count));
        }
        (iters, total_iters)
    }
}

fn run<T, F: FnMut() -> T>(sample_sizes: Vec<u64>, mut closure: F) -> SamplingData {
    let times = sample_sizes
        .iter()
        .copied()
        .map(|it_count| {
            let start = Instant::now();
            for _ in 0..it_count {
                black_box(closure());
            }
            start.elapsed().as_nanos()
        })
        .collect();
    SamplingData {
        samples: sample_sizes,
        times,
    }
}

/// Fitting if some setup for the benchmark is required, and that setup should not be timed.
/// The setup will be run prior to each benchmarking run.
/// ```no_run
/// use tiny_bench::{bench_with_configuration_labeled, BenchmarkConfig};
/// bench_with_configuration_labeled("my_benchmark", &BenchmarkConfig::default(), || {
///     // Some code that should be benched
/// })
/// ```
pub fn bench_with_setup<T, R, F: FnMut(R) -> T, S: FnMut() -> R>(setup: S, closure: F) {
    bench_with_setup_configuration_labeled(
        "anonymous",
        &BenchmarkConfig::default(),
        setup,
        closure,
    );
}

/// Run bench with setup and a label
/// ```no_run
/// use std::time::Duration;
/// use tiny_bench::{bench_with_setup_labeled, BenchmarkConfig};
/// bench_with_setup_labeled("my_benchmark", || std::thread::sleep(Duration::from_micros(5)), |_| {
///     // Some code that should be benched
/// })
/// ```
pub fn bench_with_setup_labeled<T, R, F: FnMut(R) -> T, S: FnMut() -> R>(
    label: &'static str,
    setup: S,
    closure: F,
) {
    bench_with_setup_configuration_labeled(label, &BenchmarkConfig::default(), setup, closure);
}

/// Run bench with setup and configuration
/// ```no_run
/// use std::time::Duration;
/// use tiny_bench::{bench_with_setup_configuration, BenchmarkConfig};
/// bench_with_setup_configuration(&BenchmarkConfig::default(), || std::thread::sleep(Duration::from_micros(5)), |_| {
///     // Some code that should be benched
/// })
/// ```
pub fn bench_with_setup_configuration<T, R, F: FnMut(R) -> T, S: FnMut() -> R>(
    cfg: &BenchmarkConfig,
    setup: S,
    closure: F,
) {
    bench_with_setup_configuration_labeled("anonymous", cfg, setup, closure);
}

/// Run bench with setup, configuration, and a label
/// ```no_run
/// use std::time::Duration;
/// use tiny_bench::{bench_with_setup_configuration_labeled, BenchmarkConfig};
/// bench_with_setup_configuration_labeled("my_benchmark", &BenchmarkConfig::default(), || std::thread::sleep(Duration::from_micros(5)), |_| {
///     // Some code that should be benched
/// })
/// ```
pub fn bench_with_setup_configuration_labeled<T, R, F: FnMut(R) -> T, S: FnMut() -> R>(
    label: &'static str,
    cfg: &BenchmarkConfig,
    mut setup: S,
    mut closure: F,
) {
    let label = fallback_to_anonymous_on_invalid_label(label);
    let mut wu_routine = || {
        let input = (setup)();
        (closure)(input);
    };
    println!(
        "{} warming up for {}",
        wrap_bold_green(label),
        wrap_high_intensity_white(&fmt_time(cfg.warm_up_time.as_nanos() as f64))
    );
    let wu = run_warm_up(&mut wu_routine, cfg.warm_up_time);
    let mean_execution_time = wu.elapsed.as_nanos() as f64 / wu.iterations as f64;

    let sample_size = cfg.num_samples as u64;

    let (iters, total_iters) =
        calculate_iters_and_total_iters(cfg, mean_execution_time, sample_size);
    println!(
        "{} mean warm up execution time {} running {} iterations",
        wrap_bold_green(label),
        wrap_high_intensity_white(&fmt_time(mean_execution_time)),
        wrap_high_intensity_white(&fmt_num(total_iters as f64))
    );
    let sampling_data = run_with_setup(iters, setup, closure);
    if cfg.dump_results_to_disk {
        crate::output::ComparedStdout.dump_sampling_data(label, &sampling_data, cfg, total_iters);
    } else {
        crate::output::SimpleStdout.dump_sampling_data(label, &sampling_data, cfg, total_iters);
    }
}

fn run_with_setup<T, R, F: FnMut(R) -> T, S: FnMut() -> R>(
    sample_sizes: Vec<u64>,
    mut setup: S,
    mut closure: F,
) -> SamplingData {
    let times = sample_sizes
        .iter()
        .copied()
        .map(|it_count| {
            let inputs = (0..it_count).map(|_| setup()).collect::<Vec<_>>();

            let start = Instant::now();
            for i in inputs {
                black_box(closure(i));
            }
            start.elapsed().as_nanos()
        })
        .collect();
    SamplingData {
        samples: sample_sizes,
        times,
    }
}

fn run_warm_up<T, F: FnMut() -> T>(closure: &mut F, warmup_time: Duration) -> WarmupResults {
    let mut elapsed = Duration::ZERO;
    let mut iterations = 0u128;
    let mut run_iterations = 1u64;
    loop {
        let start = Instant::now();
        for _ in 0..run_iterations {
            closure();
        }
        elapsed += start.elapsed();
        iterations += u128::from(run_iterations);
        run_iterations = run_iterations.wrapping_mul(2);
        if elapsed >= warmup_time {
            return WarmupResults {
                iterations,
                elapsed,
            };
        }
    }
}

#[derive(Debug)]
struct WarmupResults {
    iterations: u128,
    elapsed: Duration,
}

#[derive(Debug)]
#[cfg(feature = "bench")]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub(crate) struct SamplingData {
    pub(crate) samples: Vec<u64>,
    pub(crate) times: Vec<u128>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn benches() {
        let closure = || {
            let mut sum = 0;
            for _ in 0..100 {
                sum += black_box(1);
            }
            assert_eq!(black_box(100), sum);
        };
        let cfg = BenchmarkConfig {
            measurement_time: Duration::from_millis(10),
            warm_up_time: Duration::from_millis(5),
            ..BenchmarkConfig::default()
        };
        bench_with_configuration(&cfg, closure);
    }
}
