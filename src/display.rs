pub fn millis_to_readable(ms: i64) -> String {
    if ms < 0 {
        panic!()
    }
    let mut s = ms / 1000;
    let mut min = s / 60;
    s -= min * 60;
    let h = min / 60;
    min -= h * 60;
    format!("{}h {}min {}s", h, min, s)
}
