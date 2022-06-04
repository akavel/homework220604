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

    fn signature_4<R: Read>(r: R) -> impl Iterator<Item = io::Result<signature::BlockSignature>> {
        signature::<R, 4>(r)
    }

    fn delta_4<S, D>(signatures: S, data: D) -> io::Result<Vec<delta::Command>>
    where
        S: IntoIterator<Item = signature::BlockSignature>,
        D: Read,
    {
        delta::<S, D, 4>(signatures, data)
    }

    #[test]
    fn test_delta() {
        use delta::Command::*;
        let old_file = vec![1, 2, 3, 4, 10, 20, 30, 40];
        let new_file = vec![0, 1, 10, 20, 30, 40, 99, 1, 2, 3, 4, 55];
        let signature: Vec<_> = signature_4(&*old_file)
            .map(|result| result.unwrap())
            .collect();
        let delta = delta_4(signature, &*new_file).unwrap();
        assert_eq!(
            delta,
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
