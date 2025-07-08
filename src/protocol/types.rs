impl Default for AuthCredentials {
    fn default() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            token: None,
            metadata: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::types::{CompressionFormat, EncryptionFormat};

    #[test]
    fn test_compression_format_default() {
        assert_eq!(CompressionFormat::default(), CompressionFormat::None);
    }

    #[test]
    fn test_encryption_format_default() {
        assert_eq!(EncryptionFormat::default(), EncryptionFormat::None);
    }
} 