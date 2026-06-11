# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- The crate is now `#![no_std]`. It has no dependencies and relies only on `core`,
  so it can be used in embedded and other environments without the standard library.
- Cargo manifest metadata for crates.io: `keywords`, `categories`, `rust-version`
  (MSRV 1.56), and `readme`.

## [0.3.0]

### Changed

- **Breaking:** reworked the comparison semantics to handle empty ranges explicitly
  and to be honest about partial orders.
- **Breaking:** renamed the `range_cmp` method to `rcmp` and the `RangeComparable`
  trait to `RangeOrd`.

### Added

- `RangeOrdering::Empty`: empty and inverted ranges (such as `45..35` or `0..0`) are
  now reported explicitly instead of returning an arbitrary, representation-dependent
  verdict. Emptiness is judged from the bounds (so `..0u32` is non-empty). Resolves
  [#6](https://github.com/Akvize/range-cmp/issues/6).
- `PartialRangeOrd` trait with a `partial_rcmp` method for types that are `PartialOrd`
  but not `Ord`.
- `RangePosition` (the pair of a value's relationships to the lower and upper bounds,
  with `is_inside` and `ordering` helpers) and `BoundOrdering`
  (`Within` / `Outside` / `Incomparable`), which preserve per-bound information over
  partial orders.

## [0.2.0]

### Changed

- **Breaking:** comparison now first checks whether the range contains the value.

## [0.1.3]

Initial published releases (`0.1.0` through `0.1.3`).

[Unreleased]: https://github.com/Akvize/range-cmp/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/Akvize/range-cmp/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/Akvize/range-cmp/compare/v0.1.3...v0.2.0
[0.1.3]: https://github.com/Akvize/range-cmp/releases/tag/v0.1.3
