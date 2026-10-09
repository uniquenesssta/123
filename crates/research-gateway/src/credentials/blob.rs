#![cfg(any(windows, test))]

use super::error::missing_key;
use crate::GatewayError;
use zeroize::Zeroizing;

pub(super) fn decode_credential_blob(bytes: &[u8]) -> Result<String, GatewayError> {
    if looks_like_utf16le(bytes) {
        let units = Zeroizing::new(
            bytes
                .chunks_exact(2)
                .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
                .take_while(|unit| *unit != 0)
                .collect::<Vec<u16>>(),
        );
        if let Ok(value) = String::from_utf16(&units) {
            let value = Zeroizing::new(value);
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Ok(trimmed.to_string());
            }
        }
    }
    if let Ok(value) = std::str::from_utf8(bytes) {
        let trimmed = value.trim_matches(char::from(0)).trim();
        if !trimmed.is_empty() && !trimmed.contains(char::from(0)) {
            return Ok(trimmed.to_string());
        }
    }
    if bytes.len() % 2 == 0 {
        let units = Zeroizing::new(
            bytes
                .chunks_exact(2)
                .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
                .take_while(|unit| *unit != 0)
                .collect::<Vec<u16>>(),
        );
        if let Ok(value) = String::from_utf16(&units) {
            let value = Zeroizing::new(value);
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Ok(trimmed.to_string());
            }
        }
    }
    Err(missing_key(
        "Windows凭据管理器中的OpenAI密钥不是有效UTF-8或UTF-16LE文本",
    ))
}

fn looks_like_utf16le(bytes: &[u8]) -> bool {
    bytes.len() >= 2
        && bytes.len() % 2 == 0
        && bytes
            .chunks_exact(2)
            .take(8)
            .any(|chunk| chunk[1] == 0 && chunk[0] != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credential_blob_accepts_utf8_and_utf16le() {
        assert_eq!(
            decode_credential_blob(b"fixture-value").expect("utf8"),
            "fixture-value"
        );
        let utf16: Vec<u8> = "fixture-value"
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect();
        assert_eq!(
            decode_credential_blob(&utf16).expect("utf16"),
            "fixture-value"
        );
    }

    #[test]
    fn terminators_trimming_and_utf16_fallback_keep_original_results() {
        assert_eq!(
            decode_credential_blob(b"\0  fixture-value \0").expect("utf8 padding"),
            "fixture-value"
        );
        let utf16: Vec<u8> = "  fixture-value \0ignored-after-terminator"
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect();
        assert_eq!(
            decode_credential_blob(&utf16).expect("utf16 padding"),
            "fixture-value"
        );
        let text = "\u{4f80}\u{5980}";
        let unicode: Vec<u8> = text.encode_utf16().flat_map(u16::to_le_bytes).collect();
        assert_eq!(
            decode_credential_blob(&unicode).expect("utf16 fallback"),
            text
        );
    }

    #[test]
    fn malformed_or_empty_blob_keeps_exact_safe_missing_credential_error() {
        for bytes in [
            &b""[..],
            &b"\0\0"[..],
            &b" \t\n"[..],
            &b"\x80"[..],
            &b"\0\xd8"[..],
            &b"a\0b"[..],
        ] {
            let error = decode_credential_blob(bytes).expect_err("malformed");
            assert_eq!(
                error,
                missing_key("Windows凭据管理器中的OpenAI密钥不是有效UTF-8或UTF-16LE文本")
            );
            assert!(error.provider_status.is_none());
            assert!(error.provider_code.is_none());
        }
    }
}
