pub fn compare_slices(v1: &[i8], v2: &[i8]) -> bool {
    v1
        .iter()
        .zip(v2.iter())
        .all(|(a, b)| {
            *a == *b
        })
}