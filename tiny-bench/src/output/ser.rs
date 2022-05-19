#[cfg(feature = "bench")]
use crate::benching::SamplingData;
use crate::error::{Error, Result};
#[cfg(feature = "timer")]
use crate::timing::TimingData;

/// We'll just turn it into bytes for now, nano-format is a pain to eyeball anyways
#[cfg(feature = "timer")]
pub(crate) fn ser_timing_data(run_data: TimingData) -> Vec<u8> {
    let mut v = Vec::with_capacity(16 * 4);
    v.extend_from_slice(&run_data.min_nanos.to_le_bytes());
    v.extend_from_slice(&run_data.max_nanos.to_le_bytes());
    v.extend_from_slice(&run_data.elapsed.to_le_bytes());
    v.extend_from_slice(&run_data.iterations.to_le_bytes());
    v
}

#[cfg(feature = "timer")]
pub(crate) fn try_de_timing_data(buf: &[u8]) -> Result<TimingData> {
    if buf.len() != 64 {
        return Err(Error::new(format!(
            "Unexpected buffer len for serialized timing data, expected 64 but got {}",
            buf.len()
        )));
    }
    // Since the buffer length is fine we're good here.
    let min_nanos = u128::from_le_bytes(buf[0..16].try_into().ok().unwrap());
    let max_nanos = u128::from_le_bytes(buf[16..32].try_into().ok().unwrap());
    let elapsed = u128::from_le_bytes(buf[32..48].try_into().ok().unwrap());
    let iterations = u128::from_le_bytes(buf[48..64].try_into().ok().unwrap());
    Ok(TimingData {
        min_nanos,
        max_nanos,
        elapsed,
        iterations,
    })
}

#[cfg(feature = "bench")]
pub(crate) fn ser_sampling_data(sampling_data: &SamplingData) -> Vec<u8> {
    let mut v = Vec::new();
    let len = sampling_data.samples.len() as u64;
    v.extend_from_slice(&len.to_le_bytes());
    for sample in &sampling_data.samples {
        v.extend_from_slice(&sample.to_le_bytes());
    }
    for time in &sampling_data.times {
        v.extend_from_slice(&time.to_le_bytes());
    }
    v
}

#[cfg(feature = "bench")]
pub(crate) fn try_de_sampling_data(buf: &[u8]) -> Result<SamplingData> {
    let buf_len = buf.len();
    if buf_len < 8 {
        return Err(Error::new(format!(
            "Found malformed serialized data, length too short {buf_len}"
        )));
    }
    // No risk of going out of bounds yet.
    let len = u64::from_le_bytes(buf[..8].try_into().unwrap());
    let mut samples = Vec::with_capacity(len as usize);
    let mut times = Vec::with_capacity(len as usize);
    let expected_total_len = 8 + len * 16 + len * 8;
    if buf_len as u64 != expected_total_len {
        return Err(Error::new(format!("Found malformed serialized data, unexpected length. Expected {expected_total_len} found {buf_len}")));
    }
    for i in 0..len {
        let sample_value_offset = (8 + i * 8) as usize;
        samples.push(u64::from_le_bytes(
            buf[sample_value_offset..sample_value_offset + 8]
                .try_into()
                .ok()
                .unwrap(),
        ));
        let times_value_offset = (8 + len * 8 + i * 16) as usize;
        times.push(u128::from_le_bytes(
            buf[times_value_offset..times_value_offset + 16]
                .try_into()
                .ok()
                .unwrap(),
        ));
    }
    Ok(SamplingData { samples, times })
}

#[cfg(test)]
mod tests {

    #[test]
    #[cfg(feature = "timer")]
    fn can_ser_de_timing() {
        let min_nanos = 0;
        let max_nanos = u128::MAX;
        let elapsed = 555_555;
        let iterations = 99_959_599_959;
        let rd = super::TimingData {
            min_nanos,
            max_nanos,
            elapsed,
            iterations,
        };
        assert_eq!(
            rd,
            super::try_de_timing_data(&super::ser_timing_data(rd)).unwrap()
        );
    }

    #[test]
    #[cfg(feature = "bench")]
    fn can_ser_de_sampling() {
        let sampling = super::SamplingData {
            samples: vec![5, 6, 7, 8, 9, 10],
            times: vec![15, 16, 17, 18, 19, 20],
        };
        assert_eq!(
            sampling,
            super::try_de_sampling_data(&super::ser_sampling_data(&sampling)).unwrap()
        );
    }
}
