pub(crate) mod analysis;
pub(crate) mod disk;
pub(crate) mod ser;

#[cfg(feature = "bench")]
use crate::benching::SamplingData;
#[cfg(feature = "bench")]
use crate::output::analysis::criterion::{
    calculate_p_value, calculate_t_value, resample, BenchmarkConfig, SamplingDataSimpleAnalysis,
};
#[cfg(feature = "bench")]
use crate::output::analysis::sample_data::simple_analyze_sampling_data;
#[cfg(feature = "timer")]
use crate::timing::TimingData;

/// Percentage increase which is deemed to be big enough to matter.
/// Only used for highlighting output
#[cfg(feature = "timer")]
const TIMING_NOISE_THRESHOLD: f64 = 5.0;

/// Percentage increase which is deemed to be big enough to matter.
/// Only used for highlighting output
#[cfg(feature = "bench")]
const NOISE_THRESHOLD: f64 = 1.0;

/// p-value under which a result is deemed significant enough to matter.
/// Only used for highlighting output
#[cfg(feature = "bench")]
const SIGNIFICANCE_LEVEL: f64 = 0.05;

#[cfg(feature = "timer")]
pub(crate) struct LabeledOutput<Output> {
    label: &'static str,
    out: Output,
}

#[cfg(feature = "timer")]
impl<O> LabeledOutput<O> {
    pub(crate) fn new(label: &'static str, out: O) -> Self {
        Self { label, out }
    }
}

#[cfg(feature = "timer")]
impl<O> LabeledOutput<O>
where
    O: Output,
{
    pub(crate) fn dump(&self, data: TimingData) {
        self.out.dump_timing_data(self.label, data);
    }
}

pub(crate) trait Output {
    #[cfg(feature = "timer")]
    fn dump_timing_data(&self, label: &'static str, data: TimingData);

    #[cfg(feature = "bench")]
    fn dump_sampling_data(
        &self,
        label: &'static str,
        sampling_data: &SamplingData,
        cfg: &BenchmarkConfig,
        total_iters: u128,
    );
}

/// Just prints the results straight to stdout
pub struct SimpleStdout;

impl Output for SimpleStdout {
    #[cfg(feature = "timer")]
    fn dump_timing_data(&self, label: &'static str, data: TimingData) {
        print_timer_header(label, &data);
        let mean = data.elapsed as f64 / data.iterations as f64;
        timer_print_elapsed(data.min_nanos as f64, mean, data.max_nanos as f64);
    }

    #[cfg(feature = "bench")]
    fn dump_sampling_data(
        &self,
        label: &'static str,
        sampling_data: &SamplingData,
        cfg: &BenchmarkConfig,
        total_iters: u128,
    ) {
        let analysis = simple_analyze_sampling_data(sampling_data);
        print_sample_header(label, total_iters, analysis.elapsed, cfg.num_samples as u64);
        print_analysis(&analysis);
    }
}

/// Checks if there has previously been any results dumped to target and compares with those
pub struct ComparedStdout;

impl Output for ComparedStdout {
    #[cfg(feature = "timer")]
    fn dump_timing_data(&self, label: &'static str, data: TimingData) {
        let mean = data.elapsed as f64 / data.iterations as f64;
        let maybe_old = disk::try_read_last_results(label);
        print_timer_header(label, &data);
        timer_print_elapsed(data.min_nanos as f64, mean, data.max_nanos as f64);
        match maybe_old {
            Ok(Some(old)) => {
                let min_change = (data.min_nanos as f64 / old.min_nanos as f64 - 1f64) * 100f64;
                let max_change = (data.max_nanos as f64 / old.max_nanos as f64 - 1f64) * 100f64;
                let mean_change =
                    (mean / (old.elapsed as f64 / old.iterations as f64) - 1f64) * 100f64;
                let mean_comparison = if mean_change >= TIMING_NOISE_THRESHOLD {
                    MeanComparison::new(mean_change, Comparison::Better)
                } else if mean_change <= -TIMING_NOISE_THRESHOLD {
                    MeanComparison::new(mean_change, Comparison::Worse)
                } else {
                    MeanComparison::new(mean_change, Comparison::Same)
                };
                print_cmp(
                    min_change,
                    &mean_comparison,
                    max_change,
                    "p=? single sample",
                );
            }
            Err(e) => {
                println!(
                    "{}, cause {e}",
                    wrap_high_insensity_red("Failed to read last results")
                );
            }
            _ => {}
        }
        disk::try_write_results(label, data);
    }

