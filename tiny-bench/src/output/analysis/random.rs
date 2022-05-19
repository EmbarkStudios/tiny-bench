use std::time::{SystemTime, UNIX_EPOCH};

/// [LCG](https://en.wikipedia.org/wiki/Linear_congruential_generator)
/// Choosing same constants as glibc here
const MOD: u128 = 2u128.pow(48);
const A: u128 = 25_214_903_917;
const C: u128 = 11;

pub(crate) struct Rng {
    seed: u64,
}

impl Rng {
    pub(crate) fn new() -> Self {
        // TODO: Find something less stupid
        let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        Rng {
            // And maybe check for overflows
            seed: seed.as_nanos() as u64,
        }
    }

    pub(crate) fn next(&mut self) -> u64 {
        self.seed = ((A * u128::from(self.seed) + C) % MOD) as u64;
        self.seed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::Entry;
    use std::collections::HashMap;

    #[test]
    fn test_lcg() {
        let mut rng = Rng::new();
        let mut distr = HashMap::new();
        let test = 10_000;
        let range = 10;
        for _ in 0..test {
            let v = rng.next() % range;
            match distr.entry(v) {
                Entry::Vacant(v) => {
                    v.insert(1);
                }
                Entry::Occupied(mut o) => {
                    *o.get_mut() += 1;
                }
            }
        }
        eprintln!("{distr:?}");
    }
}
