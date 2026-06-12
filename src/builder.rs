use std::ops::RangeInclusive;
use std::{
    marker::PhantomData,
    ops::{Add, Bound, Sub},
};

use crate::{GetBeginEnd, Mrs};

/// The **Number Implementation** of [IncDecCpCmp].  
///
/// Acts as the general proxy layer for safly working with primitive number types inside [crate].
/// Note: unlike [AnyIncDecCpCmp], the self.inc(a,b) and self.dec(a,b) are checked.
///
/// The following types are implemented for [NumberIncDecCpCmp]
///  - [u8], [u16], [u32], [u64], [u128], [usize]
///  - [i8], [i16], [i32], [i64], [i128], [isize]
///  - [f32], [f64]
#[derive(Clone, Copy, Debug)]
pub struct NumberIncDecCpCmp<T>
where
    T: Copy + Clone,
{
    min: T,
    max: T,
}

/// The **Generic Implementation** of [IncDecCpCmp].
///
/// Acts as the general proxy layer for working with any type inside [crate].
///
/// Note: unlike [NumberIncDecCpCmp], the self.inc(a,b) and self.dec(a,b) are unchecked!
/// If you are working with primitive numbers use: [NumberIncDecCpCmp] in stead.
#[derive(Clone, Copy, Debug)]
pub struct AnyIncDecCpCmp<T>
where
    T: PartialOrd + Clone + Copy,
{
    min: T,
    max: T,
}
impl<T> AnyIncDecCpCmp<T>
where
    T: PartialOrd + Clone + Copy,
{
    pub fn new(min: T, max: T) -> Self {
        Self { min, max }
    }

    /// Sets the values returned by self.min() and self.min_ref().
    /// Changing this value from the default will further constrain what ranges are considered invalid.
    pub fn set_min(&mut self, v: T) {
        self.min = v;
    }

    /// Sets the values returned by self.max() and self.max_ref().
    /// Changing this value from the default will further constrain what ranges are considered invalid.
    pub fn set_max(&mut self, v: T) {
        self.max = v;
    }
}

impl<T> CpCmp<T> for AnyIncDecCpCmp<T>
where
    T: PartialOrd + Copy,
{
    /// Returns a copy of v.
    fn cp(&self, v: &T) -> T {
        return *v;
    }

    /// Returns a ref to self.min.
    fn min_ref(&self) -> &T {
        return &self.min;
    }

    /// Returns a ref to self.max.
    fn max_ref(&self) -> &T {
        return &self.max;
    }

    /// Returns true if a lt b.
    fn lt(&self, a: &T, b: &T) -> bool {
        return a < b;
    }

    /// Returns a copy of min.
    fn min(&self) -> T {
        return self.min;
    }

    /// Returns a copy of max.
    fn max(&self) -> T {
        return self.max;
    }
}

impl<T, V> IncDecCpCmp<T, V> for AnyIncDecCpCmp<T>
where
    V: Copy,
    T: PartialOrd + Copy + Add<V, Output = T> + Sub<V, Output = T>,
{
    /// Increments a by b, if the resulting value is le a None is returned.
    fn inc(&self, a: &T, b: &V) -> Option<T> {
        let x = *a;
        let c = a.add(*b);
        if x <= c {
            return Some(c);
        }
        return None;
    }

    /// Increments a by b, if the resulting value is ge a None is returned.
    fn dec(&self, a: &T, b: &V) -> Option<T> {
        let x = *a;
        let c = a.sub(*b);
        if x >= c {
            return Some(c);
        }
        return None;
    }

    fn cp_v(&self, v: &V) -> V {
        return *v;
    }
}
impl<T> NumberIncDecCpCmp<T>
where
    T: Clone + Copy,
    NumberIncDecCpCmp<T>: DefaultValues<T, T>,
{
    pub fn defaults() -> Self {
        return Self::new(
            NumberIncDecCpCmp::default_min(),
            NumberIncDecCpCmp::default_max(),
        );
    }
    pub fn new(min: T, max: T) -> Self {
        Self { min, max }
    }

    /// Sets the values returned by self.min() and self.min_ref().
    /// Changing this value from the default will further constrain what ranges are considered invalid.
    pub fn set_min(&mut self, v: T) {
        self.min = v;
    }

    /// Sets the values returned by self.max() and self.max_ref().
    /// Changing this value from the default will further constrain what ranges are considered invalid.
    pub fn set_max(&mut self, v: T) {
        self.max = v;
    }
}

