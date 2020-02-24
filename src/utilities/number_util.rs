#[allow(dead_code)]
pub fn wrap_f32(basic: f32, min: f32, max: f32) -> f32 {
    let range = max - min;
    basic - f32::floor(basic / range) * range + min
}

#[allow(dead_code)]
pub fn wrap_usize(basic: usize, min: usize, max: usize) -> usize {
    assert!(max > min, "Passed a range into wrap_usize which was incorrect.");
    let range = max - min;
    // we're taking advantage of flooring here...
    basic - (basic / range) * range + min
}

#[allow(dead_code)]
pub fn wrap_isize(basic: isize, min: isize, max: isize) -> isize {
    assert!(max > min, "Passed a range into wrap_usize which was incorrect.");
    let range = max - min;
    // we're taking advantage of flooring here...
    basic - (basic / range) * range + min
}
