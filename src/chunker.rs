use std::io::{self, Read};

pub struct Chunker<R: Read> {
    chunk_size: u16,
    source: R,
}

impl<R: Read> Chunker<R> {
    pub fn new(chunk_size: u16, source: R) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
