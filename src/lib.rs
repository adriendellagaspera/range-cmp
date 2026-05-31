// Copyright 2023 Developers of the range_cmp project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This crate provides the [`RangeOrd`] trait on all types that implement [`Ord`].
//! This trait exposes a [`rcmp`](RangeOrd::rcmp) associated method that allows
//! comparing a value with a range of values:
//!
//! ```
//! use range_cmp::{RangeOrd, RangeOrdering};
//! assert_eq!(15.rcmp(20..30), RangeOrdering::Below);
//! assert_eq!(25.rcmp(20..30), RangeOrdering::Inside);
//! assert_eq!(35.rcmp(20..30), RangeOrdering::Above);
//! ```
//!
//! # Empty ranges
//!
//! Unlike previous versions, the crate now handles empty ranges explicitly, instead
//! of returning an arbitrary, representation-dependent answer. An empty range (such as
//! `30..20` or `0..0`) is reported as [`RangeOrdering::Empty`]:
//!
//! ```
//! use range_cmp::{RangeOrd, RangeOrdering};
//! assert_eq!(25.rcmp(30..20), RangeOrdering::Empty);
//! assert_eq!(0.rcmp(0..0), RangeOrdering::Empty);
//! ```
//!
//! Emptiness is judged from the *bounds*, not from the population of the type: `..0u32`
//! is treated as a regular (non-empty) range even though no `u32` is below `0`.
//!
//! # Partial orders
//!
//! The crate also provides the [`PartialRangeOrd`] trait on all types that implement
//! [`PartialOrd`]. Because a partial order is not a line but a poset, a value cannot
//! always be collapsed into a single `Below`/`Inside`/`Above` verdict: it may be
//! incomparable with one or both bounds. [`partial_rcmp`](PartialRangeOrd::partial_rcmp)
//! therefore returns a [`RangePosition`], the *pair* of the value's relationships to the
//! lower and upper bounds, which never loses information:
//!
//! ```
//! use range_cmp::{PartialRangeOrd, RangeOrdering};
//! // `f64` is `PartialOrd` but not `Ord`.
//! assert_eq!(1.5_f64.partial_rcmp(2.0..3.0).ordering(), Some(RangeOrdering::Below));
//! assert_eq!(2.5_f64.partial_rcmp(2.0..3.0).ordering(), Some(RangeOrdering::Inside));
//! assert_eq!(3.5_f64.partial_rcmp(2.0..3.0).ordering(), Some(RangeOrdering::Above));
//! // `NaN` is incomparable with the bounds, so there is no single verdict:
//! assert_eq!(f64::NAN.partial_rcmp(2.0..3.0).ordering(), None);
//! ```

use std::borrow::Borrow;
use std::cmp::Ordering;
use std::ops::{Bound, RangeBounds};

/// Simplified result for [`RangeOrd::rcmp`], obtained for totally ordered types or by
/// collapsing a [`RangePosition`] through [`RangePosition::ordering`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RangeOrdering {
    /// The value is below (all) the range. For instance, `-1` is below the range `0..42`.
    Below,
    /// The value is contained inside the range. For instance, `34` is inside the range `0..42`.
    Inside,
    /// The value is above (all) the range. For instance, `314` is above the range `0..42`.
    Above,
    /// The range is empty, so the value cannot be meaningfully positioned. For instance,
    /// `42..0` is empty.
    Empty,
}

/// Position of a value relative to a single bound of a range.
///
/// This is the building block of [`RangePosition`]. For a lower bound, `Within` means the
/// value satisfies the bound (it is greater than, or equal to, the bound depending on
/// inclusiveness) and `Outside` means it is below it. For an upper bound, the meaning is
/// mirrored.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum BoundOrdering {
    /// The value is on the inner side of the bound (it satisfies the bound).
    Within,
    /// The value is on the outer side of the bound (it violates the bound).
    Outside,
    /// The value is incomparable with the bound. This can only happen for types that are
    /// [`PartialOrd`] but not [`Ord`].
    Incomparable,
}

/// Full position of a value relative to a range, expressed as the pair of its
/// relationships to the lower and the upper bound.
///
/// Keeping both relationships separate is what allows [`PartialRangeOrd`] to stay honest
/// over partial orders: a value can be, say, comparable with the lower bound and
/// incomparable with the upper one, and the information is preserved instead of being
/// flattened into a single ambiguous verdict.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RangePosition {
    /// Relationship of the value to the lower bound. `Within` means the value satisfies
    /// the lower bound, `Outside` means it is below it.
    pub lower: BoundOrdering,
    /// Relationship of the value to the upper bound. `Within` means the value satisfies
    /// the upper bound, `Outside` means it is above it.
    pub upper: BoundOrdering,
}

