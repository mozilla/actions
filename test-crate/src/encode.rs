//! Base64 encoding utilities (requires the `encoding` feature).

use base64::Engine as _;

/// Encode the given bytes as a standard base64 string.
pub fn encode_base64(data: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_empty() {
        assert_eq!(encode_base64(b""), "");
    }

    #[test]
    fn test_encode_hello() {
        assert_eq!(encode_base64(b"hello"), "aGVsbG8=");
    }
}