    #[cfg(feature = "bench")]
    fn dump_sampling_data(
        &self,
        label: &'static str,
        sampling_data: &SamplingData,
        cfg: &BenchmarkConfig,
        total_iters: u128,
    ) {
        let analysis = simple_analyze_sampling_data(sampling_data);
        print_sample_header(label, total_iters, analysis.elapsed, cfg.num_samples as u64);
        print_analysis(&analysis);
        match disk::try_read_last_simpling(label) {
            Ok(Some(last)) => {
                let old_analysis = simple_analyze_sampling_data(&last);
                let min_change = (analysis.min / old_analysis.min - 1f64) * 100f64;
                let max_change = (analysis.max / old_analysis.max - 1f64) * 100f64;
                let mean_change = (analysis.average / old_analysis.average - 1f64) * 100f64;
                let t = calculate_t_value(
                    &analysis.per_sample_average,
                    &old_analysis.per_sample_average,
                );
                let t_distribution = resample(
                    &analysis.per_sample_average,
                    &old_analysis.per_sample_average,
                    cfg.num_resamples,
                );
                let p = calculate_p_value(t, &t_distribution);
                let mean_change = if mean_change.abs() >= NOISE_THRESHOLD && p <= SIGNIFICANCE_LEVEL
                {
                    if mean_change > 0.0 {
                        MeanComparison::new(mean_change, Comparison::Worse)
                    } else if mean_change < 0.0 {
                        MeanComparison::new(mean_change, Comparison::Better)
                    } else {
                        MeanComparison::new(mean_change, Comparison::Same)
                    }
                } else {
                    MeanComparison::new(mean_change, Comparison::Same)
                };
                print_cmp(min_change, &mean_change, max_change, &format!("p = {p:.2}"));
            }
            Err(e) => {
                println!(
                    "{}, cause {e}",
                    wrap_high_insensity_red("Failed to read last sample")
                );
            }
            _ => {}
        }

        disk::try_write_last_simpling(label, sampling_data);
    }
}

#[cfg(feature = "timer")]
pub(crate) fn print_timer_header(label: &'static str, data: &TimingData) {
    println!(
        "{} [{} iterations in {}]:",
        wrap_bold_green(label),
        fmt_num(data.iterations as f64),
        fmt_time(data.elapsed as f64)
    );
}

#[cfg(feature = "bench")]
pub(crate) fn print_sample_header(
    label: &'static str,
    total_iterations: u128,
    total_elapsed: u128,
    num_samples: u64,
) {
    println!(
        "{} [{} iterations in {} with {} samples]:",
        wrap_bold_green(label),
        fmt_num(total_iterations as f64),
        fmt_time(total_elapsed as f64),
        fmt_num(num_samples as f64)
    );
}

#[cfg(feature = "bench")]
pub(crate) fn print_analysis(analysis: &SamplingDataSimpleAnalysis) {
    // Variance has the unit T-squared,
    println!(
        "\telapsed\t[{} {} {}]:\t[{} {} {}] (sample data: med = {}, var = {}², stddev = {})",
        wrap_gray("min"),
        wrap_high_intensity_white("mean"),
        wrap_gray("max"),
        wrap_gray(&fmt_time(analysis.min)),
        wrap_high_intensity_white(&fmt_time(analysis.average)),
        wrap_gray(&fmt_time(analysis.max)),
        fmt_time(analysis.median),
        fmt_time(analysis.variance),
        fmt_time(analysis.stddev),
    );
}

#[cfg(feature = "timer")]
pub(crate) fn timer_print_elapsed(min: f64, mean: f64, max: f64) {
    // Variance has the unit T-squared,
    println!(
        "\telapsed\t[{} {} {}]:\t[{} {} {}]",
        wrap_gray("min"),
        wrap_high_intensity_white("mean"),
        wrap_gray("max"),
        wrap_gray(&fmt_time(min)),
        wrap_high_intensity_white(&fmt_time(mean)),
        wrap_gray(&fmt_time(max)),
    );
}

pub(crate) struct MeanComparison {
    mean: f64,
    comparison: Comparison,
}

impl MeanComparison {
    pub(crate) fn new(mean: f64, comparison: Comparison) -> Self {
        Self { mean, comparison }
    }

