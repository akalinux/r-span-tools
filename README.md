# common-range-tools ![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue) [![common-range-tools on crates.io](https://img.shields.io/crates/v/common-range-tools)](https://crates.io/crates/common-range-tools) [![common-range-tools on docs.rs](https://docs.rs/common-range-tools/badge.svg)](https://docs.rs/common-range-tools)

## Overview

The **common-range-tools** crate, is a library that, can be used to find all common intersections for ranges of generic types.  
It interoperates with the built in range types for rust via the [std::ops::RangeBounds][__link0] trait.  When working with primitive
numbers, the increment and decrementing of values are always checked, preventing overflows and underflows.

### Numbers

The **common-range-tools** crate, implements support for all primitive number types in rust via the [NumberIncDecCpCmp][__link1] object type.

### Default Number Example

This example converts [std::ops::RangeBounds][__link2] instances to [std::ops::RangeInclusive][__link3].
The bounds to the left or right of .. represent the ($ty::MIN)..($ty::MAX), defined by the [NumberIncDecCpCmp][__link4] object.
The min and max numbers can be changed, but this example, uses the defaults.

```rust

// Import the Intersector
use common_range_tools::Intersector;

fn main() {
    let mut isec = Intersector::num_defaults();
    let range = 1..4;
    isec.add_range(&range);
    let range_inclusive = 3..=5;
    isec.add_range(&range_inclusive);
    let min_to_end = ..=7;
    isec.add_range(&min_to_end);
    let begin_to_max = 7..;
    isec.add_range(&begin_to_max);

    // Note 7.. and ..7 include our min and max all ready.. so this is a bit redundant
    // but works non the less.
    let min_to_max = ..;
    isec.add_range(&min_to_max);

    println!("//! |Start|End|");
    println!("//! |-----|---|");
    for i in isec.into_iter() {
        println!("//! |{:^14}|{:^14}|", i.start(), i.end());
    }
}


```

So using the default values we for [i32::MIN][__link5] and [i32::MAX][__link6], we end up with the following data range intersections.

|Start|End|
|-----|---|
|-2147483648|1|
|2|3|
|4|5|
|6|6|
|7|7|
|8|2147483647|

### Numeric Boundries

In truth the defaults are useful but in most cases the min and max are something we will want to set.
In this example we set the following:

|field|what it does|
|-----|------------|
|step|sets the value used to progress between the begin or end of a range|
|rebound|sets the value used to redefine a range fom an [std::ops::Bound::Excluded][__link7]|
|min|the minimum value for ranges in the context of: **..**|
|max|the maximum vaue for ranges in the context of: **..**|

```rust

// Import the Intersector
use common_range_tools::Intersector;

fn main() {
    let mut isec = Intersector::num(
        1, // step
        1, // rebound
        0, // min
        8, // max
    );
    let range = 1..4;
    isec.add_range(&range);
    let range_inclusive = 3..=5;
    isec.add_range(&range_inclusive);
    let min_to_end = ..=7;
    isec.add_range(&min_to_end);

    println!("//! |Start|End|");
    println!("//! |-----|---|");
    for i in isec.into_iter() {
        println!("//! |{:^14}|{:^14}|", i.start(), i.end());
    }
}


```

The resulting table now has 0 as our min, and 8 as our max.

|Start|End|
|-----|---|
|0|1|
|2|3|
|4|5|
|6|7|
|8|8|

### Working with Floats

When working with floaing points, its nessesary to understand how floats are handled by the internals.
Floating point numbers are in a word **imprecise**; The internals cannot check them for over or underflow;
The internals of [NumberIncDecCpCmp][__link8] simply makes sure that the values properly increment and decrement.

```rust

use common_range_tools::{IncDecCpCmp, NumberIncDecCpCmp};

fn main() {
    let l = NumberIncDecCpCmp::defaults();
    // f32 Increment examples
    assert_eq!(l.inc(&0.2, &0.5), Some(0.7));
    assert_eq!(l.inc(&1.7, &-0.5), None);
    assert_eq!(l.inc(&f32::INFINITY, &0.5), None);
    assert_eq!(l.inc(&f32::INFINITY, &f32::INFINITY), None);
    assert_eq!(l.inc(&1.0, &f32::INFINITY), Some(f32::INFINITY));
    assert_eq!(l.inc(&1.0, &f32::NEG_INFINITY), None);

    // f32 Decrement examples
    assert_eq!(l.dec(&0.5, &0.5), Some(-0.0));
    assert_eq!(l.dec(&1.7, &-0.5), None);
    assert_eq!(l.dec(&f32::INFINITY, &0.5), None);
    assert_eq!(l.dec(&f32::INFINITY, &f32::INFINITY), None);
    assert_eq!(l.dec(&1.0, &f32::INFINITY), Some(f32::NEG_INFINITY));
    assert_eq!(l.dec(&1.0, &f32::NEG_INFINITY), None);
}


```

### Generic Data types

The [AnyIncDecCpCmp][__link9] object supports working with any data type provided it implements: [PartialOrd][__link10], [std::ops::Add][__link11], [std::ops::Sub][__link12], [Copy][__link13], and [Clone][__link14].
In truth the value used by step and rebound do not have to be the same type, a good example of this is [std::time::Duration][__link15] and [std::time::SystemTime][__link16].

