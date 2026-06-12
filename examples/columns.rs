use common_range_tools::{Columns, DefaultValues, GetBeginEnd, NumberIncDecCpCmp, sort_forward};

fn main() {
    // We create all of our column data unsorted
    let mut col_a = vec![0..=11, 2..=3, 7..=9, 22..=33, 34..=39];
    let mut col_b = vec![6..=9, 6..=9, 6..=7, 11..=22, 7..=11, 9..=9];
    let mut col_c = vec![3..=4, 3..=9, 4..=6, 30..=41];

    // ** Full Sort Example here! **
    // We will use this to drive the internals of the sort function
    let t = NumberIncDecCpCmp::defaults();

    // We create our sort function here
    let sort_by = |a: &std::ops::RangeInclusive<i32>, b: &std::ops::RangeInclusive<i32>| {
        sort_forward(a, b, &t.default_rebound(), &t)
    };

    // Sort all of our rows and force them to exist in the correct order!
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
            // We still want to access the column or columns that error out before we stop
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
