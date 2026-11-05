/// Calculates width in pixels for each segment of the multi-segment physical memory distribution bar.
///
/// Returns `(in_use_w, standby_w, mod_w, remaining_w)`.
pub fn calculate_memory_bar_segments(
    total_mb: u64,
    used_mb: u64,
    standby_mb: u64,
    modified_mb: u64,
    bar_width: i32,
) -> (i32, i32, i32, i32) {
    if total_mb == 0 || bar_width <= 0 {
        return (0, 0, 0, bar_width.max(0));
    }
    let in_use_w = (((used_mb as u128) * (bar_width as u128)) / (total_mb as u128)) as i32;
    let standby_w = (((standby_mb as u128) * (bar_width as u128)) / (total_mb as u128)) as i32;
    let mod_w = (((modified_mb as u128) * (bar_width as u128)) / (total_mb as u128)) as i32;
    let allocated = in_use_w + standby_w + mod_w;
    let remaining_w = (bar_width - allocated).max(0);
    (in_use_w, standby_w, mod_w, remaining_w)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_memory_bar_segments() {
        let (in_use, standby, mod_w, rem) = calculate_memory_bar_segments(16384, 8192, 4096, 1024, 340);
        assert_eq!(in_use, 170); // 50%
        assert_eq!(standby, 85);  // 25%
        assert_eq!(mod_w, 21);   // 6.25%
        assert_eq!(rem, 64);     // remaining free RAM
        assert_eq!(in_use + standby + mod_w + rem, 340);
    }

    #[test]
    fn test_calculate_memory_bar_segments_zero_total() {
        let (in_use, standby, mod_w, rem) = calculate_memory_bar_segments(0, 0, 0, 0, 340);
        assert_eq!((in_use, standby, mod_w, rem), (0, 0, 0, 340));
    }
}
