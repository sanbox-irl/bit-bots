pub fn clamp(mut val: f32, min: f32, max: f32) -> f32 {
    assert!(min <= max);

    if val < min {
        val = min;
    }
    if val > max {
        val = max;
    }
    val
}

pub fn clamped(val: &mut f32, min: f32, max: f32) {
    *val = clamp(*val, min, max);
}

pub fn asymptotic_motion(start: f32, end: f32, weight: f32) -> f32 {
    (1.0 - weight) * start + weight * end
}

pub fn approach(start: f32, end: f32, move_amount: f32) -> f32 {
    if start < end {
        (start + move_amount).min(end)
    } else {
        (start - move_amount).max(end)
    }
}
