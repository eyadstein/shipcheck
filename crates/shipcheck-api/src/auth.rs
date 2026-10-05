//! API key check for write endpoints.

use axum::http::HeaderMap;

use crate::error::ApiError;

/// Header that carries the API key.
pub(crate) const KEY_HEADER: &str = "x-api-key";

/// Accepts the request only when its key header matches `expected`.
pub(crate) fn authorize(headers: &HeaderMap, expected: &str) -> Result<(), ApiError> {
    let given = headers
        .get(KEY_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    if !expected.is_empty() && constant_time_eq(given.as_bytes(), expected.as_bytes()) {
        Ok(())
    } else {
        Err(ApiError::Unauthorized)
    }
}

/// Compares two byte strings without stopping at the first difference.
fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right)
        .fold(0_u8, |diff, (a, b)| diff | (a ^ b))
        == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headers(key: Option<&str>) -> HeaderMap {
        let mut map = HeaderMap::new();
        if let Some(key) = key {
            map.insert(KEY_HEADER, key.parse().expect("valid header value"));
        }
        map
    }

    #[test]
    fn correct_key_is_accepted() {
        assert!(authorize(&headers(Some("secret-key")), "secret-key").is_ok());
    }

    #[test]
    fn missing_key_is_rejected() {
        assert!(matches!(
            authorize(&headers(None), "secret-key"),
            Err(ApiError::Unauthorized)
        ));
    }

    #[test]
    fn wrong_or_longer_keys_are_rejected() {
        assert!(authorize(&headers(Some("secret-kez")), "secret-key").is_err());
        assert!(authorize(&headers(Some("secret-key-and-more")), "secret-key").is_err());
    }

    #[test]
    fn empty_expected_key_never_matches() {
        assert!(authorize(&headers(None), "").is_err());
    }
}
