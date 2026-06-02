#![doc = include_str!("../README.md")]

use std::marker::PhantomData;
use std::ops::{Bound, RangeBounds, RangeInclusive};

// re-export to be nice!
pub use crate::builder::*;
pub use crate::iter::*;
pub use crate::utils::*;
pub mod builder;
pub mod iter;
pub mod utils;

/// [`crate::Mrs`] **Minimal Range Span**
///
/// In a nut shell this is the minimal struct to represent a range for [crate].
/// Requires that [crate::GetBeginEnd] and [std::ops::RangeBounds] be imported to use all implemented traits.
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

/// Proxy data structure for [crate::Mrs].
pub struct MrsP<'r, T, R: GetBeginEnd<T>> {
    r: &'r R,
    _t: PhantomData<T>,
}

/// This struct acts as an owned wrapper for the [Option::Some] produced by the [crate::Consolidate] Iterator,
/// which then acts as a normal instance of [crate::GetBeginEnd].
pub struct ConsolidateMrsP<T, R>
where
    R: GetBeginEnd<T>,
{
    r: R,
    src: Vec<(usize, R)>,
    _t: PhantomData<T>,
}

impl<T, R> ConsolidateMrsP<T, R>
where
    R: GetBeginEnd<T>,
{
    /// Wwraps the data set and makes it operate as if it is the instance src.0.
    pub fn new(src: (R, Vec<(usize, R)>)) -> Self {
        Self {
            r: src.0,
            src: src.1,
            _t: PhantomData,
        }
    }

    /// Converts back to the orignal data set used to create this instance.
    pub fn as_src(self) -> (R, Vec<(usize, R)>) {
        return (self.r, self.src);
    }

    /// Returns a ref to the internal data set.
    pub fn src(&self) -> &Vec<(usize, R)> {
        return &self.src;
    }
}

impl<T, R: GetBeginEnd<T>> GetBeginEnd<T> for ConsolidateMrsP<T, R> {
    /// Wrapper for internal [crate::Mrs] instance.
    fn get_begin(&self) -> &T {
        return self.r.get_begin();
    }

    /// Wrapper for internal [crate::Mrs] instance.
    fn get_end(&self) -> &T {
        return self.r.get_end();
    }

    /// This will drop the sources if used.
    /// Consider using [crate::ConsolidateMrsP::as_src] in stead.
    fn to_tuple(self) -> (T, T) {
        return self.r.to_tuple();
    }
}

impl<T, R: GetBeginEnd<T>> RangeBounds<T> for ConsolidateMrsP<T, R> {
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
    /// Creates a new instance of [crate::MrsP].
    pub fn new(r: &'r R) -> Self {
        return Self { r, _t: PhantomData };
    }
}

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
    /// Constructs a new [crate::Mrs] instance.
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

impl<'r, T, R: GetBeginEnd<T>> GetBeginEnd<T> for MrsP<'r, T, R> {
    /// Wrapper for internal [crate::Mrs] instance.
    fn get_begin(&self) -> &T {
        return self.r.get_begin();
    }

    /// Wrapper for internal [crate::Mrs] instance.
    fn get_end(&self) -> &T {
        return self.r.get_end();
    }

    /// Due to the internals being pointer to the real [crate::GetBeginEnd] instance, this method is ***intentionally unimplemented***
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
