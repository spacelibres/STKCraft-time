mod cases;
use stkcraft_time::{contract::TimeContext, LeapSecondTable};
fn main() {
    let lsk = LeapSecondTable::builtin().unwrap();
    cases::check(TimeContext::new(&lsk)).unwrap();
    println!(
        "PASS: 3 CSPICE forward bit checks and 3 civil roundtrips; core default features disabled."
    );
    println!("Host correctness check only; no target timing was measured.");
}
