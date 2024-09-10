<!-- markdownlint-disable blanks-around-headings blanks-around-lists no-duplicate-heading -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## [Unreleased] - ReleaseDate
### Added

### Changed

### Fixed

## [0.4.0] - 2024-09-10
### Added

### Changed
- Dont use mod_modules anymore
- Reexport `std::hint::black_box()` instead of using criterion's custom
implementation, now that that's stabilized.  

### Fixed
- Proper feature gating on benchmark [PR](https://github.com/EmbarkStudios/tiny-bench/pull/8)
- Update Embark lints
- Cap iterations at 5 billion
- Precompute, chunk, and batch setup values to reduce measurement error

## [0.3.0] - 2023-06-12

### Added

- `max_iterations` overriding argument to set a hard maximum of iterations.

### Fixed

- Update for new lints and a reformat pass.  

## [0.2.0] - 2022-10-07

### Added

- Sample median, variance, and stddev displayed

### Fixed

- Now respects `num_resamples` in effect changing the default from `100` to `100_000` which it should have been

## [0.1.1] - 2022-08-19

### Fixed

- Fix results display showing max instead of min; thanks @lqd!

## [0.1.0] - 2022-06-10

### Added

- A tiny benchmarker
- A tiny timer

<!-- next-url -->

[Unreleased]: https://github.com/EmbarkStudios/tiny-bench/compare/0.1.1...HEAD

[0.1.1]: https://github.com/EmbarkStudios/tiny-bench/compare/0.1.0...0.1.1

[0.1.0]: https://github.com/EmbarkStudios/tiny-bench/releases/tag/0.1.0