impl RangePosition {
    /// Returns whether the value lies inside the range, i.e. it satisfies both bounds.
    ///
    /// ```
    /// use range_cmp::PartialRangeOrd;
    /// assert!(2.5_f64.partial_rcmp(2.0..3.0).is_inside());
    /// assert!(!3.5_f64.partial_rcmp(2.0..3.0).is_inside());
    /// ```
    pub fn is_inside(&self) -> bool {
        matches!(
            (self.lower, self.upper),
            (BoundOrdering::Within, BoundOrdering::Within)
        )
    }

    /// Collapses the pair into a simple [`RangeOrdering`] when possible.
    ///
    /// Returns `None` when the value is incomparable with at least one bound, in which
    /// case no single `Below`/`Inside`/`Above`/`Empty` verdict captures the position;
    /// inspect the [`lower`](RangePosition::lower) and [`upper`](RangePosition::upper)
    /// fields directly in that case.
    ///
    /// The `(Outside, Outside)` case — being simultaneously below the lower bound and
    /// above the upper bound — can only occur for an empty (inverted) range, and is
    /// reported as [`RangeOrdering::Empty`].
    pub fn ordering(&self) -> Option<RangeOrdering> {
        match (self.lower, self.upper) {
            (BoundOrdering::Within, BoundOrdering::Within) => Some(RangeOrdering::Inside),
            (BoundOrdering::Outside, BoundOrdering::Within) => Some(RangeOrdering::Below),
            (BoundOrdering::Within, BoundOrdering::Outside) => Some(RangeOrdering::Above),
            (BoundOrdering::Outside, BoundOrdering::Outside) => Some(RangeOrdering::Empty),
            _ => None,
        }
    }
}

/// Computes the relationship of `value` to a lower bound.
fn lower_ordering<T: PartialOrd>(value: &T, bound: Bound<&T>) -> BoundOrdering {
    match bound {
        Bound::Unbounded => BoundOrdering::Within,
        Bound::Included(key) => match value.partial_cmp(key) {
            Some(Ordering::Less) => BoundOrdering::Outside,
            Some(Ordering::Equal | Ordering::Greater) => BoundOrdering::Within,
            None => BoundOrdering::Incomparable,
        },
        Bound::Excluded(key) => match value.partial_cmp(key) {
            Some(Ordering::Less | Ordering::Equal) => BoundOrdering::Outside,
            Some(Ordering::Greater) => BoundOrdering::Within,
            None => BoundOrdering::Incomparable,
        },
    }
}

/// Computes the relationship of `value` to an upper bound.
fn upper_ordering<T: PartialOrd>(value: &T, bound: Bound<&T>) -> BoundOrdering {
    match bound {
        Bound::Unbounded => BoundOrdering::Within,
        Bound::Included(key) => match value.partial_cmp(key) {
            Some(Ordering::Greater) => BoundOrdering::Outside,
            Some(Ordering::Equal | Ordering::Less) => BoundOrdering::Within,
            None => BoundOrdering::Incomparable,
        },
        Bound::Excluded(key) => match value.partial_cmp(key) {
            Some(Ordering::Greater | Ordering::Equal) => BoundOrdering::Outside,
            Some(Ordering::Less) => BoundOrdering::Within,
            None => BoundOrdering::Incomparable,
        },
    }
}

/// Builds the [`RangePosition`] of `value` relative to `range`.
fn position_in<T: PartialOrd, R: RangeBounds<T>>(value: &T, range: &R) -> RangePosition {
    RangePosition {
        lower: lower_ordering(value, range.start_bound()),
        upper: upper_ordering(value, range.end_bound()),
    }
}

/// Returns whether a range is empty, judged solely from its bounds (and thus from the
/// total order over `T`). A range with at least one unbounded side is never empty.
fn range_is_empty<T: Ord, R: RangeBounds<T>>(range: &R) -> bool {
    match (range.start_bound(), range.end_bound()) {
        (Bound::Included(start), Bound::Included(end)) => start > end,
        (Bound::Included(start), Bound::Excluded(end))
        | (Bound::Excluded(start), Bound::Included(end))
        | (Bound::Excluded(start), Bound::Excluded(end)) => start >= end,
        _ => false,
    }
}