```rust

use common_range_tools::Intersector;
use std::time::{Duration, UNIX_EPOCH};

fn main() {
    let min = UNIX_EPOCH;
    let max = UNIX_EPOCH + Duration::from_millis(u64::MAX);
    let step = Duration::from_secs(1);
    let rebound = Duration::from_secs(1);

    let mut isec = Intersector::any(step, rebound, min, max);

    let mut pos = 0;
    for _ in 1..=5 {
        let start = UNIX_EPOCH + Duration::from_secs(pos);
        let end = UNIX_EPOCH + Duration::from_secs(pos + 10);
        pos += 5;
        let range = start..=end;
        isec.add_raw_range(range);
    }
    for r in isec.into_iter() {
        let start = r.start().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let end = r.end().duration_since(UNIX_EPOCH).unwrap().as_secs();
        println!("Start: {}, End: {}", start, end);
    }
}


```

### Beyond Generics

In some cases the ranges do not implement: [PartialOrd][__link17], [std::ops::Add][__link18], [std::ops::Sub][__link19], [Copy][__link20], [Clone][__link21] and [AnyIncDecCpCmp][__link22], or do so in a way
that is incompatable with the required data mode.  The internals of [crate][__link23] use a proxy layer which can be customized to meet most requirements.
This example shows how to work with ragnes of custom data strcutres.

```rust

use common_range_tools::{CpCmp, IncDecCpCmp, Intersector, RiFactory};
#[derive(Clone, Copy, Debug)]
struct Point {
    p: i32,
}

const MIN: Point = Point { p: 0 };
const MAX: Point = Point { p: 8 };
struct CustomIncDecCpCmp {}

impl CpCmp<Point> for CustomIncDecCpCmp {
    fn cp(&self, v: &Point) -> Point {
        return v.clone();
    }

    // The only compare operation that is required!
    fn lt(&self, a: &Point, b: &Point) -> bool {
        a.p < b.p
    }

    fn min(&self) -> Point {
        return MIN;
    }

    fn max(&self) -> Point {
        return MAX;
    }

    fn min_ref(&self) -> &Point {
        &MIN
    }
    fn max_ref(&self) -> &Point {
        &MAX
    }
}

impl IncDecCpCmp<Point, Point> for CustomIncDecCpCmp {
    fn inc(&self, a: &Point, b: &Point) -> Option<Point> {
        match a.p.checked_add(b.p) {
            Some(x) => Some(Point { p: x }),
            None => None,
        }
    }

    fn dec(&self, a: &Point, b: &Point) -> Option<Point> {
        match a.p.checked_sub(b.p) {
            Some(x) => Some(Point { p: x }),
            None => None,
        }
    }
}

fn main() {
    let t = CustomIncDecCpCmp {};

    let mut isec = Intersector::new(
        Vec::new(),       // Container for our internal ranges
        Point { p: 1 },   // step
        Point { p: 1 },   // Rebound value
        t,                // our compare instance
        RiFactory::new(), // Factory used to construct new ranges
    );

    isec.add_range(&(..Point { p: 2 }));
    isec.add_range(&(Point { p: 1 }..Point { p: 3 }));
    isec.add_range(&(Point { p: 3 }..=Point { p: 4 }));
    isec.add_range(&(Point { p: 3 }..));
    for r in isec.into_iter() {
        println!("X: {:?}, Y: {:?}", r.start(), r.end());
    }
}


```


 [__cargo_doc2readme_dependencies_info]: ggGmYW0CYXZlMC43LjJhdIQb2o_SNWoR6AAb3_T-k0ODPHwbnQW7uS_D2XsbjVFFtK-lC3BhYvVhcoQbcoOgrLqONRcbeafyNPVeTlUba7Ca6_umWeIbTzkuMoz2pPRhZIOCbkFueUluY0RlY0NwQ21w9oJxTnVtYmVySW5jRGVjQ3BDbXD2g3Jjb21tb24tcmFuZ2UtdG9vbHNlMC4xLjByY29tbW9uX3JhbmdlX3Rvb2xz
 [__link0]: https://doc.rust-lang.org/stable/std/?search=ops::RangeBounds
 [__link1]: https://crates.io/crates/NumberIncDecCpCmp
 [__link10]: https://doc.rust-lang.org/stable/std/cmp/trait.PartialOrd.html
 [__link11]: https://doc.rust-lang.org/stable/std/?search=ops::Add
 [__link12]: https://doc.rust-lang.org/stable/std/?search=ops::Sub
 [__link13]: https://doc.rust-lang.org/stable/std/marker/trait.Copy.html
 [__link14]: https://doc.rust-lang.org/stable/std/clone/trait.Clone.html
 [__link15]: https://doc.rust-lang.org/stable/std/?search=time::Duration
 [__link16]: https://doc.rust-lang.org/stable/std/?search=time::SystemTime
 [__link17]: https://doc.rust-lang.org/stable/std/cmp/trait.PartialOrd.html
 [__link18]: https://doc.rust-lang.org/stable/std/?search=ops::Add
 [__link19]: https://doc.rust-lang.org/stable/std/?search=ops::Sub
 [__link2]: https://doc.rust-lang.org/stable/std/?search=ops::RangeBounds
 [__link20]: https://doc.rust-lang.org/stable/std/marker/trait.Copy.html
 [__link21]: https://doc.rust-lang.org/stable/std/clone/trait.Clone.html
 [__link22]: https://crates.io/crates/AnyIncDecCpCmp
 [__link23]: https://crates.io/crates/common-range-tools/0.1.0
 [__link3]: https://doc.rust-lang.org/stable/std/?search=ops::RangeInclusive
 [__link4]: https://crates.io/crates/NumberIncDecCpCmp
 [__link5]: https://doc.rust-lang.org/stable/std/?search=i32::MIN
 [__link6]: https://doc.rust-lang.org/stable/std/?search=i32::MAX
 [__link7]: https://doc.rust-lang.org/stable/std/?search=ops::Bound::Excluded
 [__link8]: https://crates.io/crates/NumberIncDecCpCmp
 [__link9]: https://crates.io/crates/AnyIncDecCpCmp
