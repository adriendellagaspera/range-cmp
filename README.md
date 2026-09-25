# range-cmp

[![Crates.io][crates-badge]][crates-url]
[![Build Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/range-cmp.svg
[crates-url]: https://crates.io/crates/range-cmp
[actions-badge]: https://github.com/adriendellagaspera/range-cmp/actions/workflows/ci.yml/badge.svg
[actions-url]: https://github.com/adriendellagaspera/range-cmp/actions/workflows/ci.yml

[Docs](https://docs.rs/range-cmp/latest/range_cmp/)

Compare a value with a range using `RangeOrd::rcmp`:

```rust
use range_cmp::{RangeOrd, RangeOrdering};
assert_eq!(15.rcmp(20..30), RangeOrdering::Below);
assert_eq!(25.rcmp(20..30), RangeOrdering::Inside);
assert_eq!(35.rcmp(20..30), RangeOrdering::Above);
```

Empty ranges return `RangeOrdering::Empty`. For partial orders, use
`PartialRangeOrd::partial_rcmp`, which preserves the relationship to each bound.
The [API documentation](https://docs.rs/range-cmp/latest/range_cmp/) defines
these semantics and contains examples.

The crate supports `no_std`, has no dependencies, and requires Rust 1.35 or later.
It is licensed under MIT OR Apache-2.0 (see `LICENSE-MIT` and `LICENSE-APACHE`).
