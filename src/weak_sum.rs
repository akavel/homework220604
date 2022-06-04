use std::fmt;
use std::num::Wrapping;

/// A rolling checksum as described in [the rsync algorithm
/// documentation](https://rsync.samba.org/tech_report/node3.html).
#[derive(Copy, Clone, Default, Eq, PartialEq, Hash)]
pub struct WeakSum {
    a: Wrapping<u16>,
    b: Wrapping<u16>,
}

impl fmt::Debug for WeakSum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.b, self.a)
    }
}

impl From<&[u8]> for WeakSum {
    fn from(buf: &[u8]) -> Self {
        let l = buf.len();
        let mut a = Wrapping(0u16);
        let mut b = Wrapping(0u16);
        for (i, byte) in buf.iter().enumerate() {
            a += *byte as u16;
            b += ((l - i) as u16) * (*byte as u16);
        }
        Self { a, b }
    }
}

impl From<&WeakSum> for u32 {
    fn from(sum: &WeakSum) -> u32 {
        sum.to_u32()
    }
}

impl WeakSum {
    /// Calculate a new rolling checksum based on a previously calculated checksum for a slice of
    /// length `slice_length`. The new checksum will be for a slice of same length with
    /// `old_prefix` byte removed from the beginning of the old slice and `new_slice` byte
    /// appended at its end.
    ///
    /// The algorithm treats slice lengths modulo 2¹⁶.
    pub fn update(&mut self, slice_length: u16, old_prefix: u8, new_suffix: u8) {
        self.a += new_suffix as u16;
        self.a -= old_prefix as u16;
        self.b += self.a;
        self.b -= slice_length * old_prefix as u16;
    }

    fn to_u32(&self) -> u32 {
        (self.b.0 as u32) << 16 | (self.a.0 as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weak_sum_from() {
        assert_eq!(WeakSum::from(&[1][..]).to_u32(), 0x00010001);
        assert_eq!(WeakSum::from(&[1, 2][..]).to_u32(), 0x00040003);
    }

    #[test]
    fn test_weak_sum_update() {
        let mut weak_sum = WeakSum::from(&[1, 2][..]);
        weak_sum.update(2, 1, 3);
        assert_eq!(weak_sum.to_u32(), WeakSum::from(&[2, 3][..]).to_u32());
    }
}