// suggestion from @benschulz https://internals.rust-lang.org/t/implement-rangebounds-for-range/19704/3
/// Helper trait to allow passing a range as either a owned value or a reference.
///
/// For instance:
///
/// ```
/// use std::ops::RangeBounds;
/// use range_cmp::BorrowRange;
/// fn f<T, R: RangeBounds<T>, B: BorrowRange<T, R>>(range: B) {
///     let range = range.borrow();
///     // ...
/// }
/// ```
///
/// With concrete type such as [`i32`], this would be achieved by taking a generic type `T` with
/// the bound `T: Borrow<i32>`. So we might be tempted to do the same with the [`RangeBounds`]
/// trait:
///
/// ```
/// use std::borrow::Borrow;
/// use std::ops::RangeBounds;
/// fn f<R: RangeBounds<i32>, B: Borrow<R>>(range: B) {
///     let range = range.borrow();
///     // ...
/// }
/// f(0..42)
/// ```
///
/// However, this fails to compile when passing a reference:
///
/// ```compile_fail,E0282
/// # use std::borrow::Borrow;
/// # use std::ops::RangeBounds;
/// # fn f<R: RangeBounds<i32>, B: Borrow<R>>(range: B) {
/// #     let range = range.borrow();
/// #     // ...
/// # }
/// f(&(0..42))
/// ```
///
/// The compilation output is:
///
/// ```shell
///   | f(&(0..42))
///   | ^ cannot infer type of the type parameter `R` declared on the function `f`
/// ```
///
/// Indeed, although we understand we want to pass a [`Range`](std::ops::Range)`<`[`i32`]`>` by
/// reference, the compiler need to assume that other types could yield a
/// `&`[`Range`](std::ops::Range)`<`[`i32`]`>` when borrowed.
pub trait BorrowRange<T: ?Sized, R>: Borrow<R> {}
impl<T, R: RangeBounds<T>> BorrowRange<T, R> for R {}
impl<T, R: RangeBounds<T>> BorrowRange<T, R> for &R {}

/// Trait to provide the [`rcmp`](RangeOrd::rcmp) method, which allows comparing
/// the type to a range. A blanket implementation is provided for all types that implement the
/// [`Ord`] trait.
pub trait RangeOrd {
    /// Compare the value to a range of values. Returns whether the value is below, inside,
    /// above, or whether the range is empty.
    ///
    /// ```
    /// use range_cmp::{RangeOrd, RangeOrdering};
    /// assert_eq!(15.rcmp(20..30), RangeOrdering::Below);
    /// assert_eq!(25.rcmp(20..30), RangeOrdering::Inside);
    /// assert_eq!(35.rcmp(20..30), RangeOrdering::Above);
    /// assert_eq!(25.rcmp(30..20), RangeOrdering::Empty);
    /// ```
    fn rcmp<R: RangeBounds<Self>, B: BorrowRange<Self, R>>(&self, range: B) -> RangeOrdering;
}

impl<T: Ord> RangeOrd for T {
    fn rcmp<R: RangeBounds<Self>, B: BorrowRange<Self, R>>(&self, range: B) -> RangeOrdering {
        let range = range.borrow();
        if range_is_empty(range) {
            return RangeOrdering::Empty;
        }
        // `Self` is totally ordered, so no bound can be incomparable, and a non-empty
        // range always yields one of `Below`, `Inside` or `Above`.
        position_in(self, range)
            .ordering()
            .expect("a total order over a non-empty range always yields a verdict")
    }
}

/// Trait to provide the [`partial_rcmp`](PartialRangeOrd::partial_rcmp) method, which allows
/// comparing the type to a range. A blanket implementation is provided for all types that
/// implement the [`PartialOrd`] trait.
pub trait PartialRangeOrd {
    /// Compare the value to a range of values, returning its full [`RangePosition`]: the
    /// pair of its relationships to the lower and the upper bound.
    ///
    /// Use [`RangePosition::ordering`] to collapse it into a simple [`RangeOrdering`] when
    /// the value is comparable with both bounds.
    ///
    /// ```
    /// use range_cmp::{PartialRangeOrd, RangeOrdering};
    /// assert_eq!(1.5_f64.partial_rcmp(2.0..3.0).ordering(), Some(RangeOrdering::Below));
    /// assert_eq!(2.5_f64.partial_rcmp(2.0..3.0).ordering(), Some(RangeOrdering::Inside));
    /// assert_eq!(3.5_f64.partial_rcmp(2.0..3.0).ordering(), Some(RangeOrdering::Above));
    /// // `NaN` is incomparable with the bounds:
    /// assert_eq!(f64::NAN.partial_rcmp(2.0..3.0).ordering(), None);
    /// ```
    fn partial_rcmp<R: RangeBounds<Self>, B: BorrowRange<Self, R>>(
        &self,
        range: B,
    ) -> RangePosition;
}

impl<T: PartialOrd> PartialRangeOrd for T {
    fn partial_rcmp<R: RangeBounds<Self>, B: BorrowRange<Self, R>>(
        &self,
        range: B,
    ) -> RangePosition {
        position_in(self, range.borrow())
    }
}

