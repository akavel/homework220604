mod chunker;
mod signature;
mod weak_sum;

pub use signature::*;
use weak_sum::WeakSum;

use md4::{Digest, Md4};
use std::collections::HashMap;
use std::io::{self, Read};

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
    let mut weak_sum = WeakSum::default();
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
        buf.push(byte);
        let buf_len = buf.len();
        if buf_len < BLOCK_SIZE as usize {
            continue;
        } else if buf_len == BLOCK_SIZE as usize {
            weak_sum = WeakSum::from(&*buf);
        } else {
            let old_byte = buf[buf_len - BLOCK_SIZE as usize - 1];
            weak_sum.update(BLOCK_SIZE, old_byte, byte);
        }
        if let Some(block_info) = block_map.get(&weak_sum) {
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

#[derive(PartialEq, Debug)]
pub enum Command {
    Raw { data: Vec<u8> },
    CopyBlock { index: usize },
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
