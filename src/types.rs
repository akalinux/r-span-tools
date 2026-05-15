/// Trait representing incrementing or decrementing via a checked value.  It is always assumed
/// that, self.checked_inc(rhs) Some(Self) will always return a larger value than either self.
/// Likewise it is always assumed that self.checked_dec(rhs) Some(Self) will always return a value smaller
/// than self.
///
/// # Examples
///
/// When imported the trait is added to integer and floating point primitives.
/// So the examples provided for i32,u32,f32 holds true for all other number primitives.
///
/// ```
/// use common_range_tools::types::SafeIncDec;
///
/// fn main() {
///
///    // i32 example(s)
///    // Increment examples
///    assert!( matches!(1.checked_inc(2), Some(3) ));      // Number went up by 2!
///    assert!( matches!(0.checked_inc(0), None ));         // Number did not go up
///    assert!( matches!(0.checked_inc(-2), None ));        // Number did not go up
///    assert!( matches!(i32::MAX.checked_inc(1), None ));  // Catch overflow
///    assert!( matches!(1.checked_dec(2), Some(-1) ));     // Number went down by 2!
///    assert!( matches!(0.checked_dec(0), None ));         // Number did not go down
///    assert!( matches!(0.checked_dec(-2), None ));        // Number did not go down
///    assert!( matches!(i32::MIN.checked_dec(1), None ));  // Catch undeflow
///
///    // u32 xample(s)
///    assert!( matches!(1_u32.checked_inc(2), Some(3) )); // Number went up by 2!
///    assert!( matches!(0_u32.checked_inc(0), None ));    // Number did not go up
///    assert!( matches!(u32::MAX.checked_inc(1), None )); // Catch overflow
///    assert!( matches!(3_u32.checked_dec(2), Some(1) )); // Number went down by 2!
///    assert!( matches!(0_u32.checked_dec(0), None ));    // Number did not go down
///    assert!( matches!(u32::MIN.checked_dec(1), None )); // Catch undeflow
///
///    // f32 example(s)
///    assert!(matches!((0.2).checked_inc(0.5), Some(1.5)));  // becomes self.ceil() +0.5
///    assert!(matches!((0.2).checked_dec(0.5), Some(-0.5))); // becomes self.floor() -0.5
///    assert!(matches!((1.7).checked_inc(-0.5), None ));     // Number did not go up
///    assert!(matches!((1.7).checked_dec(-0.5), None ));     // Number did not go down
/// 
///    // Adding or sub tracting infinity from a number is valid
///    assert!(matches!((1.0).checked_inc(f32::INFINITY), Some(f32::INFINITY) ));
///    assert!(matches!((1.0).checked_dec(f32::INFINITY), Some(f32::NEG_INFINITY) ));  
/// 
///    // Infinity cannot be incremented or decremented.. so all  of these will fail!
///    assert!(matches!((f32::INFINITY).checked_inc(0.5), None ));            
///    assert!(matches!((f32::INFINITY).checked_dec(0.5), None ));            
///    assert!(matches!((f32::INFINITY).checked_inc(f32::INFINITY), None ));
///    assert!(matches!((f32::INFINITY).checked_dec(f32::INFINITY), None ));
/// 
///    // in this case we cannot increment or decrement by negetive infinity!
///    assert!(matches!( (1.0).checked_dec(f32::NEG_INFINITY), None));
///    assert!(matches!( (1.0).checked_inc(f32::NEG_INFINITY), None));
///
/// }
///
/// ```
///
/// ## Implementation Example
///
/// This example shows how to safely grow or shrik a struct called `MilkSupply`.
///
/// ```
/// use common_range_tools::types::SafeIncDec;
///
/// #[derive(Debug, Copy, Clone, PartialEq)]
/// struct MilkSupply { hundreths: u64 }
///
/// impl SafeIncDec for MilkSupply {
///    fn checked_inc(self,rhs: Self) ->Option<Self> {
///      // if we add the number must always go up!
///      if rhs.hundreths ==0 { return None }
///      // check for overflow
///      let next=self.hundreths.checked_add(rhs.hundreths);
///      match next {
///         Some(hundreths)=>Some(MilkSupply { hundreths } ),
///         None=>None,
///      }
///    }
///
///    fn checked_dec(self,rhs: Self) ->Option<Self> {
///      // if we subtract the number must always go down!
///      if rhs.hundreths==0 { return None }
///      // check for overflow
///      let next=self.hundreths.checked_sub(rhs.hundreths);
///      match next {
///         Some(hundreths)=>Some(MilkSupply { hundreths } ),
///         None=>None,
///      }
///    }
/// }
///
/// ```
pub trait SafeIncDec: Sized {
    /// Should capture overflow and the returned Self should be: gt self &&  ltrhs.
    fn checked_inc(self, rhs: Self) -> Option<Self>;
    /// Should capture overflow and the returned Self should be: tt self && lt rhs.
    fn checked_dec(self, rhs: Self) -> Option<Self>;
}

