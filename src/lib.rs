//! # Common Range tools
//!
//! The `common-range-tool` is a library that, can be used to find all common intersections for ranges of generic types.

#[doc(inline)]
pub use crate::iter::*;
#[doc(inline)]
pub use crate::types::*;
#[doc(inline)]
pub use crate::utils::*;
pub mod iter;
pub mod types;
pub mod utils;

pub enum RangeRelation {
    Before,
    Overlap,
    After,
}
// The minimal RangeSet Implementation!
pub struct Mrs<T> {
    a: T,
    z: T,
}

impl<T: RangeValue> Mrs<T> {
    pub fn new(a: T, z: T) -> Self {
        return Self { a, z };
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
pub trait GetBeginEnd<T> {
    fn get_begin(&self) -> &T;
    fn get_end(&self) -> &T;
}

impl<T: RangeValue> RangeSet<T> for Mrs<T> {
    fn get_begin(&self) -> &T {
        return &self.a;
    }
    fn get_end(&self) -> &T {
        return &self.z;
    }
}

pub trait RangeSet<T: RangeValue> {
    fn get_begin(&self) -> &T;
    fn get_end(&self) -> &T;

    fn contains_value(&self, value: &T) -> bool {
        !self.is_invalid() && !(value < self.get_begin() || value > self.get_end())
    }

    fn contains_range(&self, check: &dyn RangeSet<T>) -> bool {
        return self.contains_value(check.get_begin()) || self.contains_value(check.get_end());
    }

    fn overlap(&self, check: &dyn RangeSet<T>) -> bool {
        return self.contains_range(check)
            || check.contains_value(self.get_begin())
            || check.contains_value(self.get_end());
    }

    /// Returns true when self.get_begin() gt self.get_end().
    fn is_invalid(&self) -> bool {
        return self.get_begin() > self.get_end();
    }

    /// Provides positional relationship of range to self
    fn range_relation(&self, range: &dyn RangeSet<T>) -> RangeRelation {
        if range.get_end() < self.get_begin() {
            return RangeRelation::Before;
        } else if self.get_end() < range.get_begin() {
            return RangeRelation::After;
        }
        return RangeRelation::Overlap;
    }
}

#[cfg(test)]
mod range_set_tests {

    use crate::{Mrs, RangeSet};

    #[test]
    fn mrs_test_is_invalid() {
        assert!(Mrs::new(0, -1).is_invalid());
        assert!(!Mrs::new(0, 0).is_invalid());
        assert!(!Mrs::new(0, 1).is_invalid());
    }

    #[test]
    fn mrs_test_contains_value() {
        assert!(!Mrs::new(0, -1).contains_value(&0));
        assert!(!Mrs::new(0, -1).contains_value(&-1));
        assert!(Mrs::new(0, 1).contains_value(&0));
        assert!(Mrs::new(0, 1).contains_value(&1));
    }
}