/// **Copy and Compare Values**
///
/// For an implementation examples and details see: [IncDecCpCmp].
pub trait CpCmp<T> {
    //. Should return a clone or copy of &T.
    fn cp(&self, v: &T) -> T;

    fn min_ref(&self) -> &T;
    fn max_ref(&self) -> &T;

    /// Returns true if a < b.
    fn lt(&self, a: &T, b: &T) -> bool;

    /// Returns the minimum begin value we will accept when converting from [`std::ops::Bound::Unbounded`].
    fn min(&self) -> T;

    /// Returns the maximum end value we will accept when converting from [`std::ops::Bound::Unbounded`].
    fn max(&self) -> T;

    /// Returns true if a gt b.
    fn gt(&self, a: &T, b: &T) -> bool {
        return self.lt(b, a);
    }

    /// Returns true if a eq b.
    fn eq(&self, a: &T, b: &T) -> bool {
        return !self.lt(a, b) && !self.lt(b, a);
    }

    /// Returns true if a ne b.
    fn ne(&self, a: &T, b: &T) -> bool {
        return self.lt(a, b) || self.lt(b, a);
    }

    /// Returns true if a le b.
    fn le(&self, a: &T, b: &T) -> bool {
        return self.lt(a, b) || !self.lt(b, a);
    }

    /// Returns true if a ge b.
    fn ge(&self, a: &T, b: &T) -> bool {
        return self.lt(b, a) || !self.lt(a, b);
    }

    /// Returns true if a and b contain c.
    fn contains(&self, a: &T, b: &T, c: &T) -> bool {
        return !(self.lt(c, a) || self.lt(b, c));
    }

    /// Returns a new owned copy of the input ref tuple.
    fn cp_tpl_ref(&self, src: (&T, &T)) -> (T, T) {
        return (self.cp(src.0), self.cp(src.1));
    }

    /// Returns true if any of the following are true
    /// - a and b contain c
    /// - a and b contain d
    /// - c and d contain a
    /// - c and d contain b
    fn overlap(&self, a: &T, b: &T, c: &T, d: &T) -> bool {
        return self.contains(a, b, c)
            || self.contains(a, b, d)
            || self.contains(c, d, a)
            || self.contains(c, d, b);
    }

    /// Returns true if b lt a or a lt self.min_ref() or self.max_ref() lt b.
    fn is_invalid_set(&self, a: &T, b: &T) -> bool {
        return self.lt(b, a) || self.lt(a, self.min_ref()) || self.lt(self.max_ref(), b);
    }
}

/// **Increment, Decrement, Copy, Compare Values and Ranges**
///
/// This is the base trait used to represent range manipulation by [crate].  
/// In general this library implements the proxy or wrapper approach to range and value manipulation as apposed to
/// manipulation by value.  This means the trait implementation for values does not need to be implemented on the generic type.
/// Since the operations are not implemented by the values, this frees us from the worry of conflicting trait resoltion.   This
/// also means we can quickly implement how the values and ranges are manipualted for the same data for a different task.
///
/// Example Implementation
///
/// ```
/// use common_range_tools::{CpCmp,IncDecCpCmp};
///
/// struct MyTrait {}
///
/// // First: implement CpCmp<T> for your trait.
/// impl CpCmp<i32> for MyTrait {
///     fn min(&self) ->i32 { <i32>::MIN }
///     fn max(&self) ->i32 { <i32>::MAX }
///     fn min_ref(&self) ->&i32 { &<i32>::MIN }
///     fn max_ref(&self) ->&i32 { &<i32>::MAX }
///
///     fn cp(&self,v: &i32) ->i32 { return v.clone() }
///
///     fn lt(&self,a:&i32,b: &i32) ->bool {  return a<b  }
/// }
///
/// // Next: implement IncDecCpCmp<T> for your trait.
/// impl IncDecCpCmp<i32,i32> for MyTrait {
///     fn dec(&self, a: &i32, b:&i32) ->Option<i32> {
///         if *b<=0 { return None}
///         return a.clone().checked_sub(b.clone());
///     }
///     fn inc(&self, a: &i32, b: &i32) -> Option<i32> {
///         if *b<=0 { return None}
///         return a.clone().checked_add(b.clone())
///     }
///     fn cp_v(&self,v:&i32) ->i32 {
///         return *v;
///     }
/// }
/// ```
///
pub trait IncDecCpCmp<T, V>: CpCmp<T> {
    /// Should safely increment a by b.  The value should always go up.. if not then it should return None.
    fn inc(&self, a: &T, b: &V) -> Option<T>;

