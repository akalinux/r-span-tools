//! # Overview
//! The **common-range-tools** crate, is a library that, can be used to find all common intersections for ranges of generic types.  It interoperates with the built in range types for rust via the [std::ops::RangeBounds] trait.  When working with primitive numbers, the increment and decrementing of values are checked (see [working with floats](#working-with-floats), for the exception).  Support for all primitive number types in rust is implemented via the [NumberIncDecCpCmp] object.
//!
//! For dealing with ranges beyond just intersections of
//! numbers see: [Generic Data Types](#generic-data-types).  For working with custom data structures see: [Beyond Generics](#beyond-generics).
//! For working with custom Ranges and range factories see: [Internal Range Trait](#internal-range-trait).
//! For consolidating duplicate and overlapping ranges see [Consolidation of ranges](#consolidation-of-ranges).
//! For finding intersections between muliple [Iterator] instances, see: [Intersections of Itertors](#intersections-of-itertors).
//!
//! ## Example
//! This is the most basic example, using the default values from [NumberIncDecCpCmp].  The [OverlapIter] is a [DoubleEndedIterator] and can be reversed.
#![doc = "```rust\n"]
#![doc = include_str!("../examples/example.rs")]
#![doc = "\n```"]
//!
//!
//! ## Any Range Example
//! This example converts [std::ops::RangeBounds] instances to [std::ops::RangeInclusive].
//! The bounds to the left or right of .. represent the ($ty::MIN)..($ty::MAX), defined by the [NumberIncDecCpCmp] object.
//! The min and max numbers can be changed, but this example uses the defaults.
#![doc = "```rust\n"]
#![doc = include_str!("../examples/tldr.rs")]
#![doc = "\n```"]
//!
//! ## Numeric Boundries
//! When working with boundries it is useful to be able to control how ranges are interpeted.  The defaults provided by [NumberIncDecCpCmp] are useful
//! but do not cover all casees.  The internals of this [crate] allow for setting various options to control how both ranges and intersections are computed.
//!
//! In this example we set the following:
//! | Field | What it does|
//! |------|----|
//! | step | sets the value used to progress to the next begin or end value for a given range |
//! | rebound | sets the value used to redefine a range value fom an instance of: [std::ops::Bound::Excluded] |
//! | min |  the minimum value for ranges in the context of: **..** |
//! | max | the maximum vaue for ranges in the context of: **..** |
#![doc = "```rust\n"]
#![doc = include_str!("../examples/setting_boundries.rs")]
#![doc = "\n```"]
//!
//! ## Working with Floats
//! When working with floating points, its nessesary to understand how floats are handled by the [NumberIncDecCpCmp].
//! Floating point numbers are in a word *imprecise*; The internals of [NumberIncDecCpCmp] does not check [f32] or [f64] for over or underflow;
//! The internals of [NumberIncDecCpCmp] simply checks that the values properly increment and decrement.
#![doc = "```rust\n"]
#![doc = include_str!("../examples/floats.rs")]
#![doc = "\n```"]
//!
//! ## Generic Data types
//! The [AnyIncDecCpCmp] object supports working with any data type, provided it implements: [PartialOrd], [std::ops::Add], [std::ops::Sub], [Copy], and [Clone].
//! When working with generics, the value used by step and rebound values do not have to be the same type as the values used by a range.
//! A practical example of this is how [std::time::Duration] and [std::time::SystemTime] handle [std::ops::Add] and [std::ops::Sub].  
//! This is an example that shows how to use [std::time::Duration] to provide the step and rebound values and [std::time::SystemTime] to operate as the range values:
#![doc = "```rust\n"]
#![doc = include_str!("../examples/systemtime.rs")]
#![doc = "\n```"]
//! ## Beyond Generics
//! In some cases the range values do not implement: [PartialOrd], [std::ops::Add], [std::ops::Sub], [Copy], [Clone] or do so in a way
//! that is incompatable with the required data structure used as a value for the range.  The internals of [OverlapIter] use a proxy layer which can be customized to meet most requirements.
//! This example shows how to work with ragnes of custom data strcutres.
#![doc = "```rust\n"]
#![doc = include_str!("../examples/beyond_any.rs")]
#![doc = "\n```"]
//!
//! ## Internal Range Trait
//! Rust has no single trait representing rages aside from [std::ops::RangeBounds], which can require recomputing the begin and or end
//! values of a range on each evaluation.  To work around this the internals of this crate use a common trait range type of [GetBeginEnd].
//! There is also a factory trait for creating instances called [GetBeginEndOption].  This example shows how to create and use both the
//! factory: [GetBeginEndOption] and the range: [GetBeginEnd].  As a note the [GetBeginEnd] trait is implemnted for [std::ops::RangeInclusive].
#![doc = "```rust\n"]
#![doc = include_str!("../examples/getbeginend.rs")]
#![doc = "\n```"]
//!
//! ## Consolidation of ranges
//! The [Consolidate] object can be used to consolidate duplicate and overlapping ranges via an [Iterator] of ranges.
//! It is recommended to convert an instance of [Consolidate] into an instance of [ConsolidateChecker] instance to verify the integrity of the data returned by the [Iterator].
//!
//! The ranges returned by the [Iterator] must be in [ConsolidationOrder].
//!  - For [ConsolidationOrder::Forward] the expected order is: *start asc, end desc*. For more information See: [crate::sort_forward].
//!  - For [ConsolidationOrder::Reverse] the expected order is: *end desc, start asc*. For more information see [crate::sort_reverse].
//!
//! This example demonstrates how to use [Consolidate] wrapped in an instance of [ConsolidateChecker] using the [ConsolidationOrder::Forward]:
#![doc = "```rust\n"]
#![doc = include_str!("../examples/overlaps.rs")]
#![doc = "\n```"]
//!
//! ## Intersections of Itertors
//! The [Columns] object is a factory can be used to construct an [Iterator] that can step through multiple [Iterator] instances of ranges that can contain duplicate
//! and overlapping ranges that intersect with one another.  Each [Column] added to [Columns] is wrapped in an instance of [ConsolidateChecker] to ensure that the consolidation is occuring in the expected [ConsolidationOrder].
//! The ranges returned by the [Iterator] must be in [ConsolidationOrder], see: [Consolidation of ranges](#consolidation-of-ranges) for more information.
//! This example demonstrates how to create a [ColumnsIter] from a [Columns] instance and walk the results.
//!
#![doc = "```rust\n"]
#![doc = include_str!("../examples/columns.rs")]
#![doc = "\n```"]
//!
//! # Motivation
//! In truth there doesn't seem to be a library on crates.io provides the following functionality:
//! - A range intersection library that handles columns of [Iterator] ranges and progress through those ranges correctly.   
//! - An intersection library that could be quickly extended to work with any data structure.  
//! - An intersection library that can support any range type via a a common trait.
//! - A reversable range intersection iterator.
//!
//! Other implementations:
//!   - [range-ext](https://docs.rs/range-ext/0.3.0/range_ext/index.html)
//!   - [range-overlap](https://docs.rs/range-overlap/latest/range_overlap/)
//!   - [rangetools](https://crates.io/crates/rangetools)
use std::marker::PhantomData;
use std::ops::{Bound, RangeBounds, RangeInclusive};

