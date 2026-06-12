# common-range-tools ![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue) [![common-range-tools on crates.io](https://img.shields.io/crates/v/common-range-tools)](https://crates.io/crates/common-range-tools) [![common-range-tools on docs.rs](https://docs.rs/common-range-tools/badge.svg)](https://docs.rs/common-range-tools)

## Overview

The **common-range-tools** crate, is a library that, can be used to find all common intersections for ranges of generic types.  It interoperates with the built in range types for rust via the [std::ops::RangeBounds][__link0] trait.  When working with primitive numbers, the increment and decrementing of values are checked (see [working with floats](#working-with-floats), for the exception).  Support for all primitive number types in rust is implemented via the [NumberIncDecCpCmp][__link1] object.

For dealing with ranges beyond just intersections of
numbers see: [Generic Data Types](#generic-data-types).  For working with custom data structures see: [Beyond Generics](#beyond-generics).
For working with custom Ranges and range factories see: [Internal Range Trait](#internal-range-trait).
For consolidating duplicate and overlapping ranges see [Consolidation of ranges](#consolidation-of-ranges).
For finding intersections between muliple [Iterator][__link2] instances, see: [Intersections of Itertors](#intersections-of-itertors).

### Example

This is the most basic example, using the default values from [NumberIncDecCpCmp][__link3].  The [OverlapIter][__link4] is a [DoubleEndedIterator][__link5] and can be reversed.

```rust

use common_range_tools::Intersector;

fn main() {
    // RangeInclusive used to make this more readable.
    let src = [1..=4, 0..=3, 3..=11, 10..=22];
    // Forwards
    println!("Forwards");
    for r in Intersector::num_from(&src) {
        println!("Common Range: {}->{}", r.start(), r.end());
    }
    // Output will be
    //  Forwards
    //  Common Range: 0->0
    //  Common Range: 1->2
    //  Common Range: 3->3
    //  Common Range: 4->4
    //  Common Range: 5->9
    //  Common Range: 10->11
    //  Common Range: 12->22

    // add a small bumper to the output
    print!("\n\n");
    // Backwards
    println!("Backwards");
    for r in Intersector::num_from(&src).rev() {
        println!("Common Range: {}->{}", r.start(), r.end());
    }
    // Outout will be
    //  Backwards
    //  Common Range: 12->22
    //  Common Range: 10->11
    //  Common Range: 5->9
    //  Common Range: 4->4
    //  Common Range: 3->3
    //  Common Range: 1->2
    //  Common Range: 0->0
}


```

### Any Range Example

This example converts [std::ops::RangeBounds][__link6] instances to [std::ops::RangeInclusive][__link7].
The bounds to the left or right of .. represent the ($ty::MIN)..($ty::MAX), defined by the [NumberIncDecCpCmp][__link8] object.
The min and max numbers can be changed, but this example uses the defaults.

```rust

// Import the Intersector
use common_range_tools::Intersector;

fn main() {
    let mut isec = Intersector::num_defaults();

    // 1 to 3
    let range: std::ops::Range<i32> = 1..4;
    isec.add_range(&range);

    // 3 to 5
    let range_inclusive: std::ops::RangeInclusive<i32> = 3..=5;
    isec.add_range(&range_inclusive);

    //  -2147483648 to 7
    let min_to_end: std::ops::RangeToInclusive<i32> = ..=7;
    isec.add_range(&min_to_end);

    // 7 to 2147483647
    let begin_to_max: std::ops::RangeFrom<i32> = 7..;
    isec.add_range(&begin_to_max);

    // Note 7.. and ..7 include our min and max all ready!
    let min_to_max: std::ops::RangeFull = ..;
    isec.add_range(&min_to_max);

    for i in isec.into_iter() {
        println!("Common Range: {:^14}->{:^14}", i.start(), i.end());
    }

    // The Output will be:
    //  Common Range:  -2147483648  ->      0
    //  Common Range:       1       ->      2
    //  Common Range:       3       ->      3
    //  Common Range:       4       ->      5
    //  Common Range:       6       ->      6
    //  Common Range:       7       ->      7
    //  Common Range:       8       ->  2147483647
}


```

### Numeric Boundries

When working with boundries it is useful to be able to control how ranges are interpeted.  The defaults provided by [NumberIncDecCpCmp][__link9] are useful
but do not cover all casees.  The internals of this [crate][__link10] allow for setting various options to control how both ranges and intersections are computed.

In this example we set the following:

|Field|What it does|
|-----|------------|
|step|sets the value used to progress to the next begin or end value for a given range|
|rebound|sets the value used to redefine a range value fom an instance of: [std::ops::Bound::Excluded][__link11]|
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
    // 1 to 3
    let range: std::ops::Range<i32> = 1..4;
    isec.add_range(&range);

    // 3 to 5
    let range_inclusive: std::ops::RangeInclusive<i32> = 3..=5;
    isec.add_range(&range_inclusive);

    // 0 to 7
    let min_to_end: std::ops::RangeToInclusive<i32> = ..=7;
    isec.add_range(&min_to_end);

    // 7 to 8
    let begin_to_max: std::ops::RangeFrom<i32> = 7..;
    isec.add_range(&begin_to_max);

    // Note 7.. and ..7 include our min and max all ready!
    let min_to_max: std::ops::RangeFull = ..;
    isec.add_range(&min_to_max);
    for i in isec.into_iter() {
        println!("  Common Range {:^3}->{:^3}", i.start(), i.end());
    }
    // The output will be:
    //  Common Range  0 -> 0
    //  Common Range  1 -> 2
    //  Common Range  3 -> 3
    //  Common Range  4 -> 5
    //  Common Range  6 -> 6
    //  Common Range  7 -> 7
    //  Common Range  8 -> 8
}


```

### Working with Floats

When working with floating points, its nessesary to understand how floats are handled by the [NumberIncDecCpCmp][__link12].
Floating point numbers are in a word *imprecise*; The internals of [NumberIncDecCpCmp][__link13] does not check [f32][__link14] or [f64][__link15] for over or underflow;
The internals of [NumberIncDecCpCmp][__link16] simply checks that the values properly increment and decrement.

```rust

use common_range_tools::{IncDecCpCmp, Intersector, NumberIncDecCpCmp};

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

    // This sets the step and rebound value to 0.1
    for r in Intersector::num_sr_from(0.1, &[1.0..=3.1, 2.5..=4.1, 1.9..=7.64]) {
        print!("Common Range: {}->{}\n", r.start(), r.end());
    }

    // The resulting output will be:
    //  Common Range: 1->1.7999999999999998
    //  Common Range: 1.9->2.4
    //  Common Range: 2.5->3.1
    //  Common Range: 3.2->4.1
    //  Common Range: 4.199999999999999->7.64
}


```

### Generic Data types

The [AnyIncDecCpCmp][__link17] object supports working with any data type, provided it implements: [PartialOrd][__link18], [std::ops::Add][__link19], [std::ops::Sub][__link20], [Copy][__link21], and [Clone][__link22].
WHen working with generics, the value used by step and rebound do not have to be the same type as the values used by a range.
A practical example of why this is useful is [std::time::Duration][__link23] and [std::time::SystemTime][__link24].

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

In some cases the range values do not implement: [PartialOrd][__link25], [std::ops::Add][__link26], [std::ops::Sub][__link27], [Copy][__link28], [Clone][__link29] or do so in a way
that is incompatable with the required data structure used as a value for the range.  The internals of [OverlapIter][__link30] use a proxy layer which can be customized to meet most requirements.
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
    // a way to copy or clone the current struct is required!
    fn cp(&self, v: &Point) -> Point {
        return v.clone();
    }
    // All compare operations can be derived from either lt or gt.
    // The only compare method that is required to be implemented
    // for this trait is the lt operator.
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

    // Note: an internal RangeInclusive instance is
    // generated for every range that is successfully added.
    // This means it is safe to drop the orginal range values
    // after they are loaded into the Intersector instance.
    isec.add_range(&(..Point { p: 2 }));
    isec.add_range(&(Point { p: 1 }..Point { p: 3 }));
    isec.add_range(&(Point { p: 3 }..=Point { p: 4 }));
    isec.add_range(&(Point { p: 3 }..));
    for r in isec.into_iter() {
        println!("X: {:?}, Y: {:?}", r.start(), r.end());
    }
    // Output will be:
    //  X: Point { p: 0 }, Y: Point { p: 0 }
    //  X: Point { p: 1 }, Y: Point { p: 1 }
    //  X: Point { p: 2 }, Y: Point { p: 2 }
    //  X: Point { p: 3 }, Y: Point { p: 4 }
    //  X: Point { p: 5 }, Y: Point { p: 8 }
}


```

### Internal Range Trait

Rust has no single trait representing rages aside from [std::ops::RangeBounds][__link31], which can require recomputing the begin and or end
values of a range on each evaluation.  To work around this the internals of this crate use a common trait range type of [GetBeginEnd][__link32].
There is also a factory trait for creating instances called [GetBeginEndOption][__link33].  This example shows how to create and use both the
factory: [GetBeginEndOption][__link34] and the range: [GetBeginEnd][__link35].  As a note the [GetBeginEnd][__link36] trait is implemnted for [std::ops::RangeInclusive][__link37].

```rust

use common_range_tools::{GetBeginEnd, GetBeginEndOption, NumberIncDecCpCmp, OverlapIter};

struct MyFactory {}

#[derive(Clone, Copy, Debug)]
struct MyRange {
    a: i32,
    b: i32,
}

impl GetBeginEndOption<i32, MyRange> for MyFactory {
    fn factory(&self, opt: Option<(i32, i32)>) -> Option<MyRange> {
        match opt {
            Some((a, b)) => Some(MyRange { a, b }),
            None => None,
        }
    }

    fn new_range(&self, src: (i32, i32)) -> MyRange {
        return MyRange { a: src.0, b: src.1 };
    }
}
impl GetBeginEnd<i32> for MyRange {
    fn get_begin(&self) -> &i32 {
        return &self.a;
    }

    fn get_end(&self) -> &i32 {
        return &self.b;
    }

    fn to_tuple(self) -> (i32, i32) {
        return (self.a, self.b);
    }
}

fn main() {
    for r in OverlapIter::new(
        vec![
            MyRange { a: 1, b: 4 },
            MyRange { a: 3, b: 5 },
            MyRange { a: 4, b: 6 },
        ],
        1,
        NumberIncDecCpCmp::defaults(),
        MyFactory {},
    ) {
        println!("{:?}", r);
    }
}


```

### Consolidation of ranges

The [Consolidate][__link38] object can be used to consolidate duplicate and overlapping ranges via an [Iterator][__link39] of ranges.
It is recommended to convert an instance of [Consolidate][__link40] into an instance of [ConsolidateChecker][__link41] instance to verify the integrity of the data returned by the [Iterator][__link42].

The ranges returned by the [Iterator][__link43] must be in [ConsolidationOrder][__link44].

* For [ConsolidationOrder::Forward][__link45] the expected order is: *start asc, end desc*. For more information See: [crate::sort_forward][__link46].
* For [ConsolidationOrder::Reverse][__link47] the expected order is: *end desc, start asc*. For more information see [crate::sort_reverse][__link48].

This example demonstrates how to use [Consolidate][__link49] wrapped in an instance of [ConsolidateChecker][__link50] using the [ConsolidationOrder::Forward][__link51]:

```rust

use common_range_tools::{Consolidate, ConsolidationOrder};

fn main() {
    for result in Consolidate::num_defaults(
        [
            1..=4,
            3..=5,
            10..=12,
            10..=11,
            13..=13,
            // This will produce an error, because 13..=13 is "After" 1..=2.
            1..=2,
        ]
        .into_iter(),
    )
    .to_consolidate_checker(ConsolidationOrder::Forward)
    {
        match result {
            Ok(row) => {
                // get our outer range and src rows.
                let (outer, src) = row.as_src();
                println!("Outer Range: {}->{}", outer.start(), outer.end());
                for (id, original) in src {
                    println!(
                        "  Row: {}, Range: {}->{}",
                        id,
                        original.start(),
                        original.end()
                    );
                }
            }
            Err((msg, row)) => {
                let (outer, src) = row.as_src();
                println!(
                    "Error: {}, \n    Produced range: {}->{}",
                    msg,
                    outer.start(),
                    outer.end()
                );
                for (id, original) in src {
                    println!(
                        "      Caused by: Row: {}, Range: {}->{}",
                        id,
                        original.start(),
                        original.end()
                    );
                }
            }
        }
    }
}

// Resulting Output will be:
//  Outer Range: 1->5
//    Row: 0, Range: 1->4
//    Row: 1, Range: 3->5
//  Outer Range: 10->12
//    Row: 2, Range: 10->12
//    Row: 3, Range: 10->11
//  Error: Out of Forward Sequence, Expected: Before|Last|Overlap, got: After,
//      Produced range: 1->13
//        Caused by: Row: 4, Range: 13->13
//        Caused by: Row: 5, Range: 1->2


```

### Intersections of Itertors

The [Columns][__link52] object is a factory can be used to construct an [Iterator][__link53] that can step through multiple [Iterator][__link54] instances of ranges that can contain duplicate
and overlapping ranges that intersect with one another.  Each [Column][__link55] added to [Columns][__link56] is wrapped in an instance of [ConsolidateChecker][__link57] to ensure that the consolidation is occuring in the expected [ConsolidationOrder][__link58].
The ranges returned by the [Iterator][__link59] must be in [ConsolidationOrder][__link60], see: [Consolidation of ranges](#consolidation-of-ranges) for more information.
This example demonstrates how to create a [ColumnsIter][__link61] from a [Columns][__link62] instance and walk the results.

```rust

use common_range_tools::{Columns, DefaultValues, GetBeginEnd, NumberIncDecCpCmp, sort_forward};

fn main() {
    // We create all of our column data unsorted
    let mut col_a = vec![0..=11, 2..=3, 7..=9, 22..=33, 34..=39];
    let mut col_b = vec![6..=9, 6..=9, 6..=7, 11..=22, 7..=11, 9..=9];
    let mut col_c = vec![3..=4, 3..=9, 4..=6, 30..=41];

    // ** Full Sort Example here! **
    // We will use this to drive the internals of the sort function
    let t = NumberIncDecCpCmp::defaults();
    // Sort all of our rows and force them to exist in the correct order!
    let sort_by = |a: &std::ops::RangeInclusive<i32>, b: &std::ops::RangeInclusive<i32>| {
        sort_forward(a, b, &t.default_rebound(), &t)
    };
    col_a.sort_by(sort_by);
    col_b.sort_by(sort_by);
    col_c.sort_by(sort_by);
    // ** End Full Sort Example **

    // Create our Columns instance using number defaults.
    let cols = Columns::num_defaults();

    // give up if we fail to add a column!
    assert!(cols.add_column(col_a.into_iter()).is_ok());
    assert!(cols.add_column(col_b.into_iter()).is_ok());
    assert!(cols.add_column(col_c.into_iter()).is_ok());

    // Just pretty printing our text table border
    println!(
        "+---------+-----------+{:-<35}+{:-<61}+{:-<35}+",
        "", "", ""
    );

    // Pretty preint our text table header
    println!(
        "| Overlap | State(id) |{:^35}|{:^61}|{:^35}|",
        "Column(A)", "Column(B)", "Column(C)"
    );
    let mut iter = cols.into_iter();
    let mut id = 0;
    loop {
        let next = iter.next();
        if next.is_none() {
            // print out the last text bumper.
            println!(
                "+---------+-----------+{:-<35}+{:-<61}+{:-<35}+",
                "", "", ""
            );
            return;
        }
        let (overlap, res, columns) = next.unwrap();
        // print a bumper text row.
        println!(
            "+---------+-----------+{:-<35}+{:-<61}+{:-<35}+",
            "", "", ""
        );

        // Print out the common intersecting range!
        print!("|  {:^2}->{:^2} |", overlap.get_begin(), overlap.get_end());
        let mut stop = false;
        if res.is_err() {
            print!("   Err({})  |", id);
            // We still want to access the column or columns that errored out before we stop
            stop = true;
        } else {
            print!("   Ok({})   |", id);
        }
        for (column_id, col) in columns.iter().enumerate() {
            let mut txt = Vec::new();
            match col {
                Ok(src) => {
                    for row in src {
                        // This range contains all of the ranges that were used to create it!
                        let container = row.as_ref();
                        txt.push(format!(
                            "[{}->{}](",
                            container.get_begin(),
                            container.get_end()
                        ));
                        let mut r = Vec::new();

                        // walk our raw source ranges that caused this larger range
                        for (row_id, range) in container.src().iter() {
                            r.push(format!("{}({}->{})", row_id, range.start(), range.end()));
                        }
                        txt.push(r.join(","));
                        txt.push(String::from(")"));
                    }
                }
                Err(msg) => {
                    // Print out our error
                    txt.push(String::from(*msg));

                    // get our raw column and provide the actual erro and the original rows that caused it!
                    let col = iter.get_column(column_id).unwrap();

                    // This Vec contains the rows that caused the error!
                    let rows = col.get_rows();
                    for row in rows {
                        for (row_id, range) in row.as_ref().src().iter() {
                            txt.push(format!("({}){}->{}", row_id, range.start(), range.end()))
                        }
                    }
                }
            }
            match column_id {
                0 => print!("{:^35}|", txt.join("")),
                1 => print!("{:^61}|", txt.join("")),
                2 => print!("{:^35}|", txt.join("")),
                _ => (),
            }
        }

        println!();
        if stop {
            // stop here if we ran into an error processing an iterator.
            break;
        }
        id += 1;
    }
}

// The resulting output will be
//  +---------+-----------+-----------------------------------+-------------------------------------------------------------+-----------------------------------+
//  | Overlap | State(id) |             Column(A)             |                          Column(B)                          |             Column(C)             |
//  +---------+-----------+-----------------------------------+-------------------------------------------------------------+-----------------------------------+
//  |  0 ->2  |   Ok(0)   | [0->11](0(0->11),1(2->3),2(7->9)) |                                                             |                                   |
//  +---------+-----------+-----------------------------------+-------------------------------------------------------------+-----------------------------------+
//  |  3 ->5  |   Ok(1)   | [0->11](0(0->11),1(2->3),2(7->9)) |                                                             |  [3->9](0(3->9),1(3->4),2(4->6))  |
//  +---------+-----------+-----------------------------------+-------------------------------------------------------------+-----------------------------------+
//  |  6 ->9  |   Ok(2)   | [0->11](0(0->11),1(2->3),2(7->9)) | [6->22](0(6->9),1(6->9),2(6->7),3(7->11),4(9->9),5(11->22)) |  [3->9](0(3->9),1(3->4),2(4->6))  |
//  +---------+-----------+-----------------------------------+-------------------------------------------------------------+-----------------------------------+
//  |  10->11 |   Ok(3)   | [0->11](0(0->11),1(2->3),2(7->9)) | [6->22](0(6->9),1(6->9),2(6->7),3(7->11),4(9->9),5(11->22)) |                                   |
//  +---------+-----------+-----------------------------------+-------------------------------------------------------------+-----------------------------------+
//  |  12->21 |   Ok(4)   |                                   | [6->22](0(6->9),1(6->9),2(6->7),3(7->11),4(9->9),5(11->22)) |                                   |
//  +---------+-----------+-----------------------------------+-------------------------------------------------------------+-----------------------------------+
//  |  22->22 |   Ok(5)   |        [22->33](3(22->33))        | [6->22](0(6->9),1(6->9),2(6->7),3(7->11),4(9->9),5(11->22)) |                                   |
//  +---------+-----------+-----------------------------------+-------------------------------------------------------------+-----------------------------------+
//  |  23->29 |   Ok(6)   |        [22->33](3(22->33))        |                                                             |                                   |
//  +---------+-----------+-----------------------------------+-------------------------------------------------------------+-----------------------------------+
//  |  30->33 |   Ok(7)   |        [22->33](3(22->33))        |                                                             |        [30->41](3(30->41))        |
//  +---------+-----------+-----------------------------------+-------------------------------------------------------------+-----------------------------------+
//  |  34->39 |   Ok(8)   |        [34->39](4(34->39))        |                                                             |        [30->41](3(30->41))        |
//  +---------+-----------+-----------------------------------+-------------------------------------------------------------+-----------------------------------+
//  |  40->41 |   Ok(9)   |                                   |                                                             |        [30->41](3(30->41))        |
//  +---------+-----------+-----------------------------------+-------------------------------------------------------------+-----------------------------------+


```

## Motivation

In truth there doesn’t seem to be a library on crates.io provides the following functionality:

* A range intersection library that handles columns of [Iterator][__link63] ranges and progress through those ranges correctly.
* An intersection library that could be quickly extended to work with any data structure.
* An intersection library that can support any range type via a a common trait.
* A reversable range intersection iterator.

Other implementations:

* [range-ext][__link64]
* [range-overlap][__link65]
* [rangetools][__link66]


 [__cargo_doc2readme_dependencies_info]: ggGmYW0CYXZlMC43LjJhdIQb2o_SNWoR6AAb3_T-k0ODPHwbnQW7uS_D2XsbjVFFtK-lC3BhYvVhcoQbaAU88HKQhEUbBPf098X_luQb_jG4opFQ8F8bhp28f-vYw89hZIuCbkFueUluY0RlY0NwQ21w9oJmQ29sdW1u9oJnQ29sdW1uc_aCa0NvbHVtbnNJdGVy9oJrQ29uc29saWRhdGX2gnJDb25zb2xpZGF0ZUNoZWNrZXL2gnJDb25zb2xpZGF0aW9uT3JkZXL2gnFHZXRCZWdpbkVuZE9wdGlvbvaCcU51bWJlckluY0RlY0NwQ21w9oJrT3ZlcmxhcEl0ZXL2g3Jjb21tb24tcmFuZ2UtdG9vbHNlMC4xLjByY29tbW9uX3JhbmdlX3Rvb2xz
 [__link0]: https://doc.rust-lang.org/stable/std/?search=ops::RangeBounds
 [__link1]: https://crates.io/crates/NumberIncDecCpCmp
 [__link10]: https://crates.io/crates/common-range-tools/0.1.0
 [__link11]: https://doc.rust-lang.org/stable/std/?search=ops::Bound::Excluded
 [__link12]: https://crates.io/crates/NumberIncDecCpCmp
 [__link13]: https://crates.io/crates/NumberIncDecCpCmp
 [__link14]: https://doc.rust-lang.org/stable/std/primitive.f32.html
 [__link15]: https://doc.rust-lang.org/stable/std/primitive.f64.html
 [__link16]: https://crates.io/crates/NumberIncDecCpCmp
 [__link17]: https://crates.io/crates/AnyIncDecCpCmp
 [__link18]: https://doc.rust-lang.org/stable/std/cmp/trait.PartialOrd.html
 [__link19]: https://doc.rust-lang.org/stable/std/?search=ops::Add
 [__link2]: https://doc.rust-lang.org/stable/std/iter/trait.Iterator.html
 [__link20]: https://doc.rust-lang.org/stable/std/?search=ops::Sub
 [__link21]: https://doc.rust-lang.org/stable/std/marker/trait.Copy.html
 [__link22]: https://doc.rust-lang.org/stable/std/clone/trait.Clone.html
 [__link23]: https://doc.rust-lang.org/stable/std/?search=time::Duration
 [__link24]: https://doc.rust-lang.org/stable/std/?search=time::SystemTime
 [__link25]: https://doc.rust-lang.org/stable/std/cmp/trait.PartialOrd.html
 [__link26]: https://doc.rust-lang.org/stable/std/?search=ops::Add
 [__link27]: https://doc.rust-lang.org/stable/std/?search=ops::Sub
 [__link28]: https://doc.rust-lang.org/stable/std/marker/trait.Copy.html
 [__link29]: https://doc.rust-lang.org/stable/std/clone/trait.Clone.html
 [__link3]: https://crates.io/crates/NumberIncDecCpCmp
 [__link30]: https://crates.io/crates/OverlapIter
 [__link31]: https://doc.rust-lang.org/stable/std/?search=ops::RangeBounds
 [__link32]: https://docs.rs/common-range-tools/0.1.0/common_range_tools/trait.GetBeginEnd.html
 [__link33]: https://crates.io/crates/GetBeginEndOption
 [__link34]: https://crates.io/crates/GetBeginEndOption
 [__link35]: https://docs.rs/common-range-tools/0.1.0/common_range_tools/trait.GetBeginEnd.html
 [__link36]: https://docs.rs/common-range-tools/0.1.0/common_range_tools/trait.GetBeginEnd.html
 [__link37]: https://doc.rust-lang.org/stable/std/?search=ops::RangeInclusive
 [__link38]: https://crates.io/crates/Consolidate
 [__link39]: https://doc.rust-lang.org/stable/std/iter/trait.Iterator.html
 [__link4]: https://crates.io/crates/OverlapIter
 [__link40]: https://crates.io/crates/Consolidate
 [__link41]: https://crates.io/crates/ConsolidateChecker
 [__link42]: https://doc.rust-lang.org/stable/std/iter/trait.Iterator.html
 [__link43]: https://doc.rust-lang.org/stable/std/iter/trait.Iterator.html
 [__link44]: https://crates.io/crates/ConsolidationOrder
 [__link45]: https://docs.rs/ConsolidationOrder/latest/ConsolidationOrder/?search=Forward
 [__link46]: https://docs.rs/common-range-tools/0.1.0/common_range_tools/?search=sort_forward
 [__link47]: https://docs.rs/ConsolidationOrder/latest/ConsolidationOrder/?search=Reverse
 [__link48]: https://docs.rs/common-range-tools/0.1.0/common_range_tools/?search=sort_reverse
 [__link49]: https://crates.io/crates/Consolidate
 [__link5]: https://doc.rust-lang.org/stable/std/iter/trait.DoubleEndedIterator.html
 [__link50]: https://crates.io/crates/ConsolidateChecker
 [__link51]: https://docs.rs/ConsolidationOrder/latest/ConsolidationOrder/?search=Forward
 [__link52]: https://crates.io/crates/Columns
 [__link53]: https://doc.rust-lang.org/stable/std/iter/trait.Iterator.html
 [__link54]: https://doc.rust-lang.org/stable/std/iter/trait.Iterator.html
 [__link55]: https://crates.io/crates/Column
 [__link56]: https://crates.io/crates/Columns
 [__link57]: https://crates.io/crates/ConsolidateChecker
 [__link58]: https://crates.io/crates/ConsolidationOrder
 [__link59]: https://doc.rust-lang.org/stable/std/iter/trait.Iterator.html
 [__link6]: https://doc.rust-lang.org/stable/std/?search=ops::RangeBounds
 [__link60]: https://crates.io/crates/ConsolidationOrder
 [__link61]: https://crates.io/crates/ColumnsIter
 [__link62]: https://crates.io/crates/Columns
 [__link63]: https://doc.rust-lang.org/stable/std/iter/trait.Iterator.html
 [__link64]: https://docs.rs/range-ext/0.3.0/range_ext/index.html
 [__link65]: https://docs.rs/range-overlap/latest/range_overlap/
 [__link66]: https://crates.io/crates/rangetools
 [__link7]: https://doc.rust-lang.org/stable/std/?search=ops::RangeInclusive
 [__link8]: https://crates.io/crates/NumberIncDecCpCmp
 [__link9]: https://crates.io/crates/NumberIncDecCpCmp
