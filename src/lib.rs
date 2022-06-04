use std::num::Wrapping;

// 1. calc checksum for a slice

// https://rsync.samba.org/tech_report/node3.html
#[derive(Copy, Clone, Debug)]
struct WeakSum {
    a: Wrapping<u16>,
    b: Wrapping<u16>,
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
    // NOTE: slice_length: u16
    fn update(&mut self, slice_length: u16, old_prefix: u8, new_suffix: u8) {
        self.a += new_suffix as u16 - old_prefix as u16;
        self.b += self.a;
        self.b -= slice_length * old_prefix as u16;
    }

    fn to_u32(&self) -> u32 {
        (self.b.0 as u32) << 16 | (self.a.0 as u32)
    }
}

// fn update_weak_sum(slice_length: u64, old_sum: u32, old_prefix: u8, new_suffix: u8) -> u32 {
//     let old_a = Wrapping(old_sum as u16);
//     let old_b = Wrapping((old_sum >> 16) as u16);
//     let new_a = old_a - old_prefix + new_suffix;
//     let new_b = old_b - (slice_length) * old_prefix + new_a;
// }

struct SignatureEntry {
    pub weak: u32,
    pub strong: u32,
}

// u16 << 16 | u16 ---> u32
// b(k,l) << 16 | a(k,l)

// // TODO: what type to return?
// // TODO: doc
// pub fn signature(data: impl io::Read) -> Vec<SignatureEntry> {
//     vec![]
// }

// fn read_n<R>(reader: R, bytes_to_read: u64) ->
// https://stackoverflow.com/a/30413877/98528

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weak_sum() {
        assert_eq!(WeakSum::from(&[1][..]).to_u32(), 0x00010001);
        assert_eq!(WeakSum::from(&[1, 2][..]).to_u32(), 0x00040003);
    }

    /*
    #[test]
    fn simple_signature() {
        let sig = signature("abc");
        assert_eq!(sig[0].weak, 123);
        assert_eq!(sig[0].strong, 123);
    }
    */

    // TODO: more tests
}
