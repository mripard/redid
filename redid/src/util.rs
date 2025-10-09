pub(crate) fn calculate_checksum(data: &[u8]) -> u8 {
    let mut sum: u8 = 0;

    for byte in data {
        sum = sum.wrapping_add(*byte);
    }

    0u8.wrapping_sub(sum)
}
