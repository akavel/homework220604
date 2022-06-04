use md4::{Digest, Md4};
use std::collections::HashMap;
use std::io::{self, Read};
use std::num::Wrapping;
use std::ops::Deref;

// TODO[LATER]: make it a parameter
const DEFAULT_BLOCK_SIZE: u16 = 1024;

pub fn signature<R, const BLOCK_SIZE: u16>(r: R) -> impl Iterator<Item = io::Result<BlockSignature>>
where
    R: Read,
{
    // TODO: do we have to calc signature for the last smaller block too? sounds risky & tricky & not worth it
    Chunker::new(BLOCK_SIZE, r)
        .map(|result| result.map(|block| BlockSignature::from(block.deref())))
}

pub fn diff<S, D, const BLOCK_SIZE: u16>(signatures: S, data: D) -> io::Result<Vec<Command>>
where
    S: Iterator<Item = BlockSignature>,
    D: Read,
{
    use Command::*;
    let block_map = block_map_from_signatures(signatures);
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
        let buf_len = buf.len();
        if buf_len < BLOCK_SIZE as usize {
            continue;
        } else if buf_len == BLOCK_SIZE as usize {
            weak_sum = Some(WeakSum::from(&*buf));
        } else {
            let old_byte = buf[buf_len - BLOCK_SIZE as usize - 1];
            weak_sum.unwrap().update(BLOCK_SIZE, old_byte, byte);
        }
        if let Some(block_info) = block_map.get(&weak_sum.unwrap()) {
            let block_begin = buf.len() - BLOCK_SIZE as usize;
            let digest = Md4::digest(&buf[block_begin..]);
            if digest != block_info.digest {
                continue;
            }
            buf.truncate(block_begin);
            if block_begin > 0 {
                commands.push(Raw {
                    data: std::mem::replace(&mut buf, vec![]),
                });
            }
            commands.push(CopyBlock {
                index: block_info.index,
            });
        }
    }
}

type BlockMap = HashMap<WeakSum, BlockInfo>;

struct BlockInfo {
    index: usize,
    digest: Md4Digest,
}

impl BlockInfo {
    fn new(index: usize, digest: Md4Digest) -> Self {
        Self { index, digest }
    }
}

fn block_map_from_signatures(signatures: impl Iterator<Item = BlockSignature>) -> BlockMap {
    signatures
        .enumerate()
        .map(|(index, signature)| (signature.weak, BlockInfo::new(index, signature.strong)))
        .collect()
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

#[derive(PartialEq, Debug)]
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
        self.a += new_suffix as u16;
        self.a -= old_prefix as u16;
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

    fn signature_4<R: Read>(r: R) -> impl Iterator<Item = io::Result<BlockSignature>> {
        signature::<R, 4>(r)
    }

    fn diff_4<S, D>(signatures: S, data: D) -> io::Result<Vec<Command>>
    where
        S: Iterator<Item = BlockSignature>,
        D: Read,
    {
        diff::<S, D, 4>(signatures, data)
    }

    #[test]
    fn test_diff() {
        use Command::*;
        let old_file = vec![1, 2, 3, 4, 10, 20, 30, 40];
        let new_file = vec![0, 1, 10, 20, 30, 40, 99, 1, 2, 3, 4, 55];
        let signature: Vec<_> = signature_4(&*old_file)
            .map(|result| result.unwrap())
            .collect();
        let diff = diff_4(signature.into_iter(), &*new_file).unwrap();
        assert_eq!(
            diff,
            [
                Raw { data: vec![0, 1] },
                CopyBlock { index: 1 },
                Raw { data: vec![99] },
                CopyBlock { index: 0 },
                Raw { data: vec![55] },
            ]
        );
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