#[cfg(test)]
mod rcmp_tests {
    use super::*;

    #[test]
    fn range_full() {
        // 1 is inside ]-inf, inf[
        assert_eq!(1.rcmp(..), RangeOrdering::Inside);
    }

    #[test]
    fn range_from() {
        // 1 is inside [1, +inf[
        assert_eq!(1.rcmp(1..), RangeOrdering::Inside);
        assert_eq!(1.rcmp(&1..), RangeOrdering::Inside);

        // 1 is below [2, +inf[
        assert_eq!(1.rcmp(2..), RangeOrdering::Below);
        assert_eq!(1.rcmp(&2..), RangeOrdering::Below);
    }

    #[test]
    fn range_to() {
        // 1 is above ]-inf, 1[
        assert_eq!(1.rcmp(..1), RangeOrdering::Above);
        assert_eq!(1.rcmp(..&1), RangeOrdering::Above);

        // 1 is inside ]-inf, 2[
        assert_eq!(1.rcmp(..2), RangeOrdering::Inside);
        assert_eq!(1.rcmp(..&2), RangeOrdering::Inside);
    }

    #[test]
    fn range() {
        // 1 is above [0, 1[
        assert_eq!(1.rcmp(0..1), RangeOrdering::Above);
        assert_eq!(1.rcmp(&0..&1), RangeOrdering::Above);

        // 1 is inside [1, 2[
        assert_eq!(1.rcmp(1..2), RangeOrdering::Inside);
        assert_eq!(1.rcmp(&1..&2), RangeOrdering::Inside);

        // 1 is below [2, 3[
        assert_eq!(1.rcmp(2..3), RangeOrdering::Below);
        assert_eq!(1.rcmp(&2..&3), RangeOrdering::Below);
    }

    #[test]
    fn range_inclusive() {
        // 1 is above [0, 0]
        assert_eq!(1.rcmp(0..=0), RangeOrdering::Above);
        assert_eq!(1.rcmp(&0..=&0), RangeOrdering::Above);

        // 1 is inside [1, 1]
        assert_eq!(1.rcmp(1..=1), RangeOrdering::Inside);
        assert_eq!(1.rcmp(&1..=&1), RangeOrdering::Inside);

        // 1 is below [2, 2]
        assert_eq!(1.rcmp(2..=2), RangeOrdering::Below);
        assert_eq!(1.rcmp(&2..=&2), RangeOrdering::Below);
    }

    #[test]
    fn range_to_inclusive() {
        // 1 is above ]-inf, 0]
        assert_eq!(1.rcmp(..=0), RangeOrdering::Above);
        assert_eq!(1.rcmp(..=&0), RangeOrdering::Above);

        // 1 is inside ]-inf, 1
        assert_eq!(1.rcmp(..=1), RangeOrdering::Inside);
        assert_eq!(1.rcmp(..=&1), RangeOrdering::Inside);
    }

    #[test]
    fn bounds_full() {
        // 1 is inside ]-inf, inf[
        let bounds: (Bound<i32>, Bound<i32>) = (Bound::Unbounded, Bound::Unbounded);
        assert_eq!(1.rcmp(bounds), RangeOrdering::Inside);
    }

    #[test]
    fn bounds_from() {
        // 1 is inside [1, +inf[
        let bounds = (Bound::Included(1), Bound::Unbounded);
        assert_eq!(1.rcmp(bounds), RangeOrdering::Inside);

        let bounds = (Bound::Included(&1), Bound::Unbounded);
        assert_eq!(1.rcmp(bounds), RangeOrdering::Inside);

        // 1 is below [2, +inf[
        let bounds = (Bound::Included(2), Bound::Unbounded);
        assert_eq!(1.rcmp(bounds), RangeOrdering::Below);

        let bounds = (Bound::Included(&2), Bound::Unbounded);
        assert_eq!(1.rcmp(bounds), RangeOrdering::Below);
    }

    #[test]
    fn bounds_to() {
        // 1 is above ]-inf, 1[
        let bounds = (Bound::Unbounded, Bound::Excluded(1));
        assert_eq!(1.rcmp(bounds), RangeOrdering::Above);

        let bounds = (Bound::Unbounded, Bound::Excluded(&1));
        assert_eq!(1.rcmp(bounds), RangeOrdering::Above);

        // 1 is inside ]-inf, 2[
        let bounds = (Bound::Unbounded, Bound::Excluded(2));
        assert_eq!(1.rcmp(bounds), RangeOrdering::Inside);

        let bounds = (Bound::Unbounded, Bound::Excluded(&2));
        assert_eq!(1.rcmp(bounds), RangeOrdering::Inside);
    }

