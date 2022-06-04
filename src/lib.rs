use md4::{Digest, Md4};
use std::collections::HashMap;
use std::io::{self, Read};
use std::num::Wrapping;
use std::ops::Deref;

// TODO[LATER]: make it a parameter
const BLOCK_SIZE: u16 = 1024;

pub fn signature(r: impl Read) -> impl Iterator<Item = io::Result<BlockSignature>> {
    // TODO: do we have to calc signature for the last smaller block too? sounds risky & tricky & not worth it
    Chunker::new(BLOCK_SIZE, r)
        .map(|result| result.map(|block| BlockSignature::from(block.deref())))
}

pub fn diff<S, D>(signature: S, data: D) -> io::Result<Vec<Command>>
where
    S: Iterator<Item = BlockSignature>,
    D: Read,
{
    use Command::*;
    type BlockInfoMap = HashMap<WeakSum, DiffBlockInfo>;
    let blocks = BlockInfoMap::from_iter(signature.enumerate().map(|(i, signature)| {
        (
            signature.weak,
            DiffBlockInfo {
                signature,
                block_index: i,
            },
        )
    }));
    let mut commands = vec![];
    let mut data_bytes = data.bytes();
    let mut buf = vec![];
    let mut weak_sum = None;
    loop {
        let byte = match data_bytes.next() {
            Some(Ok(byte)) => byte,
            Some(Err(err)) => return Err(err),
            None => {
                // EOF, per io::Read::bytes() docs.
                commands.push(Raw { data: buf });
                return Ok(commands);
            }
        };
        // TODO[LATER]: enum for func state
        buf.push(byte);
        const BLOCK_SIZE_: usize = BLOCK_SIZE as usize;
        const BLOCK_SIZE_MINUS_1: usize = BLOCK_SIZE_ - 1;
        match buf.len() {
            0..=BLOCK_SIZE_MINUS_1 => continue,
            BLOCK_SIZE_ => {
                weak_sum = Some(WeakSum::from(&*buf));
            }
            n @ _ => {
                weak_sum
                    .unwrap()
                    .update(BLOCK_SIZE, buf[n - BLOCK_SIZE_ - 1], byte);
            }
        }
        if let Some(block_info) = blocks.get(&weak_sum.unwrap()) {
            let block_begin = buf.len() - BLOCK_SIZE_;
            let strong_sum = Md4::digest(&buf[block_begin..]);
            if strong_sum != block_info.signature.strong {
                continue;
            }
            buf.truncate(block_begin);
            if block_begin > 0 {
                commands.push(Raw {
                    data: std::mem::replace(&mut buf, vec![]),
                });
            }
            commands.push(CopyBlock {
                index: block_info.block_index,
            });
        }
    }
}

struct DiffBlockInfo {
    block_index: usize,
    signature: BlockSignature,
}

// TODO[LATER]: move to submodule
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
        match limited_reader.read_to_end(&mut buf) {
            Err(err) => Some(Err(err)),
            Ok(n) if n == self.chunk_size as usize => Some(Ok(buf)),
            Ok(_) => None,
        }
    }
}

pub enum Command {
    Raw { data: Vec<u8> },
    CopyBlock { index: usize },
}

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

// TODO: verify with rdiff
// https://rsync.samba.org/tech_report/node3.html
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
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
