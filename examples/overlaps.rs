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