    /// Should safely decrement a by b.  The value should always go down... if not then it should return None.
    fn dec(&self, a: &T, b: &V) -> Option<T>;

    fn cp_v(&self, v: &V) -> V;

    /// Returns the raw adjusted start value.
    ///   - [std::ops::Bound::Unbounded] becomes self.min()
    ///   - [std::ops::Bound::Included] value is not changed
    ///   - [std::ops::Bound::Excluded] value is incremented
    fn rebound_start(&self, start: Bound<&T>, rebound: &V) -> Option<T> {
        match start {
            Bound::Included(begin) => Some(self.cp(begin)),
            Bound::Excluded(begin) => self.inc(begin, rebound),
            Bound::Unbounded => Some(self.min()),
        }
    }

    /// Returns the raw adjusted end value.
    ///   - [std::ops::Bound::Unbounded] becomes
    ///   - [std::ops::Bound::Included] value is not changed
    ///   - [std::ops::Bound::Excluded] value is decremented
    fn rebound_end(&self, end: Bound<&T>, rebound: &V) -> Option<T> {
        match end {
            Bound::Included(end) => Some(self.cp(end)),
            Bound::Excluded(end) => self.dec(end, rebound),
            Bound::Unbounded => Some(self.max()),
        }
    }
}

/// **Default values**
///
/// Implemenations of this trait drive the internals used for [crate::iter::Intersector].
pub trait DefaultValues<T, V>: IncDecCpCmp<T, V> {
    /// Returns the default value use for progressing a begin or end value of a range.
    fn default_step(&self) -> V;

    /// Returns the value used to adjust a start or end value in the context of [std::ops::Bound].
    fn default_rebound(&self) -> V;

    /// Returns the default minimum value.
    fn default_min() -> T;

    /// Returns the default maximum value.
    fn default_max() -> T;
}

macro_rules! impl_inc_dec_cp_cmp_trait_i {
    ($($t:ty),*) => {
        $(
            impl CpCmp<$t> for NumberIncDecCpCmp<$t> {
                fn min(&self) ->$t { self.min }
                fn max(&self) ->$t { self.max }
                fn min_ref(&self) ->&$t { &self.min }
                fn max_ref(&self) ->&$t { &self.max }

                fn cp(&self,v: &$t) ->$t {
                    return v.clone();
                }

                fn lt(&self,a:&$t,b: &$t) ->bool {
                    return a<b;
                }
            }

            impl IncDecCpCmp<$t,$t> for NumberIncDecCpCmp<$t> {
                fn dec(&self, a: &$t, b:&$t) ->Option<$t> {
                    if *b<=0 { return None}
                    return a.clone().checked_sub(b.clone());
                }
                fn inc(&self, a: &$t, b: &$t) -> Option<$t> {
                    if *b<=0 { return None}
                    return a.clone().checked_add(b.clone())
                }
                fn cp_v(&self,v: &$t) ->$t {
                    return *v;
                }
            }

            impl DefaultValues<$t,$t> for NumberIncDecCpCmp<$t> {
                fn default_step(&self) ->$t { return 1}
                fn default_rebound(&self) ->$t { return 1}
                fn default_min() ->$t { <$t>::MIN }
                fn default_max() ->$t { <$t>::MAX }
            }
        )*
    };
}