    #[test]
    fn bounds() {
        // 1 is above [0, 1[
        let bounds = (Bound::Included(0), Bound::Excluded(1));
        assert_eq!(1.rcmp(bounds), RangeOrdering::Above);

        let bounds = (Bound::Included(&0), Bound::Excluded(&1));
        assert_eq!(1.rcmp(bounds), RangeOrdering::Above);

        // 1 is inside [1, 2[
        let bounds = (Bound::Included(1), Bound::Excluded(2));
        assert_eq!(1.rcmp(bounds), RangeOrdering::Inside);

        let bounds = (Bound::Included(&1), Bound::Excluded(&2));
        assert_eq!(1.rcmp(bounds), RangeOrdering::Inside);

        // 1 is below [2, 3[
        let bounds = (Bound::Included(2), Bound::Excluded(3));
        assert_eq!(1.rcmp(bounds), RangeOrdering::Below);

        let bounds = (Bound::Included(&2), Bound::Excluded(&3));
        assert_eq!(1.rcmp(bounds), RangeOrdering::Below);
    }

    #[test]
    fn bounds_inclusive() {
        // 1 is above [0, 0]
        let bounds = (Bound::Included(0), Bound::Included(0));
        assert_eq!(1.rcmp(bounds), RangeOrdering::Above);

        let bounds = (Bound::Included(&0), Bound::Included(&0));
        assert_eq!(1.rcmp(bounds), RangeOrdering::Above);

        // 1 is inside [1, 1]
        let bounds = (Bound::Included(1), Bound::Included(1));
        assert_eq!(1.rcmp(bounds), RangeOrdering::Inside);

        let bounds = (Bound::Included(&1), Bound::Included(&1));
        assert_eq!(1.rcmp(bounds), RangeOrdering::Inside);

        // 1 is below [2, 2]
        let bounds = (Bound::Included(2), Bound::Included(2));
        assert_eq!(1.rcmp(bounds), RangeOrdering::Below);

        let bounds = (Bound::Included(&2), Bound::Included(&2));
        assert_eq!(1.rcmp(bounds), RangeOrdering::Below);
    }

    #[test]
    fn bounds_to_inclusive() {
        // 1 is above ]-inf, 0]
        let bounds = (Bound::Unbounded, Bound::Included(0));
        assert_eq!(1.rcmp(bounds), RangeOrdering::Above);

        let bounds = (Bound::Unbounded, Bound::Included(&0));
        assert_eq!(1.rcmp(bounds), RangeOrdering::Above);

        // 1 is inside ]-inf, 1]
        let bounds = (Bound::Unbounded, Bound::Included(1));
        assert_eq!(1.rcmp(bounds), RangeOrdering::Inside);

        let bounds = (Bound::Unbounded, Bound::Included(&1));
        assert_eq!(1.rcmp(bounds), RangeOrdering::Inside);
    }

    #[test]
    fn bounds_exclusive_inclusive() {
        // 1 is above ]-1, 0]
        let bounds: (Bound<i32>, Bound<i32>) = (Bound::Excluded(-1), Bound::Included(0));
        assert_eq!(1.rcmp(bounds), RangeOrdering::Above);

        let bounds: (Bound<&i32>, Bound<&i32>) = (Bound::Excluded(&-1), Bound::Included(&0));
        assert_eq!(1.rcmp(bounds), RangeOrdering::Above);

        // 1 is inside ]0, 1]
        let bounds: (Bound<i32>, Bound<i32>) = (Bound::Excluded(0), Bound::Included(1));
        assert_eq!(1.rcmp(bounds), RangeOrdering::Inside);

        let bounds: (Bound<&i32>, Bound<&i32>) = (Bound::Excluded(&0), Bound::Included(&1));
        assert_eq!(1.rcmp(bounds), RangeOrdering::Inside);

        // 1 is below ]1, 2]
        let bounds: (Bound<i32>, Bound<i32>) = (Bound::Excluded(1), Bound::Included(2));
        assert_eq!(1.rcmp(bounds), RangeOrdering::Below);

        let bounds: (Bound<&i32>, Bound<&i32>) = (Bound::Excluded(&1), Bound::Included(&2));
        assert_eq!(1.rcmp(bounds), RangeOrdering::Below);
    }

    #[test]
    fn bounds_as_reference() {
        let bounds = 0..2;
        assert_eq!(1.rcmp(&bounds), RangeOrdering::Inside);
        assert_eq!(1.rcmp(bounds), RangeOrdering::Inside);
    }

