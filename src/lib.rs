use std::num::Wrapping;

// 1. calc checksum for a slice

fn weak_sum(buf: &[u8]) -> u32 {
    // https://rsync.samba.org/tech_report/node3.html
    let l = buf.len();
    let mut a = Wrapping(0u16);
    let mut b = Wrapping(0u16);
    for (i, byte) in buf.iter().enumerate() {
        a += *byte as u16;
        b += ((l - i) as u16) * (*byte as u16);
    }
    (b.0 as u32) << 16 | (a.0 as u32)
}

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
        assert_eq!(weak_sum(&[1]), 0x00010001);
        assert_eq!(weak_sum(&[1, 2]), 0x00040003);
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