/// The minimum constraint for value of a RangeSet.
pub trait RangeValue: Clone + PartialOrd {}
impl<T: Clone + PartialOrd<Self>> RangeValue for T {}

/// The minimum constraint for value of a RangeSet value that can be incremented or decremented.
pub trait RangeAddSubValue: RangeValue + SafeIncDec {}
impl<T: RangeValue + SafeIncDec> RangeAddSubValue for T {}

/// Creates unsigned integer behabior for SafeIncDec.
#[macro_export]
macro_rules! impl_checked_inc_sub_u {
    ($($t:ty),*) => {
        $(

            impl SafeIncDec for $t {
                fn checked_dec(self, rhs: Self) ->Option<Self> {
                    if rhs==0  { return None }
                    return self.checked_sub(rhs);
                }
                fn checked_inc(self, rhs: Self) -> Option<Self> {
                    if rhs==0 { return None }
                    return self.checked_add(rhs)
                }
            }
        )*
    };
}

/// Creates signed integer behabior for SafeIncDec.
#[macro_export]
macro_rules! impl_checked_inc_sub_i {
    ($($t:ty),*) => {
        $(
            impl SafeIncDec for $t {
                fn checked_dec(self, rhs: Self) ->Option<Self> {
                    if rhs<=0 { return None}
                    return self.checked_sub(rhs);
                }
                fn checked_inc(self, rhs: Self) -> Option<Self> {
                    if rhs<=0 { return None}
                    return self.checked_add(rhs)
                }
            }
        )*
    };
}

/// Creates float integer behabior for SafeIncDec.
#[macro_export]
macro_rules! impl_checked_inc_sub_f {
    ($($t:ty),*) => {
        $(
            impl SafeIncDec for $t {
                fn checked_dec(self, rhs: Self) ->Option<Self> {
                    let floor=self.floor();
                    let res=floor - rhs;
                    if res.is_nan() || res >=floor { None } else { Some(res) }
                }
                fn checked_inc(self, rhs: Self) -> Option<Self> {
                    let ceil=self.ceil();
                    let res=ceil + rhs;
                    if res.is_nan() || res <=ceil { None } else { Some(res) }
                }
            }
        )*
    };
}

impl_checked_inc_sub_i!(i8, i16, i32, i64, i128, isize);
impl_checked_inc_sub_f!(f32, f64);
impl_checked_inc_sub_u!(u8, u16, u32, u64, u128, usize);

#[cfg(test)]
mod safe_sub_add_tests {
    use crate::types::SafeIncDec;

