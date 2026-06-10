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
