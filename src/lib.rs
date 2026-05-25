#![doc = include_str!("../README.md")]

use std::ops::{Bound, RangeBounds};

#[doc(inline)]
pub use crate::builder::*;
#[doc(inline)]
pub use crate::iter::*;
#[doc(inline)]
pub use crate::utils::*;
pub mod iter;
pub mod utils;

pub mod builder;

/// [`crate::Mrs`] **Minimal Range Span**
///
/// In a nut shell this is the absolut minimal struct to represent a range.
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

pub trait GetBeginEnd<T> {
    fn get_begin(&self) -> &T;
    fn get_end(&self) -> &T;
}

impl<T> Mrs<T> {
    pub const fn new(a: T, z: T) -> Self {
        Self { a, z }
    }
    pub fn to_tuple(self) -> (T, T) {
        return (self.a, self.z);
    }
    pub fn to_tuple_ref(&self) -> (&T, &T) {
        return (&self.a, &self.z);
    }
}

impl<T> GetBeginEnd<T> for Mrs<T> {
    fn get_begin(&self) -> &T {
        return &self.a;
    }
    fn get_end(&self) -> &T {
        return &self.z;
    }
}

impl<T> RangeBounds<T> for Mrs<T> {
    fn start_bound(&self) -> Bound<&T> {
        return Bound::Included(&self.a);
    }
    fn end_bound(&self) -> Bound<&T> {
        return Bound::Included(&self.z);
    }
}

#[cfg(test)]
mod mrs_tests {
    use std::ops::{Bound, RangeBounds};

    use crate::{GetBeginEnd, Mrs};

    #[test]
    fn test_gets_begin_end() {
        assert_eq!(Mrs::new(0, 1).get_begin(), &0);
        assert_eq!(Mrs::new(0, 1).get_end(), &1);
    }

    #[test]
    fn test_range_bounds() {
        assert!(matches!(Mrs::new(0, 1).start_bound(), Bound::Included(&0)));
        assert!(matches!(Mrs::new(0, 1).end_bound(), Bound::Included(&1)));
        assert!(Mrs::new(0, 1).contains(&1));
        assert!(Mrs::new(0, 1).contains(&0));
        assert!(!Mrs::new(0, 1).contains(&-1));
    }
}