    pub(crate) fn format(&self) -> String {
        match self.comparison {
            Comparison::Worse => wrap_high_insensity_red(&fmt_change(self.mean)),
            Comparison::Same => wrap_high_intensity_white(&fmt_change(self.mean)),
            Comparison::Better => wrap_high_intensity_green(&fmt_change(self.mean)),
        }
    }
}

pub enum Comparison {
    Worse,
    Same,
    Better,
}

pub(crate) fn print_cmp(min: f64, mean: &MeanComparison, max: f64, reliability_comment: &str) {
    println!(
        "\tchange\t[{} {} {}]:\t[{} {} {}] ({reliability_comment})",
        wrap_gray("min"),
        wrap_high_intensity_white("mean"),
        wrap_gray("max"),
        wrap_gray(&fmt_change(min)),
        mean.format(),
        wrap_gray(&fmt_change(max)),
    );
}

const NANO_LIMIT: f64 = 1000f64;
const MICRO_LIMIT: f64 = NANO_LIMIT * 1000f64;
const MILLI_LIMIT: f64 = MICRO_LIMIT * 1000f64;

pub(crate) fn wrap_bold_green(text: &str) -> String {
    format!("\x1b[1;32m{text}\x1b[0m")
}

pub(crate) fn wrap_high_intensity_green(text: &str) -> String {
    format!("\x1b[0;92m{text}\x1b[0m")
}

pub(crate) fn wrap_yellow(text: &str) -> String {
    format!("\x1b[0;93m{text}\x1b[0m")
}

pub(crate) fn wrap_high_insensity_red(text: &str) -> String {
    format!("\x1b[0;91m{text}\x1b[0m")
}

pub(crate) fn wrap_gray(text: &str) -> String {
    format!("\x1b[0;37m{text}\x1b[0m")
}

pub(crate) fn wrap_high_intensity_white(text: &str) -> String {
    format!("\x1b[0;97m{text}\x1b[0m")
}

pub(crate) fn fmt_time(time: f64) -> String {
    // Nanos
    if time < NANO_LIMIT {
        format!("{:.2}ns", time)
    } else if time < MICRO_LIMIT {
        format!("{:.2}µs", time / NANO_LIMIT)
    } else if time < MILLI_LIMIT {
        format!("{:.2}ms", time / MICRO_LIMIT)
    } else {
        format!("{:.2}s", time / MILLI_LIMIT)
    }
}

fn fmt_change(change: f64) -> String {
    format!("{:.4}%", change)
}

pub(crate) fn fmt_num(num: f64) -> String {
    if num < NANO_LIMIT {
        format!("{:.1}", num)
    } else if num < MICRO_LIMIT {
        format!("{:.1} thousand", num / NANO_LIMIT)
    } else if num < MILLI_LIMIT {
        format!("{:.1}M", num / MICRO_LIMIT)
    } else {
        format!("{:.1}B", num / MILLI_LIMIT)
    }
}

/// Some illegal filename symbols, not meant to be exhaustive but good enough
const ILLEGAL: [char; 10] = [
    // Linux
    '/', '\0', // Windows
    ':', '<', '>', '"', '\\', '|', '?', '*',
];

#[cfg_attr(test, derive(Eq, PartialEq, Debug))]
pub(crate) enum LabelValidationResult {
    Valid,
    Invalid(&'static str),
}

pub(crate) fn fallback_to_anonymous_on_invalid_label(label: &'static str) -> &'static str {
    if let LabelValidationResult::Invalid(reason) = validate_label(label) {
        println!(
            "{} falling back to 'anonymous'.",
            wrap_high_insensity_red(reason)
        );
        "anonymous"
    } else {
        label
    }
}

fn validate_label(label: &'static str) -> LabelValidationResult {
    for ch in ILLEGAL {
        if label.contains(ch) {
            return LabelValidationResult::Invalid("Label contains illegal character {ch}");
        }
    }
    for ch in 0..32 {
        let ascii_ctrl = char::from(ch);
        if label.contains(ascii_ctrl) {
            return LabelValidationResult::Invalid(
                "Label contains illegal ascii-control character number {ch}",
            );
        }
    }
    if label.ends_with('.') {
        return LabelValidationResult::Invalid("Label cannot end with dot");
    }
    if label.ends_with(' ') {
        return LabelValidationResult::Invalid("Label cannot end with a space");
    }
    LabelValidationResult::Valid
}

#[cfg(test)]
mod tests {
    use crate::output::{fmt_change, fmt_num, fmt_time, validate_label, LabelValidationResult};

    #[test]
    fn validates_label() {
        assert_eq!(LabelValidationResult::Valid, validate_label("Hello!"));
        assert_eq!(
            LabelValidationResult::Valid,
            validate_label("Some,weird_name_but.okay.png")
        );
        assert!(matches!(
            validate_label("."),
            LabelValidationResult::Invalid(_)
        ));
        assert!(matches!(
            validate_label("hello!."),
            LabelValidationResult::Invalid(_)
        ));
        assert!(matches!(
            validate_label("hello! "),
            LabelValidationResult::Invalid(_)
        ));
        assert!(matches!(
            validate_label("bad/label"),
            LabelValidationResult::Invalid(_)
        ));
        assert!(matches!(
            validate_label("bad:label"),
            LabelValidationResult::Invalid(_)
        ));
        assert!(matches!(
            validate_label("bad>label"),
            LabelValidationResult::Invalid(_)
        ));
        assert!(matches!(
            validate_label("bad<label"),
            LabelValidationResult::Invalid(_)
        ));
        assert!(matches!(
            validate_label("bad\0label"),
            LabelValidationResult::Invalid(_)
        ));
        assert!(matches!(
            validate_label("bad\\label"),
            LabelValidationResult::Invalid(_)
        ));
        assert!(matches!(
            validate_label("bad\"label"),
            LabelValidationResult::Invalid(_)
        ));
        assert!(matches!(
            validate_label("bad|label"),
            LabelValidationResult::Invalid(_)
        ));
        assert!(matches!(
            validate_label("bad?label"),
            LabelValidationResult::Invalid(_)
        ));
        assert!(matches!(
            validate_label("bad*label"),
            LabelValidationResult::Invalid(_)
        ));
    }

    #[test]
    fn formats_time() {
        assert_eq!("5.15ns", &fmt_time(5.15));
        assert_eq!("1.50µs", &fmt_time(1500.0));
        assert_eq!("3.33ms", &fmt_time(3_330_000.0));
        assert_eq!("5.79s", &fmt_time(5_790_000_000.0));
        assert_eq!("68.00s", &fmt_time(68_000_000_000.0));
    }

    #[test]
    fn formats_number() {
        assert_eq!("5.1", &fmt_num(5.1));
        assert_eq!("35.0 thousand", &fmt_num(35_000.0));
        assert_eq!("97.0M", &fmt_num(97_000_000.0));
        assert_eq!("7.9B", &fmt_num(7_900_000_000.0));
    }

    #[test]
    fn formats_change() {
        assert_eq!("5.1973%", &fmt_change(5.1973));
    }
}