// re-export to be nice!
pub use crate::builder::*;
pub use crate::iter::consolidate::checker::column::columns::*;
pub use crate::iter::consolidate::checker::column::*;
pub use crate::iter::consolidate::checker::*;
pub use crate::iter::consolidate::*;
pub use crate::iter::*;
pub use crate::utils::*;
pub mod builder;
pub mod iter;
pub mod utils;

/// [Mrs] **Minimal Range Span**
///
/// In a nut shell this is the minimal struct to represent a range for [crate].
/// Requires that [GetBeginEnd] and [std::ops::RangeBounds] be imported to use all implemented traits.
///
/// ```
/// use common_range_tools::{
///   Mrs,
///   GetBeginEnd  // only required for the self.get_begin() and self.get_end() methods.
/// };
/// use std::ops::{RangeBounds,Bound};
///
/// fn main () {
///    let r=Mrs::new(1,2);
///    assert_eq!(r.start_bound(),Bound::Included(&1));
///    assert_eq!(r.end_bound(),Bound::Included(&2));
///    assert_eq!(r.get_begin(),&1);
///    assert_eq!(r.get_end(),&2);
/// }
///
/// ```
pub struct Mrs<T> {
    a: T,
    z: T,
}

/// Proxy data structure for [Mrs].
pub struct MrsP<'r, T, R: GetBeginEnd<T>> {
    r: &'r R,
    _t: PhantomData<T>,
}

/// This struct acts as an owned wrapper for the [Option::Some] produced by the [Consolidate] Iterator,
/// which then acts as a normal instance of [GetBeginEnd].
pub struct ConsolidateMrsP<T, R, S>
where
    R: GetBeginEnd<T>,
    S: GetBeginEnd<T>,
{
    r: R,
    src: Vec<(usize, S)>,
    _t: PhantomData<T>,
}

impl<T, R, S> ConsolidateMrsP<T, R, S>
where
    R: GetBeginEnd<T>,
    S: GetBeginEnd<T>,
{
    /// Wwraps the data set and makes it operate as if it is the instance src.0.
    pub fn new(src: (R, Vec<(usize, S)>)) -> Self {
        Self {
            r: src.0,
            src: src.1,
            _t: PhantomData,
        }
    }

    /// Converts back to the orignal data set used to create this instance.
    pub fn as_src(self) -> (R, Vec<(usize, S)>) {
        return (self.r, self.src);
    }

    /// Returns a ref to the internal data set.
    pub fn src(&self) -> &Vec<(usize, S)> {
        return &self.src;
    }
}

