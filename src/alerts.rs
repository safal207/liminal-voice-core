#[derive(Default)]
pub struct AlertStats {
    pub drift_breaches: usize,
    pub res_breaches: usize,
    pub total: usize,
    pub max_drift: f32,
    pub min_res: f32,
}

pub fn update(stats: &mut AlertStats, drift: f32, res: f32, base_drift: f32, base_res: f32) {
    stats.total += 1;
    if drift > base_drift {
        stats.drift_breaches += 1;
    }
    if res < base_res {
        stats.res_breaches += 1;
    }
    if drift > stats.max_drift {
        stats.max_drift = drift;
    }
    if stats.min_res == 0.0 || res < stats.min_res {
        stats.min_res = res;
    }
}

pub fn summary_lines(stats: &AlertStats, base_drift: f32, base_res: f32) -> Vec<String> {
    let header = format!(
        "[health] baseline_drift>{:.2}, baseline_res<{:.2}",
        base_drift, base_res
    );
    let breaches = format!(
        "[health] breaches: drift={}, res={}, total={}",
        stats.drift_breaches, stats.res_breaches, stats.total
    );
    let worst = format!(
        "[health] worst: drift_max={:.2}, res_min={:.2}",
        stats.max_drift, stats.min_res
    );
    let ok = stats.drift_breaches == 0 && stats.res_breaches == 0;
    let status = format!(
        "[health] status: {}",
        if ok { "OK ✅" } else { "ATTENTION ⚠️" }
    );

    vec![header, breaches, worst, status]
}

pub fn print_summary(stats: &AlertStats, base_drift: f32, base_res: f32) {
    let lines = summary_lines(stats, base_drift, base_res);
    if lines.is_empty() {
        return;
    }
    println!();
    for line in lines {
        println!("{}", line);
    }
}
