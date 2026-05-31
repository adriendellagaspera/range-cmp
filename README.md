# range_cmp

[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]
[![Apache licensed][apache-badge]][apache-url]
[![Build Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/range-cmp.svg
[crates-url]: https://crates.io/crates/range-cmp
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/Akvize/range-cmp/blob/master/LICENSE-MIT
[apache-badge]: https://img.shields.io/badge/license-APACHE-blue.svg
[apache-url]: https://github.com/Akvize/range-cmp/blob/master/LICENSE-APACHE
[actions-badge]: https://github.com/Akvize/range-cmp/actions/workflows/ci.yml/badge.svg
[actions-url]: https://github.com/Akvize/range-cmp/actions/workflows/ci.yml

[Docs](https://docs.rs/range-cmp/latest/range_cmp/)

This Rust crate provides the `RangeOrd` trait on all types that
implement `Ord`. This trait exposes a `rcmp` associated method
that allows comparing a value with a range of values:

```rust
use range_cmp::{RangeOrd, RangeOrdering};
assert_eq!(15.rcmp(20..30), RangeOrdering::Below);
assert_eq!(25.rcmp(20..30), RangeOrdering::Inside);
assert_eq!(35.rcmp(20..30), RangeOrdering::Above);
```

## Empty ranges handling

Empty ranges are handled explicitly, instead of returning an arbitrary,
representation-dependent answer. An empty range is reported as
`RangeOrdering::Empty`:

```rust
use range_cmp::{RangeOrd, RangeOrdering};
assert_eq!(30.rcmp(45..35), RangeOrdering::Empty);
assert_eq!(30.rcmp(25..15), RangeOrdering::Empty);
assert_eq!(0.rcmp(0..0), RangeOrdering::Empty);
```

Emptiness is judged from the *bounds*, not from the population of the type:
`..0u32` is treated as a regular (non-empty) range even though no `u32` is
below `0`.

## Partial orders

The crate also provides the `PartialRangeOrd` trait on all types that
implement `PartialOrd`. Because a partial order is a poset rather than a
line, a value may be incomparable with one or both bounds, so a single
`Below`/`Inside`/`Above` verdict is not always meaningful. `partial_rcmp`
therefore returns a `RangePosition`: the *pair* of the value's relationships
to the lower and upper bounds, which never loses information. Use
`RangePosition::ordering` to collapse it into a simple `RangeOrdering` when
the value is comparable with both bounds.

```rust
use range_cmp::{PartialRangeOrd, RangeOrdering};
assert_eq!(1.5_f64.partial_rcmp(2.0..3.0).ordering(), Some(RangeOrdering::Below));
assert_eq!(2.5_f64.partial_rcmp(2.0..3.0).ordering(), Some(RangeOrdering::Inside));
assert_eq!(3.5_f64.partial_rcmp(2.0..3.0).ordering(), Some(RangeOrdering::Above));
// `NaN` is incomparable with the bounds:
assert_eq!(f64::NAN.partial_rcmp(2.0..3.0).ordering(), None);
```
