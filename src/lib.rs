//! # R Span Tools
//!
//! `r_span_tools` is a library that, can be used to find all common intersections of ranges for generic typs.

use core::range::RangeInclusive;

#[doc(inline)]
use crate::types::{RangeAddSubValue, RangeValue};
pub mod iter;
pub mod types;
pub mod utils;

pub enum RangeRelation {
    Before,
    Overlap,
    After,
}

pub trait RangeSet<T: RangeValue> {
    fn get_begin(&self) -> &T;
    fn get_end(&self) -> &T;

    fn contains_value(&self, value: &T) -> bool {
        !(value < self.get_begin() || value > self.get_end())
    }

    fn contains_range(&self, check: &dyn RangeSet<T>) -> bool {
        return self.contains_value(check.get_begin()) || self.contains_value(check.get_end());
    }

    fn overlap(&self, check: &dyn RangeSet<T>) -> bool {
        return self.contains_range(check)
            || check.contains_value(&self.get_begin())
            || check.contains_value(&self.get_end());
    }

    fn is_valid(&self) -> bool {
        return self.get_begin() <= self.get_end();
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

impl<T: RangeValue> RangeSet<T> for RangeInclusive<T> {
    fn get_begin(&self) -> &T {
        return &self.start;
    }
    fn get_end(&self) -> &T {
        return &self.last;
    }

    fn is_valid(&self) -> bool {
        return !self.is_empty();
    }
}

pub struct Span<T: RangeValue> {
    begin: T,
    end: T,
}

impl<T: RangeValue> RangeSet<T> for Span<T> {
    fn get_begin(&self) -> &T {
        &self.begin
    }

    fn get_end(&self) -> &T {
        &self.end
    }
}

impl<T: RangeAddSubValue> Span<T> {
    pub fn new(begin: T, end: T) -> Self {
        return Span { begin, end };
    }
}
