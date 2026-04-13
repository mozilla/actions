//! CRC-32 hashing utilities (requires the `hashing` feature).

/// Compute the CRC-32 checksum of the given bytes.
pub fn crc32(data: &[u8]) -> u32 {
    crc32fast::hash(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc32_empty() {
        assert_eq!(crc32(b""), 0);
    }

    #[test]
    fn test_crc32_hello() {
        // Known CRC-32 of "hello".
        assert_eq!(crc32(b"hello"), 0x3610_a686);
    }
}
