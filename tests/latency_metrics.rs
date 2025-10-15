use std::thread;
use std::time::Duration;

use liminal_voice_core::metrics;

#[test]
fn latency_total_exceeds_sleep() {
    let mut vm = metrics::start();
    thread::sleep(Duration::from_millis(20));
    metrics::finish(&mut vm);
    assert!(vm.total_ms >= 20);
}

#[test]
fn clamp01_bounds() {
    assert_eq!(metrics::clamp01(-0.5), 0.0);
    assert_eq!(metrics::clamp01(0.5), 0.5);
    assert_eq!(metrics::clamp01(1.5), 1.0);
}
