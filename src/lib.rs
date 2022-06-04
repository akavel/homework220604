use md4::{Digest, Md4};
use std::io::{self, Read};
use std::num::Wrapping;

pub struct Chunker<R: Read> {
    chunk_size: u16,
    source: R,
}

impl<R: Read> Chunker<R> {
    fn new(chunk_size: u16, source: R) -> Self {
        Self { chunk_size, source }
    }
}

impl<R: Read> Iterator for Chunker<R> {
    type Item = io::Result<Vec<u8>>;
    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = vec![];
        let mut limited_reader = self.source.by_ref().take(self.chunk_size as u64);
        // TODO[LATER]: test behavior on perfectly and imperfectly chunked inputs
        match limited_reader.read_to_end(&mut buf) {
            Err(err) => Some(Err(err)),
            Ok(n) if n == self.chunk_size as usize => Some(Ok(buf)),
            Ok(_) => None,
        }
    }
}

// pub fn signature(r: impl IntoIterator<Item = u8>) -> impl Iterator<Item =
// TODO[LATER]: fn (?) -> impl Iterator<Item = BlockSignature>
//   -> see e.g. "chunker" type idea: https://kevinhoffman.medium.com/creating-a-stream-chunking-iterator-in-rust-d4063ffd21ed
// pub fn signature(r: impl Read) -> io::Result<Vec<BlockSignature>> {
//     loop {

//     }
// }

#[derive(Debug)]
pub struct BlockSignature {
    pub weak: WeakSum,
    pub strong: Md4Digest,
}

type Md4Digest = digest::generic_array::GenericArray<u8, digest::typenum::U16>;

impl From<&[u8]> for BlockSignature {
    fn from(buf: &[u8]) -> Self {
        Self {
            weak: buf.into(),
            strong: Md4::digest(buf),
        }
    }
}

// // TODO[LATER]: iterator of sized chunks of an io::Read impl
// pub fn signature(r: impl io::Read) -> io::Result<Vec<SignatureEntry>> {
// }

// TODO: do we have to calc signature for the last smaller block too? sounds risky & tricky & not worth it

// TODO: verify with rdiff
// https://rsync.samba.org/tech_report/node3.html
#[derive(Copy, Clone, Debug)]
pub struct WeakSum {
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

    #[test]
    fn test_chunker() {
        let source = vec![1, 2, 3];

        let chunks_len_2: Vec<_> = Chunker::new(2, &*source)
            .map(|result| result.ok())
            .collect();
        assert_eq!(chunks_len_2, [Some(vec![1, 2])]);

        let chunks_len_3: Vec<_> = Chunker::new(3, &*source)
            .map(|result| result.ok())
            .collect();
        assert_eq!(chunks_len_3, [Some(vec![1, 2, 3])]);
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
