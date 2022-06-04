struct SignatureEntry {
    pub weak: u32,
    pub strong: u32,
}

// TODO: what type to return?
// TODO: doc
fn signature(data: &str) -> Vec<SignatureEntry> {
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_signature() {
        let sig = signature("abc");
        assert_eq!(sig[0].weak, 123);
        assert_eq!(sig[0].strong, 123);
    }

    // TODO: more tests
}
