//! Implementation of a file diffing algorithm based on a rolling hash. Based on a description of
//! the rsync algorithm used in the [rdiff](https://linux.die.net/man/1/rdiff) tool.
//!
//! See [`signature()`] and [`delta()`] for more details.

mod chunker;
mod delta;
mod signature;
mod weak_sum;

pub use delta::delta;
pub use signature::signature;

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::{self, Read};

    fn signature_4byte<R: Read>(
        r: R,
    ) -> impl Iterator<Item = io::Result<signature::BlockSignature>> {
        signature::<R, 4>(r)
    }

    fn delta_4byte<S, D>(signatures: S, data: D) -> io::Result<Vec<delta::Command>>
    where
        S: IntoIterator<Item = signature::BlockSignature>,
        D: Read,
    {
        delta::<S, D, 4>(signatures, data)
    }

    fn quick_delta_4byte(old_file: Vec<u8>, new_file: Vec<u8>) -> Vec<delta::Command> {
        let signature: Vec<_> = signature_4byte(&*old_file)
            .map(|result| result.unwrap())
            .collect();
        delta_4byte(signature, &*new_file).unwrap()
    }

    #[test]
    fn test_delta() {
        use delta::Command::*;
        let old_file = vec![1, 2, 3, 4, 10, 20, 30, 40];
        let new_file = vec![0, 1, 10, 20, 30, 40, 99, 1, 2, 3, 4, 55];
        let delta = quick_delta_4byte(old_file, new_file);
        let expected_delta = [
            Raw { data: vec![0, 1] },
            CopyBlock { index: 1 },
            Raw { data: vec![99] },
            CopyBlock { index: 0 },
            Raw { data: vec![55] },
        ];
        assert_eq!(delta, expected_delta);
    }

    #[test]
    fn test_delta_overlapping() {
        use delta::Command::*;
        let old_file = vec![1, 1, 1, 1];
        let new_file = vec![1, 1, 1, 1, 1];
        let delta = quick_delta_4byte(old_file, new_file);
        let expected_delta = [CopyBlock { index: 0 }, Raw { data: vec![1] }];
        assert_eq!(delta, expected_delta);
    }
}
