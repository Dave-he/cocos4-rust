pub fn get_stacktrace(_skip: u32, _max_depth: u32) -> String {
    format!(
        "[getStacktrace not implemented - skip: {}, max_depth: {}",
        _skip, _max_depth
    )
}

/// Returns the next power of two greater than or equal to x
pub fn next_pot(x: u32) -> u32 {
    if x == 0 {
        return 0;
    }
    let mut v = x - 1;
    v |= v >> 1;
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;
    v + 1
}

/// Returns the lowest set bit in the mask
pub fn get_lowest_bit(mask: u32) -> u32 {
    mask & mask.wrapping_neg()
}

/// Clears the lowest set bit in the mask
pub fn clear_lowest_bit(mask: &mut u32) {
    *mask &= mask.wrapping_sub(1);
}

/// Returns the position of the lowest set bit (0-indexed)
pub fn get_bit_position(v: u32) -> u32 {
    if v == 0 {
        0
    } else {
        v.trailing_zeros()
    }
}

/// Returns the position of the lowest set bit in a u64 (0-indexed)
pub fn get_bit_position64(v: u64) -> u32 {
    if v == 0 {
        0
    } else {
        v.trailing_zeros()
    }
}

/// Returns the number of set bits in the mask
pub fn popcount(mask: u32) -> u32 {
    mask.count_ones()
}

/// Aligns size to the given alignment
pub fn align_to(size: usize, alignment: usize) -> usize {
    size.div_ceil(alignment) * alignment
}

pub const ALIGN_TO: fn(usize, usize) -> usize = align_to;

/// Converts a value to u32
pub fn to_uint<T>(value: T) -> u32
where
    T: std::convert::Into<u32>,
{
    value.into()
}

#[cfg(test)]
mod math_tests {
    use crate::math::*;

    #[test]
    fn test_equals() {
        assert!(approx(1.0, 1.0, None));
        assert!(approx(1.0, 1.0000001, None));
        assert!(!approx(1.0, 2.0, None));
    }

    #[test]
    fn test_clamp() {
        assert_eq!(clamp(5, 0, 10), 5);
        assert_eq!(clamp(-5, 0, 10), 0);
        assert_eq!(clamp(15, 0, 10), 10);
    }

    #[test]
    fn test_clamp01() {
        assert_eq!(clamp01(0.5), 0.5);
        assert_eq!(clamp01(-0.5), 0.0);
        assert_eq!(clamp01(1.5), 1.0);
    }

    #[test]
    fn test_lerp() {
        assert_eq!(lerp(0.0, 10.0, 0.5), 5.0);
        assert_eq!(lerp(0.0, 10.0, 0.0), 0.0);
        assert_eq!(lerp(0.0, 10.0, 1.0), 10.0);
    }

    #[test]
    fn test_angle_conversion() {
        assert!(approx(to_radian(180.0), std::f32::consts::PI, None));
        assert!(approx(to_degree(std::f32::consts::PI), 180.0, None));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_pot() {
        assert_eq!(next_pot(16), 16);
        assert_eq!(next_pot(15), 16);
        assert_eq!(next_pot(17), 32);
        assert_eq!(next_pot(31), 32);
        assert_eq!(next_pot(1), 1);
        assert_eq!(next_pot(0), 0);
    }

    #[test]
    fn test_get_lowest_bit() {
        let mask = 0b1010_1100;
        // The lowest set bit in 0b1010_1100 is at position 2 (value 4)
        assert_eq!(get_lowest_bit(mask), 4);

        let mut mask_copy = mask;
        clear_lowest_bit(&mut mask_copy);
        // After clearing lowest bit, 0b1010_1100 becomes 0b1010_1000
        assert_eq!(mask_copy, 0b1010_1000);
        // The lowest set bit in 0b1010_1000 is at position 3 (value 8)
        assert_eq!(get_lowest_bit(mask_copy), 8);
    }

    #[test]
    fn test_get_bit_position() {
        assert_eq!(get_bit_position(0), 0);
        assert_eq!(get_bit_position(1), 0);  // bit 0 is set
        assert_eq!(get_bit_position(2), 1);  // bit 1 is set
        assert_eq!(get_bit_position(3), 0);  // bit 0 is the lowest set bit
        assert_eq!(get_bit_position(4), 2);  // bit 2 is set
        assert_eq!(get_bit_position(5), 0);  // bit 0 is the lowest set bit
        assert_eq!(get_bit_position(8), 3);  // bit 3 is set
        assert_eq!(get_bit_position(16), 4); // bit 4 is set
        assert_eq!(get_bit_position(255), 0); // bit 0 is the lowest set bit
    }

    #[test]
    fn test_align_to() {
        assert_eq!(align_to(10, 4), 12);
        assert_eq!(align_to(11, 4), 12);
        assert_eq!(align_to(12, 4), 12);
        assert_eq!(align_to(13, 4), 16);
        assert_eq!(align_to(15, 1), 15);
    }

    #[test]
    fn test_popcount() {
        assert_eq!(popcount(0b0000_0000), 0);
        assert_eq!(popcount(0b0000_0001), 1);
        assert_eq!(popcount(0b0000_1111), 4);  // 4 bits set
        assert_eq!(popcount(0b1111_0000), 4);  // 4 bits set
        assert_eq!(popcount(0b1111_1111), 8);  // 8 bits set
        assert_eq!(popcount(0b1111_1111_1111), 12); // 12 bits set
    }

    #[test]
    fn test_to_uint() {
        assert_eq!(to_uint::<u32>(42), 42);
        assert_eq!(to_uint::<u32>(0u32), 0);
        assert_eq!(to_uint::<u32>(u32::MAX), u32::MAX);
    }
}
