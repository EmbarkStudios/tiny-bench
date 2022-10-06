<!-- Allow this file to not have a first line heading -->
<!-- markdownlint-disable-file MD041 no-emphasis-as-heading -->

<!-- inline html -->
<!-- markdownlint-disable-file MD033 -->

<div align="center">

# `🛠 tiny-bench`

**A tiny benchmarking library**

[![Embark](https://img.shields.io/badge/embark-open%20source-blueviolet.svg)](https://embark.dev)
[![Embark](https://img.shields.io/badge/discord-ark-%237289da.svg?logo=discord)](https://discord.gg/dAuKfZS)
[![Crates.io](https://img.shields.io/crates/v/tiny-bench.svg)](https://crates.io/crates/tiny-bench)
[![Docs](https://docs.rs/tiny-bench/badge.svg)](https://docs.rs/tiny-bench)
[![dependency status](https://deps.rs/repo/github/EmbarkStudios/tiny-bench/status.svg)](https://deps.rs/repo/github/EmbarkStudios/tiny-bench)
[![Build status](https://github.com/EmbarkStudios/tiny-bench/workflows/CI/badge.svg)](https://github.com/EmbarkStudios/tiny-bench/actions)
</div>

## The library

A benchmarking and timing library inspired by [Criterion](https://github.com/bheisler/criterion.rs).  
Inspired in this case means copying the things that criterion does well (and I do mean ctrl-c), like
statistical analysis of results, trimming that down, and leaving much of the customizability out.  
[Criterion](https://github.com/bheisler/criterion.rs) is MIT licensed, please see the license at that repo or
[here](tiny-bench/src/benching/criterion/CRITERION-LICENSE-MIT).

## Primary goals

* Reliable results
* Fast build
* No dependencies
* Simple code that anyone can read, understand, and modify

## Purpose

Sometimes you just need some back-of-the-envelope calculations of how long something takes.
This library aims to fulfill that need and not much else.

The aim of the benchmarking is to be accurate enough to deliver reliable benchmarks with a minimal footprint,
so that you can easily get a sense of whether you're going down a bad path.

The aim of the timing is to provide something that will let you figure out the same with
the caveat of not being as reliable. It times some code so that you can get a sense of
how much time pieces of your code takes to run.

## Caveats

This library does not aim to provide production grade analysis tooling. It just prints data to stdout to guide you.
If you need advanced analysis [Criterion](https://github.com/bheisler/criterion.rs) has tooling better suited to that.  
If you need to find where your application spends its time [flamegraph](https://github.com/flamegraph-rs/flamegraph)
may be better suited for that.  
If you need to track single pieces of your application when it's running [Tracing](https://github.com/tokio-rs/tracing)
may be better suited for that.  
Lastly, if you want an even smaller benchmarking library, check
out [benchmark-simple](https://github.com/jedisct1/rust-benchmark-simple).

## Unimplemented

There are a few statistical measures that would be nice to have but are limited by the methods used by this library.  
Since it potentially runs billions of iterations, calculating statistics based on seeing all iterations such as median,
standard deviation, and percentiles are not feasible without caching data to disk.
Therefore, measures like variance, or median are prefixed by "sample" as they are not related to individual iteration times, 
but a comparison between samples.

There is no arg-parsing or bench-matching in this library, so you can't run cargo bench <test-matcher>.
Instead, the user needs to put different benches into functions, and add/remove those functions from bench main.
The reason for this is that those libraries are heavy-weight and would likely require some macros to
select which benches to run which decreases readability and understandability.

## Examples

### Getting a hint of what parts of your application take time

"I have this iterator, and I'd like to get some sense of how long it takes to complete"

```Rust
use std::time::Duration;
use tiny_bench::Timeable;

pub fn main() {
    let v = (0..100)
        .map(|a| {
            my_expensive_call();
            a
        })
        .timed()
        .max();
    assert_eq!(99, v.unwrap())
    // prints:
    // anonymous [100.0 iterations in 512.25ms]:
    // elapsed	[min mean max]:	[5.06ms 5.12ms 5.20ms]
}

fn my_expensive_call() {
    std::thread::sleep(Duration::from_millis(5));
}
```

"I have this loop that has side effects, and I'd like to time its execution"

```Rust
use tiny_bench::run_timed_from_iterator;

fn main() {
    let generator = 0..100;
    let mut spooky_calculation = 0;
    let results = run_timed_from_iterator(generator, |i| {
        spooky_calculation += i;
    });
    results.pretty_print();
    assert_eq!(4950, spooky_calculation);
}
```

### More involved comparisons

"My algorithm is pretty stupid, but I'm only sorting vectors with a max-length of 5, so maybe it doesn't matter
in the grand scheme of things"

```Rust
use tiny_bench::BenchmarkConfig;

fn main() {
    let v = vec![10, 5, 3, 8, 7, 5];
    tiny_bench::run_bench(&BenchmarkConfig::default(), || {
        let sorted = bad_sort(v.clone());
        assert_eq!(vec![3, 5, 5, 7, 8, 10], sorted);
    })
    // Prints:
    // anonymous [2.5M iterations in 4.99s with 100.0 samples]:
    // elapsed	[min mean max]:	[2.14µs 2.01µs 2.14µs]
}

fn bad_sort(mut v: Vec<u32>) -> Vec<u32> {
    let mut sorted = Vec::with_capacity(v.len());
    while !v.is_empty() {
        let mut min_val = u32::MAX;
        let mut min_index = 0;
        for i in 0..v.len() {
            if v[i] < min_val {
                min_index = i;
                min_val = v[i];
            }
        }
        sorted.push(min_val);
        v.remove(min_index);
    }
    sorted
}
```

"I'd like to compare different implementations with each other"

```Rust
use tiny_bench::black_box;

fn main() {
    // Results are compared by label
    let label = "compare_functions";
    tiny_bench::bench_labeled(label, my_slow_function);
    tiny_bench::bench_labeled(label, my_faster_function);
    // prints:
    //compare_functions [30.3 thousand iterations in 5.24s with 100.0 samples]:
    //elapsed	[min mean max]:	[246.33µs 175.51µs 246.33µs]
    //compare_functions [60.6 thousand iterations in 5.24s with 100.0 samples]:
    //elapsed	[min mean max]:	[87.67µs 86.42µs 87.67µs]
    //change	[min mean max]:	[-49.6111% -50.7620% -64.4102%] (p = 0.00)
}

fn my_slow_function() {
    let mut num_iters = 0;
    for _ in 0..10_000 {
        num_iters += black_box(1);
    }
    assert_eq!(10_000, black_box(num_iters))
}

fn my_faster_function() {
    let mut num_iters = 0;
    for _ in 0..5_000 {
        num_iters += black_box(1);
    }
    assert_eq!(5_000, black_box(num_iters))
}

```

## Contribution

[![Contributor Covenant](https://img.shields.io/badge/contributor%20covenant-v1.4-ff69b4.svg)](CODE_OF_CONDUCT.md)

We welcome community contributions to this project.

Please read our [Contributor Guide](CONTRIBUTING.md) for more information on how to get started.
Please also read our [Contributor Terms](CONTRIBUTING.md#contributor-terms) before you make any contributions.

Any contribution intentionally submitted for inclusion in an Embark Studios project, shall comply with the Rust standard
licensing model (MIT OR Apache 2.0) and therefore be dual licensed as described below, without any additional terms or
conditions:

### License

This contribution is dual licensed under EITHER OF

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

For clarity, "your" refers to Embark or any other licensee/user of the contribution.
