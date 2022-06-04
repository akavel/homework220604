use crate::chunker::Chunker;
use crate::weak_sum::WeakSum;

use md4::{Digest, Md4};
use std::fmt;
use std::io::{self, Read};
use std::ops::Deref;

/// Splits data read from `r` into consecutive blocks of `BLOCK_SIZE` bytes. Calculates a
/// [`BlockSignature`] for each of the blocks. Any trailing block of smaller size is ignored.
pub fn signature<R, const BLOCK_SIZE: u16>(r: R) -> impl Iterator<Item = io::Result<BlockSignature>>
where
    R: Read,
{
    Chunker::new(BLOCK_SIZE, r)
        .map(|result| result.map(|block| BlockSignature::from(block.deref())))
}

pub struct BlockSignature {
    pub weak: WeakSum,
    pub strong: Md4Digest,
}

/// An [`md4::Md4`] hash digest.
pub type Md4Digest = digest::generic_array::GenericArray<u8, digest::typenum::U16>;

impl fmt::Debug for BlockSignature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let strong_in_hex = base16ct::lower::encode_string(&self.strong);
        write!(f, "[{} {:?}]", strong_in_hex, self.weak)
    }
}

impl From<&[u8]> for BlockSignature {
    fn from(buf: &[u8]) -> Self {
        Self {
            weak: buf.into(),
            strong: Md4::digest(buf),
        }
    }
}
