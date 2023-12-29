pub fn get_distance(a: f32, b: f32) -> f32 {
    if a > b {
        return a-b;
    } else {
        return b-a;
    }
}