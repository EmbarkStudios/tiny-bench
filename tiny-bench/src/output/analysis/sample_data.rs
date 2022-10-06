use crate::benching::SamplingData;
use crate::output::analysis::criterion::{calculate_variance, SamplingDataSimpleAnalysis};

pub(crate) fn simple_analyze_sampling_data(
    sampling_data: &SamplingData,
) -> SamplingDataSimpleAnalysis {
    let mut min = f64::MAX;
    let mut max = 0f64;
    let mut total = 0f64;
    let mut total_elapsed = 0;
    let mut sample_averages = Vec::with_capacity(sampling_data.samples.len());
    for (num_samples, elapsed_nanos) in sampling_data
        .samples
        .iter()
        .copied()
        .zip(sampling_data.times.iter().copied())
    {
        let sample_average = elapsed_nanos as f64 / num_samples as f64;
        sample_averages.push(sample_average);
        if sample_average < min {
            min = sample_average;
        }
        if sample_average > max {
            max = sample_average;
        }
        total += sample_average;
        total_elapsed += elapsed_nanos;
    }
    let total_average = total / sampling_data.samples.len() as f64;
    let variance = calculate_variance(&sample_averages, total_average);
    SamplingDataSimpleAnalysis {
        elapsed: total_elapsed,
        min,
        max,
        average: total_average,
        variance,
        per_sample_average: sample_averages,
    }
}
