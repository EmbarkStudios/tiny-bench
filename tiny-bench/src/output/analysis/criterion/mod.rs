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
///
/// ```
/// use tiny_bench::black_box;
/// for i in 0..10 {
///     assert_eq!(i, black_box(i));
/// }
/// ```
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
    /// How long the bench 'should' run, `num_samples` is prioritized so benching will take
    /// longer to be able to collect `num_samples` if the code to be benched is slower
    /// than this time limit allowed.
    pub measurement_time: Duration,
    /// How many resamples should be done
    pub num_resamples: usize,
    /// Recommended at least 50, above 100 <https://en.wikipedia.org/wiki/Bootstrapping_(statistics)#Recommendations>
    /// doesn't seem to yield a significantly different result
    pub num_samples: usize,
    /// How long the bench should warm up
    pub warm_up_time: Duration,
    /// Puts results in target/tiny-bench/label/.. if target can be found.
    /// used for comparing previous runs
    pub dump_results_to_disk: bool,

    /// Sets a hard ceiling on max iterations, overriding the heuristic calculations for iteration
    /// count. A rule of thumb; if this is used, the results are unlikely to be statistically
    /// significant.
    pub max_iterations: Option<u64>,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        BenchmarkConfig {
            measurement_time: Duration::from_secs(5),
            num_resamples: 100_000,
            num_samples: 100,
            warm_up_time: Duration::from_secs(3),
            dump_results_to_disk: true,
            max_iterations: None,
        }
    }
}

pub(crate) fn calculate_iterations(
    warmup_mean_execution_time: f64,
    num_samples: u64,
    target_time: Duration,
) -> Vec<u64> {
    let met = warmup_mean_execution_time;
    let m_ns = target_time.as_nanos();
    // Solve: [d + 2*d + 3*d + ... + n*d] * met = m_ns

    let total_runs = num_samples * (num_samples + 1) / 2;
    let d = ((m_ns as f64 / met / total_runs as f64).ceil() as u64).max(1);
    let expected_nanoseconds = total_runs as f64 * d as f64 * met;
    if d == 1 {
        let actual_time = Duration::from_nanos(expected_nanoseconds as u64);
        println!(
            "{} You may wish to increase target time to {:.1?} or lower the requested number of samples",
            wrap_yellow(&format!(
                "Unable to complete {num_samples} samples in {target_time:.1?}"
            )),
            actual_time
        );
    }

    (1..=num_samples).map(|a| a * d).collect()
}

pub(crate) fn calculate_t_value(sample_a: &[f64], sample_b: &[f64]) -> f64 {
    let a_mean = calculate_mean(sample_a);
    let b_mean = calculate_mean(sample_b);
    let a_var = calculate_variance(sample_a, a_mean);
    let b_var = calculate_variance(sample_b, b_mean);
    let a_len = sample_a.len() as f64;
    let b_len = sample_b.len() as f64;
    let mean_diff = a_mean - b_mean;
    let d = (a_var / a_len + b_var / b_len).sqrt();
    mean_diff / d
}

pub(crate) fn calculate_mean(a: &[f64]) -> f64 {
    a.iter().sum::<f64>() / a.len() as f64
}

pub(crate) fn calculate_variance(sample: &[f64], mean: f64) -> f64 {
    let sum = sample
        .iter()
        .copied()
        .map(|val| (val - mean).powi(2))
        .sum::<f64>();
    sum / (sample.len() as f64 - 1f64) // use n - 1 when measuring variance from a sample
}

pub(crate) fn resample(sample_a: &[f64], sample_b: &[f64], times: usize) -> Vec<f64> {
    let a_len = sample_a.len();
    let mut combined = Vec::with_capacity(a_len + sample_b.len());
    combined.extend_from_slice(sample_a);
    combined.extend_from_slice(sample_b);
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
        let t = calculate_t_value(&sample_a, &sample_b);
        distributions.push(t);
    }
    distributions
}

pub(crate) fn calculate_p_value(total_t: f64, distribution: &[f64]) -> f64 {
    let hits = distribution.iter().filter(|x| x < &&total_t).count();
    let tails = 2; // I don't know what this is, Two-tailed significance testing something something
    let min = std::cmp::min(hits, distribution.len() - hits);
    (min * tails) as f64 / distribution.len() as f64
}

#[inline]
pub(crate) fn calculate_median(sample: &mut Vec<f64>) -> f64 {
    sample.sort_by(f64::total_cmp);
    sample.get(sample.len() / 2).copied().unwrap_or_default()
}

pub(crate) struct SamplingDataSimpleAnalysis {
    pub(crate) elapsed: u128,
    pub(crate) min: f64,
    pub(crate) max: f64,
    pub(crate) average: f64,
    pub(crate) median: f64,
    pub(crate) variance: f64,
    pub(crate) stddev: f64,
    pub(crate) per_sample_average: Vec<f64>,
}

#[cfg(test)]
mod tests {
    use crate::output::analysis::criterion::{
        calculate_mean, calculate_t_value, calculate_variance,
    };

    #[test]
    fn calculates_mean() {
        let data = vec![46.0, 69.0, 32.0, 60.0, 52.0, 41.0];
        assert!(calculate_mean(&data) - 50.0 < 0.0000_001);
    }

    #[test]
    fn calculates_variance() {
        let data = vec![46.0, 69.0, 32.0, 60.0, 52.0, 41.0];
        assert!(calculate_variance(&data, 50.0) - 177.2 < 0.00001);
    }

    #[test]
    fn calculate_t() {
        let sample_a = vec![19.7, 20.4, 19.6, 17.8, 18.5, 18.9, 18.3, 18.9, 19.5, 21.95];
        let sample_b = vec![
            28.3, 26.7, 20.1, 23.3, 25.2, 22.1, 17.7, 27.6, 20.6, 13.7, 23.2, 17.5, 20.6, 18.0,
            23.9, 21.6, 24.3, 20.4, 23.9, 13.3,
        ];
        assert!(calculate_t_value(&sample_a, &sample_b).abs() - 2.24787 < 0.0001);
    }
}
