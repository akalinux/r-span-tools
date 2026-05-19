//! # Common Range tools
//!
//! The `common-range-tool` is a library that, can be used to find all common intersections for ranges of generic types, but
//! also interoperates with the built in range types for rust.

use std::ops::{Bound, RangeBounds};

#[doc(inline)]
pub use crate::iter::*;
#[doc(inline)]
pub use crate::utils::*;
pub mod iter;
pub mod utils;

/// [`crate::Mrs`] **Minimal Range Span**
///
/// In a nut shell this is the absolut minimal struct to represent a range.
pub struct Mrs<T> {
    a: T,
    z: T,
}

pub trait GetBeginEnd<T> {
    fn get_begin(&self) -> &T;
    fn get_end(&self) -> &T;
}

impl<T> Mrs<T> {
    pub fn new(a: T, z: T) -> Self {
        Self { a, z }
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
    fn start_bound(&self) -> std::ops::Bound<&T> {
        return Bound::Included(&self.a);
    }
    fn end_bound(&self) -> std::ops::Bound<&T> {
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
