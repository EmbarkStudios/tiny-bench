#[cfg(feature = "bench")]
use crate::benching::SamplingData;
use crate::error::{Error, Result};
use crate::output::{wrap_high_insensity_red, wrap_yellow};
#[cfg(feature = "timer")]
use crate::timing::TimingData;
use std::ffi::OsStr;
use std::io::ErrorKind;
use std::path::PathBuf;

#[cfg(feature = "timer")]
const CURRENT_RESULTS: &str = "current-results";
#[cfg(feature = "timer")]
const OLD_RESULTS: &str = "old-results";

#[cfg(feature = "bench")]
const CURRENT_SAMPLE: &str = "current-sample";

#[cfg(feature = "bench")]
const OLD_SAMPLE: &str = "old-sample";

#[cfg(feature = "timer")]
pub(crate) fn try_read_last_results(label: &'static str) -> Result<Option<TimingData>> {
    let maybe_data = try_read(label, CURRENT_RESULTS)?;
    if let Some(data) = maybe_data {
        Ok(Some(crate::output::ser::try_de_timing_data(&data)?))
    } else {
        Ok(None)
    }
}

#[cfg(feature = "timer")]
pub(crate) fn try_write_results(label: &'static str, data: TimingData) {
    if let Err(e) = try_write(
        label,
        &crate::output::ser::ser_timing_data(data),
        CURRENT_RESULTS,
        OLD_RESULTS,
    ) {
        println!(
            "{} {e}",
            wrap_high_insensity_red("Failed to write timing data, cause")
        );
    }
}

#[cfg(feature = "bench")]
pub(crate) fn try_write_last_simpling(label: &'static str, data: &SamplingData) {
    if let Err(e) = try_write(
        label,
        &crate::output::ser::ser_sampling_data(data),
        CURRENT_SAMPLE,
        OLD_SAMPLE,
    ) {
        println!(
            "{} {e}",
            wrap_high_insensity_red("Failed to write sampling data, cause:")
        );
    }
}

fn try_write(
    label: &'static str,
    data: &[u8],
    current_file_name: &str,
    old_file_name: &'static str,
) -> Result<()> {
    if label.contains(std::path::is_separator) {
        return Err(Error::new(format!(
            "Label {label} contains a path separator, cannot write to disk."
        )));
    }
    let parent_dir = find_or_create_result_parent_dir(label)?;
    std::fs::create_dir_all(&parent_dir).map_err(|e| {
        Error::new(format!(
            "Failed to create output directory {:?}, cause {e}, will not write results",
            parent_dir
        ))
    })?;

    let latest_persisted = parent_dir.join(current_file_name);
    if std::fs::metadata(&latest_persisted).is_ok() {
        let old_file = parent_dir.join(old_file_name);
        if let Err(e) = std::fs::rename(&latest_persisted, &old_file) {
            println!(
                "{} from {:?} to {:?}, cause {e}, will try to overwrite.",
                wrap_yellow("Failed to move old sample"),
                latest_persisted,
                old_file
            );
        }
    }
    std::fs::write(&latest_persisted, data).map_err(|e| {
        Error::new(format!(
            "Failed to write benchmark-data to {:?}, cause {e}",
            latest_persisted
        ))
    })
}

fn try_read(label: &'static str, current_file_name: &'static str) -> Result<Option<Vec<u8>>> {
    if label.contains(std::path::is_separator) {
        return Err(Error::new(format!(
            "Label {label} contains a path separator, cannot read old data from disk."
        )));
    }
    let parent_dir = find_or_create_result_parent_dir(label)?;
    let latest_persisted_path = parent_dir.join(current_file_name);
    match std::fs::read(&latest_persisted_path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(e) => match e.kind() {
            ErrorKind::NotFound => Ok(None),
            _ => Err(Error::new(format!(
                "Failed to read file at {:?}, cause: {e}",
                latest_persisted_path
            ))),
        },
    }
}

#[cfg(feature = "bench")]
pub(crate) fn try_read_last_simpling(label: &'static str) -> Result<Option<SamplingData>> {
    let maybe_data = try_read(label, CURRENT_SAMPLE)?;
    if let Some(data) = maybe_data {
        Ok(Some(crate::output::ser::try_de_sampling_data(&data)?))
    } else {
        Ok(None)
    }
}

fn find_or_create_result_parent_dir(label: &'static str) -> Result<PathBuf> {
    let target = find_target()?;
    let pb = PathBuf::from(&target);
    let target_buf = std::fs::metadata(&pb).map_err(|e| {
        Error::new(format!(
            "Failed to check metadata for target dir {:?}, cause {e}",
            target
        ))
    })?;
    if !target_buf.is_dir() {
        return Err(Error::new(format!(
            "Expected target directory {pb:?} is not a directory"
        )));
    }
    let all_results_dir = pb.join("simple-bench");

    let result_parent_dir = all_results_dir.join(label);

    std::fs::create_dir_all(&result_parent_dir).map_err(|e| {
        Error::new(format!(
            "Failed to create output directory {:?}, cause {e}",
            result_parent_dir
        ))
    })?;
    Ok(result_parent_dir)
}

fn find_target() -> Result<PathBuf> {
    let exe = std::env::current_exe().map_err(|e| {
        Error::new(format!(
            "Failed to get this executable's directory from environment, cause {e}"
        ))
    })?;
    let mut cur = exe.as_path();
    let target_os_str = OsStr::new("target");
    while let Some(parent) = cur.parent() {
        let last = parent
            .components()
            .last()
            .ok_or_else(|| Error::new("Could not find target directory to place output"))?;
        if last.as_os_str() == target_os_str {
            return Ok(parent.to_path_buf());
        }
        cur = parent;
    }
    Err(Error::new(
        "Could not find target directory to place output",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "timer")]
    fn can_dump_and_read_results() {
        let label = "label";
        let rd1 = TimingData {
            min_nanos: 0,
            max_nanos: 5,
            elapsed: 10,
            iterations: 15,
        };
        try_write_results(label, rd1);
        assert_eq!(rd1, try_read_last_results(label).unwrap().unwrap());
        let rd2 = TimingData {
            min_nanos: 100,
            max_nanos: 105,
            elapsed: 110,
            iterations: 115,
        };
        try_write_results(label, rd2);
        assert_eq!(rd2, try_read_last_results(label).unwrap().unwrap());
    }

    #[test]
    #[cfg(feature = "bench")]
    fn can_dump_and_read_samples() {
        let label = "label";
        let s1 = SamplingData {
            samples: vec![1, 2, 3, 4, 5],
            times: vec![6, 7, 8, 9, 10],
        };
        try_write_last_simpling(label, &s1);
        assert_eq!(s1, try_read_last_simpling(label).unwrap().unwrap());
        let s2 = SamplingData {
            samples: vec![5, 4, 3, 2, 1],
            times: vec![10, 9, 8, 7, 6],
        };
        try_write_last_simpling(label, &s2);
        assert_eq!(s2, try_read_last_simpling(label).unwrap().unwrap());
    }
}
