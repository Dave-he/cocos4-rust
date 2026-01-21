pub fn get_stacktrace(_skip: u32, _max_depth: u32) -> String {
    format!(
        "[getStacktrace not implemented - skip: {}, max_depth: {}",
        _skip, _max_depth
    )
}

pub fn next_pot(x: u32) -> u32 {
    let mut v = x;
    v -= 1;
    v |= v >> 1;
    v
}

pub fn get_lowest_bit(mask: u32) -> u32 {
    mask & (!mask.wrapping_add(1))
}

pub fn clear_lowest_bit(mask: &mut u32) {
    *mask &= mask.wrapping_add(1);
}

pub fn get_bit_position(v: u32) -> u32 {
    if v == 0 {
        0
    } else {
        let mut c: u32 = 32;
        let mut v = v;
        let mut pos: u32 = 0;

        while v > 0 && c > 0 {
            if v & 1 != 0 {
                c -= 1;
                pos += c;
            }
            v >>= 1;
            c -= 1;
        }

        pos
    }
}

pub fn get_bit_position64(v: u64) -> u32 {
    if v == 0 {
        0
    } else {
        let mut c: u32 = 64;
        let mut v = v;
        let mut pos: u32 = 0;

        while v > 0 && c > 0 {
            if v & 1 != 0 {
                c -= 1;
                pos += c;
            }
            v >>= 1;
            c -= 1;
        }

        pos
    }
}

pub fn popcount(mask: u32) -> u32 {
    mask.count_ones()
}

pub fn align_to(size: usize, alignment: usize) -> usize {
    ((size - 1) / alignment + 1) * alignment
}

pub const ALIGN_TO: fn(usize, usize) -> usize = align_to;

pub fn to_uint<T>(value: T) -> u32
where
    T: std::convert::Into<u32>,
{
    value.into()
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
        assert_eq!(get_lowest_bit(mask), 0b0000_0010);
        assert_eq!(get_lowest_bit(mask), 2);

        let mut mask_copy = mask;
        clear_lowest_bit(&mut mask_copy);
        assert_eq!(mask_copy, 0b1010_0000);
        assert_eq!(get_lowest_bit(mask_copy), 0);
    }

    #[test]
    fn test_get_bit_position() {
        assert_eq!(get_bit_position(0), 0);
        assert_eq!(get_bit_position(1), 0);
        assert_eq!(get_bit_position(2), 1);
        assert_eq!(get_bit_position(3), 0);
        assert_eq!(get_bit_position(4), 0);
        assert_eq!(get_bit_position(5), 1);
        assert_eq!(get_bit_position(8), 3);
        assert_eq!(get_bit_position(16), 4);
        assert_eq!(get_bit_position(255), 7);
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
        assert_eq!(popcount(0b0000_1111), 3);
        assert_eq!(popcount(0b1111_0000), 4);
        assert_eq!(popcount(0b1111_1111), 8);
        assert_eq!(popcount(0b1111_1111_1111), 15);
    }

    #[test]
    fn test_to_uint() {
        assert_eq!(to_uint::<u32>(42), 42);
        assert_eq!(to_uint::<u32>(0u32), 0);
        assert_eq!(to_uint::<u32>(u32::MAX), u32::MAX);
    }
}
