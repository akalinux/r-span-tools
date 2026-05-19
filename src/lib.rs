//! # Common Range tools
//!
//! The `common-range-tool` is a library that, can be used to find all common intersections for ranges of generic types, but
//! also interoperates with the built in range types for rust.

#[doc(inline)]
pub use crate::iter::*;
#[doc(inline)]
pub use crate::utils::*;
pub mod iter;
pub mod utils;

impl<T> GetBeginEnd<T> for Mrs<T> {
    fn get_begin(&self) -> &T {
        return &self.a;
    }
    fn get_end(&self) -> &T {
        return &self.z;
    }
}
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
#[cfg(test)]
mod mrs_tests {
    use crate::{GetBeginEnd, Mrs};

    #[test]
    fn test_gets_begin_end() {
        assert_eq!(Mrs::new(0, 1).get_begin(), &0);
        assert_eq!(Mrs::new(0, 1).get_end(), &1);
    }
}