    #[test]
    #[allow(clippy::reversed_empty_ranges)] // intentionally testing empty/inverted ranges
    fn empty_ranges() {
        // [0, 0[ is empty
        assert_eq!(0.rcmp(0..0), RangeOrdering::Empty);
        assert_eq!(0.rcmp(&0..&0), RangeOrdering::Empty);

        // ]-inf, 0u32[ is a regular range (emptiness is judged from the bounds, not the
        // population of the type), and 0u32 is above it
        assert_eq!(0.rcmp(..0u32), RangeOrdering::Above);
        assert_eq!(0.rcmp(..&0u32), RangeOrdering::Above);

        // [45, 35[ is empty (inverted)
        assert_eq!(30.rcmp(45..35), RangeOrdering::Empty);
        assert_eq!(30.rcmp(&45..&35), RangeOrdering::Empty);

        // [25, 15[ is empty (inverted)
        assert_eq!(30.rcmp(25..15), RangeOrdering::Empty);
        assert_eq!(30.rcmp(&25..&15), RangeOrdering::Empty);

        // [0, 0] is *not* empty: it contains exactly 0
        assert_eq!(0.rcmp(0..=0), RangeOrdering::Inside);
        assert_eq!(1.rcmp(0..=0), RangeOrdering::Above);
    }
}

#[cfg(test)]
mod partial_rcmp_tests {
    use super::*;

    /// A deliberately partial order: `Div(a)` compares to `Div(b)` through divisibility of
    /// their absolute values. `Div(a) < Div(b)` iff `|a|` strictly divides `|b|`; values
    /// that do not divide one another (e.g. `2` and `3`) are incomparable.
    #[derive(Clone, Copy, Debug, PartialEq)]
    struct Div(i32);

    impl PartialOrd for Div {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            let a_s = self.0.abs();
            let a_o = other.0.abs();

