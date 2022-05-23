use crate::output::analysis::random::Rng;
use crate::output::wrap_yellow;
use std::time::Duration;

/// Everything in this module is more or less copied from [criterion.rs](https://github.com/bheisler/criterion.rs)
/// with some rewrites to make it fit, the license is included in this file's directory

/// A function that is opaque to the optimizer, used to prevent the compiler from
/// optimizing away computations in a benchmark.
///
/// This variant is stable-compatible, but it may cause some performance overhead
/// or fail to prevent code from being eliminated.
#[allow(unsafe_code)]
pub fn black_box<T>(dummy: T) -> T {
    unsafe {
        let ret = std::ptr::read_volatile(&dummy);
        std::mem::forget(dummy);
        ret
    }
}

/// Struct containing all of the configuration options for a benchmark.
pub struct BenchmarkConfig {
    pub measurement_time: Duration,
    pub noise_threshold: f64,
    pub nresamples: usize,
    pub sample_size: usize,
    pub significance_level: f64,
    pub warm_up_time: Duration,
    pub dump_results_to_disk: bool,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        BenchmarkConfig {
            measurement_time: Duration::from_secs(5),
            noise_threshold: 0.01,
            nresamples: 100_000,
            sample_size: 100,
            significance_level: 0.05,
            warm_up_time: Duration::from_secs(3),
            dump_results_to_disk: true,
        }
    }
}

pub(crate) fn calculate_iterations(
    warmup_mean_execution_time: f64,
    mut sample_size: u64,
    target_time: Duration,
) -> Vec<u64> {
    let met = warmup_mean_execution_time;
    let m_ns = target_time.as_nanos();
    // Solve: [d + 2*d + 3*d + ... + n*d] * met = m_ns

    let mut total_runs = sample_size * (sample_size + 1) / 2;
    let mut d = ((m_ns as f64 / met / total_runs as f64).ceil() as u64).max(1);
    let mut expected_nanoseconds = total_runs as f64 * d as f64 * met;
    if d == 1 {
        let actual_time = Duration::from_nanos(expected_nanoseconds as u64);
        println!(
            "{} You may wish to increase target time to {:.1?}. Will compress sample size",
            wrap_yellow(&format!(
                "Unable to complete {} samples in {:.1?}",
                sample_size, target_time
            )),
            actual_time
        );
        while d == 1 && sample_size > 1 {
            sample_size -= 1;
            total_runs = sample_size * (sample_size + 1) / 2;
            d = ((m_ns as f64 / met / total_runs as f64).ceil() as u64).max(1);
            expected_nanoseconds = total_runs as f64 * d as f64 * met;
        }
        println!(
            "{} with an expected running time of {:.1?}",
            wrap_yellow(&format!("Compressed sample_size to {}", sample_size)),
            Duration::from_nanos(expected_nanoseconds as u64)
        );
    }

    (1..=sample_size).map(|a| a * d).collect()
}

pub(crate) fn test_t_between_samples(a: &[f64], b: &[f64]) -> f64 {
    let a_mean = calculate_mean(a);
    let b_mean = calculate_mean(b);
    let a_var = calculate_variance(a, a_mean);
    let b_var = calculate_variance(b, b_mean);
    let a_len = a.len() as f64;
    let b_len = b.len() as f64;
    let mean_diff = a_mean - b_mean;
    let d = (a_var / a_len + b_var / b_len).sqrt();
    mean_diff / d
}

pub(crate) fn calculate_mean(a: &[f64]) -> f64 {
    a.iter().sum::<f64>() / a.len() as f64
}

pub(crate) fn calculate_variance(a: &[f64], mean: f64) -> f64 {
    let sum = a
        .iter()
        .copied()
        .map(|val| (val - mean).powi(2))
        .sum::<f64>();
    sum / a.len() as f64
}

pub(crate) fn resample(a: &[f64], b: &[f64], times: usize) -> Vec<f64> {
    let a_len = a.len();
    let mut combined = Vec::with_capacity(a_len + b.len());
    combined.extend_from_slice(a);
    combined.extend_from_slice(b);
    let mut rng = Rng::new();
    let combined_len = combined.len();
    let mut distributions = Vec::new();
    for _ in 0..times {
        let mut sample = Vec::with_capacity(combined_len);
        for _ in 0..combined_len {
            let index = (rng.next() % combined.len() as u64) as usize;
            sample.push(combined[index]);
        }
        let sample_a = Vec::from(&sample[..a_len]);
        let sample_b = Vec::from(&sample[a_len..]);
        let t = test_t_between_samples(&sample_a, &sample_b);
        distributions.push(t);
    }
    distributions
}

pub(crate) fn calculate_p_value(total_t: f64, distribution: &[f64]) -> f64 {
    let hits = distribution.iter().filter(|x| x < &&total_t).count();
    let tails = 2; // I don't know what this is, Two-tailed significance testing something something
    let min = std::cmp::min(hits, distribution.len() - hits);
    min as f64 / (distribution.len() * tails) as f64
}

pub(crate) struct SamplingDataSimpleAnalysis {
    pub(crate) elapsed: u128,
    pub(crate) min: f64,
    pub(crate) max: f64,
    pub(crate) average: f64,
    pub(crate) per_sample_average: Vec<f64>,
}
