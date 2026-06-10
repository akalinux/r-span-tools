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
    //  Common Range: 1->1.9
    //  Common Range: 2->2.5
    //  Common Range: 2.6->3.1
    //  Common Range: 3.2->4.1
    //  Common Range: 4.199999999999999->7.64
}