            match a_s.cmp(&a_o) {
                Ordering::Less if a_o % a_s == 0 => Some(Ordering::Less),
                Ordering::Greater if a_s % a_o == 0 => Some(Ordering::Greater),
                Ordering::Equal => Some(Ordering::Equal),
                _ => None,
            }
        }
    }

    // Shorthands for terser assertions.
    const W: BoundOrdering = BoundOrdering::Within;
    const O: BoundOrdering = BoundOrdering::Outside;
    const I: BoundOrdering = BoundOrdering::Incomparable;

    fn pos(lower: BoundOrdering, upper: BoundOrdering) -> RangePosition {
        RangePosition { lower, upper }
    }

    #[test]
    fn range_full() {
        // 1 is an integer, comparable to everything in ]-inf, inf[
        assert_eq!(Div(1).partial_rcmp(..), pos(W, W));
        assert_eq!(
            Div(1).partial_rcmp(..).ordering(),
            Some(RangeOrdering::Inside)
        );
    }

    #[test]
    fn range_from() {
        // 1 is a multiple of 1
        assert_eq!(Div(1).partial_rcmp(Div(1)..), pos(W, W));
        assert_eq!(Div(1).partial_rcmp(&Div(1)..), pos(W, W));

        // 1 is below the multiples of 2
        assert_eq!(Div(1).partial_rcmp(Div(2)..), pos(O, W));
        assert_eq!(Div(1).partial_rcmp(&Div(2)..), pos(O, W));

        // 2 is incomparable with the multiples of 3
        assert_eq!(Div(2).partial_rcmp(Div(3)..), pos(I, W));
        assert_eq!(Div(2).partial_rcmp(&Div(3)..), pos(I, W));
        assert_eq!(Div(2).partial_rcmp(Div(3)..).ordering(), None);
    }

    #[test]
    fn range_to() {
        // 4 is a multiple of all divisors of 2, hence above ]-inf, 2[
        assert_eq!(Div(4).partial_rcmp(..Div(2)), pos(W, O));
        assert_eq!(Div(4).partial_rcmp(..&Div(2)), pos(W, O));

        // 1 is a divisor of 2
        assert_eq!(Div(1).partial_rcmp(..Div(2)), pos(W, W));
        assert_eq!(Div(1).partial_rcmp(..&Div(2)), pos(W, W));

        // 3 is incomparable with the divisors of 10
        assert_eq!(Div(3).partial_rcmp(..Div(10)), pos(W, I));
        assert_eq!(Div(3).partial_rcmp(..&Div(10)), pos(W, I));
        assert_eq!(Div(3).partial_rcmp(..Div(10)).ordering(), None);
    }

    #[test]
    fn range() {
        // 3 is a multiple of all divisors of 3, hence above [1, 3[
        assert_eq!(Div(3).partial_rcmp(Div(1)..Div(3)), pos(W, O));
        assert_eq!(Div(3).partial_rcmp(&Div(1)..&Div(3)), pos(W, O));

        // 6 is a multiple of 2 and a divisor of 12
        assert_eq!(Div(6).partial_rcmp(Div(2)..Div(12)), pos(W, W));
        assert_eq!(Div(6).partial_rcmp(&Div(2)..&Div(12)), pos(W, W));

        // 2 divides all multiples of 4 that divide 8, hence below [4, 8[
        assert_eq!(Div(2).partial_rcmp(Div(4)..Div(8)), pos(O, W));
        assert_eq!(Div(2).partial_rcmp(&Div(4)..&Div(8)), pos(O, W));

        // 3 is incomparable with 4 (lower bound) but divides 12 (within the upper bound)
        assert_eq!(Div(3).partial_rcmp(Div(4)..Div(12)), pos(I, W));
        assert_eq!(Div(3).partial_rcmp(&Div(4)..&Div(12)), pos(I, W));
        assert_eq!(Div(3).partial_rcmp(Div(4)..Div(12)).ordering(), None);
    }

    #[test]
    fn range_inclusive() {
        // 6 is a multiple of all divisors of 3, hence above [1, 3]
        assert_eq!(Div(6).partial_rcmp(Div(1)..=Div(3)), pos(W, O));
        assert_eq!(Div(6).partial_rcmp(&Div(1)..=&Div(3)), pos(W, O));

        // 6 is a multiple of 6 and a divisor of 6
        assert_eq!(Div(6).partial_rcmp(Div(6)..=Div(6)), pos(W, W));
        assert_eq!(Div(6).partial_rcmp(&Div(6)..=&Div(6)), pos(W, W));

        // 2 divides all multiples of 4 that divide 8, hence below [4, 8]
        assert_eq!(Div(2).partial_rcmp(Div(4)..=Div(8)), pos(O, W));
        assert_eq!(Div(2).partial_rcmp(&Div(4)..=&Div(8)), pos(O, W));

        // 3 is incomparable with 4 (lower bound) but divides 12 (within the upper bound)
        assert_eq!(Div(3).partial_rcmp(Div(4)..=Div(12)), pos(I, W));
        assert_eq!(Div(3).partial_rcmp(&Div(4)..=&Div(12)), pos(I, W));
    }

    #[test]
    fn range_to_inclusive() {
        // 4 is a multiple of all divisors of 2, hence above ]-inf, 2]
        assert_eq!(Div(4).partial_rcmp(..=Div(2)), pos(W, O));
        assert_eq!(Div(4).partial_rcmp(..=&Div(2)), pos(W, O));

        // 1 is a divisor of 2
        assert_eq!(Div(1).partial_rcmp(..=Div(2)), pos(W, W));
        assert_eq!(Div(1).partial_rcmp(..=&Div(2)), pos(W, W));

        // 3 is incomparable with the divisors of 10
        assert_eq!(Div(3).partial_rcmp(..=Div(10)), pos(W, I));
        assert_eq!(Div(3).partial_rcmp(..=&Div(10)), pos(W, I));
    }

    #[test]
    fn bounds_full() {
        let bounds: (Bound<Div>, Bound<Div>) = (Bound::Unbounded, Bound::Unbounded);
        assert_eq!(Div(1).partial_rcmp(bounds), pos(W, W));
    }

    #[test]
    fn bounds_from() {
        // 1 is a multiple of 1
        let bounds = (Bound::Included(Div(1)), Bound::Unbounded);
        assert_eq!(Div(1).partial_rcmp(bounds), pos(W, W));

        let bounds = (Bound::Included(&Div(1)), Bound::Unbounded);
        assert_eq!(Div(1).partial_rcmp(bounds), pos(W, W));

        // 1 is below all multiples of 2
        let bounds = (Bound::Included(Div(2)), Bound::Unbounded);
        assert_eq!(Div(1).partial_rcmp(bounds), pos(O, W));

        let bounds = (Bound::Included(&Div(2)), Bound::Unbounded);
        assert_eq!(Div(1).partial_rcmp(bounds), pos(O, W));

        // 2 is incomparable with the multiples of 3
        let bounds = (Bound::Included(Div(3)), Bound::Unbounded);
        assert_eq!(Div(2).partial_rcmp(bounds), pos(I, W));

        let bounds = (Bound::Included(&Div(3)), Bound::Unbounded);
        assert_eq!(Div(2).partial_rcmp(bounds), pos(I, W));
    }

    #[test]
    fn bounds_to() {
        // 4 is above ]-inf, 2[
        let bounds = (Bound::Unbounded, Bound::Excluded(Div(2)));
        assert_eq!(Div(4).partial_rcmp(bounds), pos(W, O));

        // 1 is inside ]-inf, 2[
        let bounds = (Bound::Unbounded, Bound::Excluded(Div(2)));
        assert_eq!(Div(1).partial_rcmp(bounds), pos(W, W));

        // 3 is incomparable with the divisors of 10
        let bounds = (Bound::Unbounded, Bound::Excluded(&Div(10)));
        assert_eq!(Div(3).partial_rcmp(bounds), pos(W, I));
    }

    #[test]
    fn bounds() {
        // 3 is above [1, 3[
        let bounds = (Bound::Included(Div(1)), Bound::Excluded(Div(3)));
        assert_eq!(Div(3).partial_rcmp(bounds), pos(W, O));

        // 6 is inside [2, 12[
        let bounds = (Bound::Included(&Div(2)), Bound::Excluded(&Div(12)));
        assert_eq!(Div(6).partial_rcmp(bounds), pos(W, W));

        // 2 is below [4, 8[
        let bounds = (Bound::Included(Div(4)), Bound::Excluded(Div(8)));
        assert_eq!(Div(2).partial_rcmp(bounds), pos(O, W));
    }

    #[test]
    fn bounds_inclusive() {
        // 6 is above [1, 3]
        let bounds = (Bound::Included(Div(1)), Bound::Included(Div(3)));
        assert_eq!(Div(6).partial_rcmp(bounds), pos(W, O));

        // 6 is inside [6, 6]
        let bounds = (Bound::Included(&Div(6)), Bound::Included(&Div(6)));
        assert_eq!(Div(6).partial_rcmp(bounds), pos(W, W));

        // 2 is below [4, 8]
        let bounds = (Bound::Included(Div(4)), Bound::Included(Div(8)));
        assert_eq!(Div(2).partial_rcmp(bounds), pos(O, W));
    }

    #[test]
    fn bounds_to_inclusive() {
        // 4 is above ]-inf, 2]
        let bounds = (Bound::Unbounded, Bound::Included(Div(2)));
        assert_eq!(Div(4).partial_rcmp(bounds), pos(W, O));

        // 1 is inside ]-inf, 2]
        let bounds = (Bound::Unbounded, Bound::Included(&Div(2)));
        assert_eq!(Div(1).partial_rcmp(bounds), pos(W, W));
    }

    #[test]
    fn bounds_exclusive_inclusive() {
        // 6 is above ]1, 3]
        let bounds = (Bound::Excluded(Div(1)), Bound::Included(Div(3)));
        assert_eq!(Div(6).partial_rcmp(bounds), pos(W, O));

        // 1 is below ]1, 2]: 1 == 1 violates the excluded lower bound
        let bounds = (Bound::Excluded(Div(1)), Bound::Included(Div(2)));
        assert_eq!(Div(1).partial_rcmp(bounds), pos(O, W));
    }

    #[test]
    fn bounds_as_reference() {
        let bounds = Div(2)..Div(12);
        assert_eq!(Div(6).partial_rcmp(&bounds), pos(W, W));
        assert_eq!(Div(6).partial_rcmp(bounds), pos(W, W));
    }

    /// The key cases the old `Option<RangeOrdering>` design lost: a value comparable with
    /// one bound and incomparable with the other. The per-bound information is preserved.
    #[test]
    fn comparable_to_one_bound_only() {
        // 4 is a multiple of 2 (within the lower bound) but incomparable with 9
        assert_eq!(Div(4).partial_rcmp(Div(2)..Div(9)), pos(W, I));
        assert_eq!(Div(4).partial_rcmp(Div(2)..Div(9)).ordering(), None);

        // 2 is below 4 (outside the lower bound) but incomparable with 9
        assert_eq!(Div(2).partial_rcmp(Div(4)..Div(9)), pos(O, I));
        assert_eq!(Div(2).partial_rcmp(Div(4)..Div(9)).ordering(), None);

        // 4 is incomparable with 3 but a divisor of 12 (within the upper bound)
        assert_eq!(Div(4).partial_rcmp(Div(3)..Div(12)), pos(I, W));
        assert_eq!(Div(4).partial_rcmp(Div(3)..Div(12)).ordering(), None);
    }

    #[test]
    fn empty_ranges() {
        // [8, 2[ is inverted; 4 is a divisor of 8 (below the lower bound) and a multiple
        // of 2 (above the upper bound), witnessing the emptiness.
        assert_eq!(Div(4).partial_rcmp(Div(8)..Div(2)), pos(O, O));
        assert_eq!(
            Div(4).partial_rcmp(Div(8)..Div(2)).ordering(),
            Some(RangeOrdering::Empty)
        );
    }
}