    #[test]
    fn test_safe_add_sub_doc_example() {
        assert!(matches!(1.checked_inc(2), Some(3))); // Number went up by 2!
        assert!(matches!(0.checked_inc(0), None)); // Number did not go up
        assert!(matches!(0.checked_inc(-2), None)); // Number did not go up
        assert!(matches!(i32::MAX.checked_inc(1), None)); // Catch overflow

        // Decrement examples
        assert!(matches!(1.checked_dec(2), Some(-1))); // Number went down by 2!
        assert!(matches!(0.checked_dec(0), None)); // Number did not go down
        assert!(matches!(0.checked_dec(-2), None)); // Number did not go down
        assert!(matches!(i32::MIN.checked_dec(1), None)); // Catch undeflow

        if let Some(value) = (0.5).checked_dec(0.5) {
            println!(" Result value: {}", value);
        }

        assert!(matches!((0.2).checked_inc(0.5), Some(1.5)));
        assert!(matches!((0.5).checked_dec(0.5), Some(-0.5)));
        assert!(matches!((1.7).checked_inc(-0.5), None));
        assert!(matches!((1.7).checked_dec(-0.5), None));
        assert!(matches!((f32::INFINITY).checked_inc(0.5), None));
        assert!(matches!((f32::INFINITY).checked_dec(0.5), None));
        assert!(matches!((f32::INFINITY).checked_inc(f32::INFINITY), None));
        assert!(matches!((f32::INFINITY).checked_dec(f32::INFINITY), None));

        assert!(matches!(1_u32.checked_inc(2), Some(3))); // Number went up by 2!
        assert!(matches!(0_u32.checked_inc(0), None)); // Number did not go up
        assert!(matches!(u32::MAX.checked_inc(1), None)); // Catch overflow

        assert!(matches!(3_u32.checked_dec(2), Some(1))); // Number went down by 2!
        assert!(matches!(0_u32.checked_dec(0), None)); // Number did not go down
        assert!(matches!(u32::MIN.checked_dec(1), None)); // Catch undeflow
        assert!(matches!( (1.0).checked_inc(f32::INFINITY), Some(f32::INFINITY)));
        assert!(matches!( (1.0).checked_dec(f32::INFINITY), Some(f32::NEG_INFINITY))); 
        assert!(matches!( (1.0).checked_dec(f32::NEG_INFINITY), None));
        assert!(matches!( (1.0).checked_inc(f32::NEG_INFINITY), None));
    }

    #[test]
    fn test_add_sub() {
        // int positive test
        let mut i: Option<u8> = 1.checked_inc(2);
        assert!(matches!(i, Some(3)));
        i = 1.checked_dec(1);
        assert!(matches!(i, Some(0)));

        // negative test
        for (a, b) in [(255, 1), (0, 0)] {
            i = a.checked_inc(b);
            assert!(matches!(i, None));
        }
        for (a, b) in [(0, 1), (0, 0)] {
            i = a.checked_dec(b);
            assert!(matches!(i, None));
        }

        // float tests
        let mut f: Option<f32> = 1.0.checked_inc(1.0);
        if let Some(c) = f {
            assert!(c > 1.0)
        } else {
            assert!(false);
        }
        f = 1.0.checked_dec(1.0);
        if let Some(c) = f {
            assert!(c < 1.0)
        } else {
            assert!(false);
        }

        for (a, b) in [(f32::INFINITY, 1.0), (0.0, 0.0)] {
            f = a.checked_inc(b);
            assert!(matches!(f, None));
        }

        let mut u: Option<i8> = 1.checked_inc(2);
        assert!(matches!(u, Some(3)));
        u = 1.checked_dec(1);
        assert!(matches!(u, Some(0)));
        u = (-1).checked_dec(1);
        assert!(matches!(u, Some(-2)));

        // negative test
        for (a, b) in [(127, 1), (0, 0)] {
            u = a.checked_inc(b);
            assert!(matches!(u, None));
        }
        for (a, b) in [(-128, 1), (0, 0), (1, -1)] {
            u = a.checked_dec(b);
            assert!(matches!(u, None));
        }
    }
}