impl<T, R: GetBeginEnd<T>, S: GetBeginEnd<T>> GetBeginEnd<T> for ConsolidateMrsP<T, R, S> {
    /// Wrapper for internal [Mrs] instance.
    fn get_begin(&self) -> &T {
        return self.r.get_begin();
    }

    /// Wrapper for internal [Mrs] instance.
    fn get_end(&self) -> &T {
        return self.r.get_end();
    }

    /// This will drop the sources if used.
    /// Consider using [ConsolidateMrsP::as_src] in stead.
    fn to_tuple(self) -> (T, T) {
        return self.r.to_tuple();
    }
}

impl<T, R: GetBeginEnd<T>, S: GetBeginEnd<T>> RangeBounds<T> for ConsolidateMrsP<T, R, S> {
    /// Wraps the return value from self.get_begin() in a [std::ops::Bound::Included].
    fn start_bound(&self) -> Bound<&T> {
        return Bound::Included(&self.get_begin());
    }

    /// Wraps the return value from self.get_end() in a [std::ops::Bound::Included].
    fn end_bound(&self) -> Bound<&T> {
        return Bound::Included(&self.get_end());
    }
}

impl<'r, T, R: GetBeginEnd<T>> MrsP<'r, T, R> {
    /// Creates a new instance of [MrsP].
    pub fn new(r: &'r R) -> Self {
        return Self { r, _t: PhantomData };
    }
}

/// Trait used by internals to represent ranges.
pub trait GetBeginEnd<T> {
    /// Should return a borrowed instance of the begin value.
    fn get_begin(&self) -> &T;

    /// Should return a borrowed instance of the end value.
    fn get_end(&self) -> &T;

    // Returns a tuple containing (self.get_begin(),self.get_end()).
    fn to_tuple_ref(&self) -> (&T, &T) {
        return (&self.get_begin(), &self.get_end());
    }

    // Implementation should  consume self and return a tuple containing (begin,end).
    fn to_tuple(self) -> (T, T);
}

impl<T> Mrs<T> {
    /// Constructs a new [Mrs] instance.
    pub fn new(a: T, z: T) -> Self {
        Self { a, z }
    }
}

impl<T> From<Mrs<T>> for RangeInclusive<T> {
    fn from(value: Mrs<T>) -> Self {
        let (a, z) = value.to_tuple();
        return RangeInclusive::new(a, z);
    }
}

impl<T> From<RangeInclusive<T>> for Mrs<T> {
    fn from(value: RangeInclusive<T>) -> Self {
        let (a, z) = value.to_tuple();
        return Mrs::new(a, z);
    }
}

impl<T> From<Mrs<T>> for (T, T) {
    fn from(value: Mrs<T>) -> Self {
        return value.to_tuple();
    }
}

impl<T> From<(T, T)> for Mrs<T> {
    fn from(value: (T, T)) -> Mrs<T> {
        return Mrs::new(value.0, value.1);
    }
}

impl<'r, T, R: GetBeginEnd<T>> GetBeginEnd<T> for MrsP<'r, T, R> {
    /// Wrapper for internal [Mrs] instance.
    fn get_begin(&self) -> &T {
        return self.r.get_begin();
    }

    /// Wrapper for internal [Mrs] instance.
    fn get_end(&self) -> &T {
        return self.r.get_end();
    }

    /// Due to the internals being pointer to the real [GetBeginEnd] instance, this method is ***intentionally unimplemented***
    /// and will panic if called!.
    fn to_tuple(self) -> (T, T) {
        unimplemented!();
    }
}

impl<T> GetBeginEnd<T> for Mrs<T> {
    // Returns a borrowed instance of self.z
    fn get_begin(&self) -> &T {
        return &self.a;
    }

    // Returns a borrowed instance of self.z
    fn get_end(&self) -> &T {
        return &self.z;
    }

    // Consumes the instance of self returing a tuple containing (a,z).
    fn to_tuple(self) -> (T, T) {
        return (self.a, self.z);
    }
}

impl<T> GetBeginEnd<T> for RangeInclusive<T> {
    // Returns a borrowed instance of self.z
    fn get_begin(&self) -> &T {
        return &self.start();
    }

    // Returns a borrowed instance of self.z
    fn get_end(&self) -> &T {
        return &self.end();
    }

    // Consumes the instance of self returing a tuple containing (a,z).
    fn to_tuple(self) -> (T, T) {
        return self.into_inner();
    }
}

impl<T> RangeBounds<T> for Mrs<T> {
    /// Wraps the return value from self.get_begin() in a [std::ops::Bound::Included].
    fn start_bound(&self) -> Bound<&T> {
        return Bound::Included(&self.a);
    }

    /// Wraps the return value from self.get_end() in a [std::ops::Bound::Included].
    fn end_bound(&self) -> Bound<&T> {
        return Bound::Included(&self.z);
    }
}
