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