macro_rules! impl_inc_dec_cp_cmp_trait_u {
    ($($t:ty),*) => {
        $(
            impl CpCmp<$t> for NumberIncDecCpCmp<$t> {
                fn min(&self) ->$t { self.min }
                fn max(&self) ->$t { self.max }
                fn min_ref(&self) ->&$t { &self.min }
                fn max_ref(&self) ->&$t { &self.max }

                fn cp(&self,v: &$t) ->$t {
                    return v.clone();
                }

                fn lt(&self,a:&$t,b: &$t) ->bool {
                    return a<b;
                }
            }
            impl IncDecCpCmp<$t,$t> for NumberIncDecCpCmp<$t> {
                fn dec(&self, a: &$t, b:&$t) ->Option<$t> {
                    if *b==0 { return None}
                    return a.clone().checked_sub(b.clone());
                }
                fn inc(&self, a: &$t, b: &$t) -> Option<$t> {
                    if *b==0 { return None}
                    return a.clone().checked_add(b.clone())
                }
                fn cp_v(&self,v: &$t) ->$t {
                    return *v;
                }
            }

            impl DefaultValues<$t,$t> for NumberIncDecCpCmp<$t> {
                fn default_step(&self) ->$t { return 1}
                fn default_rebound(&self) ->$t { return 1}
                fn default_min() ->$t { <$t>::MIN }
                fn default_max() ->$t { <$t>::MAX }
            }
        )*
    };
}

macro_rules! impl_inc_dec_cp_cmp_trait_f {
    ($($t:ty),*) => {
        $(


            impl CpCmp<$t> for NumberIncDecCpCmp<$t> {

                fn min(&self) ->$t { self.min }
                fn max(&self) ->$t { self.max }
                fn min_ref(&self) ->&$t { &self.min }
                fn max_ref(&self) ->&$t { &self.max }
                fn cp(&self,v: &$t) ->$t {
                    return v.clone();
                }
                fn lt(&self,a:&$t,b: &$t) ->bool {
                    return a<b;
                }
            }
            impl IncDecCpCmp<$t,$t> for NumberIncDecCpCmp<$t> {
                fn dec(&self, a: &$t, b:&$t) ->Option<$t> {
                    let res=a - b;
                    if res.is_nan() || res >=*a { None } else { Some(res) }
                }
                fn inc(&self, a: &$t, b: &$t) -> Option<$t> {
                    let res=a + b;
                    if res.is_nan() || res <=*a { None } else { Some(res) }
                }
                fn cp_v(&self,v: &$t) ->$t {
                    return *v;
                }
            }

            impl DefaultValues<$t,$t> for NumberIncDecCpCmp<$t> {
                fn default_step(&self) ->$t { return 1.0}
                fn default_rebound(&self) ->$t { return 1.0 }
                fn default_min() ->$t { <$t>::MIN }
                fn default_max() ->$t { <$t>::MAX }
            }
        )*
    };
}

impl_inc_dec_cp_cmp_trait_u!(u8, u16, u32, u64, u128, usize);
impl_inc_dec_cp_cmp_trait_i!(i8, i16, i32, i64, i128, isize);
impl_inc_dec_cp_cmp_trait_f!(f32, f64);

/// This trait represents how ranges factories are to be implemented.
pub trait GetBeginEndOption<T, R: GetBeginEnd<T>> {
    fn factory(&self, opt: Option<(T, T)>) -> Option<R>;
    fn new_range(&self, src: (T, T)) -> R;
}

/// This is the factory implemntation of [GetBeginEndOption] for [Mrs].
#[derive(Copy, Clone)]
pub struct MrsFactory<T> {
    _t: PhantomData<T>,
}

/// This is the factory implementation of [GetBeginEndOption] for [std::ops::RangeInclusive].
#[derive(Copy, Clone)]
pub struct RiFactory<T> {
    _t: PhantomData<T>,
}

impl<T> RiFactory<T> {
    pub fn new() -> Self {
        return Self { _t: PhantomData };
    }
}

impl<T> MrsFactory<T> {
    pub fn new() -> Self {
        return Self { _t: PhantomData };
    }
}

impl<T> GetBeginEndOption<T, Mrs<T>> for MrsFactory<T> {
    fn new_range(&self, src: (T, T)) -> Mrs<T> {
        return Mrs::new(src.0, src.1);
    }
    fn factory(&self, opt: Option<(T, T)>) -> Option<Mrs<T>> {
        match opt {
            Some((a, z)) => Some(Mrs::new(a, z)),
            None => None,
        }
    }
}

impl<T> GetBeginEndOption<T, RangeInclusive<T>> for RiFactory<T> {
    fn new_range(&self, src: (T, T)) -> RangeInclusive<T> {
        return RangeInclusive::new(src.0, src.1);
    }
    fn factory(&self, opt: Option<(T, T)>) -> Option<RangeInclusive<T>> {
        match opt {
            Some((a, z)) => Some(RangeInclusive::new(a, z)),
            None => None,
        }
    }
}
