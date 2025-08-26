use axum::extract::FromRequestParts;
use base64::{Engine as _, engine::general_purpose};
use serde::{Serialize, de::DeserializeOwned};

use super::ApiError;
use crate::api::ApiResult;

/// Custom Path extractor with error handling
pub struct Path<T>(pub T);

impl<S, T> FromRequestParts<S> for Path<T>
where
    T: DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let value = axum::extract::Path::<T>::from_request_parts(parts, state)
            .await
            .map_err(Self::Rejection::PathRejection)?;

        Ok(Self(value.0))
    }
}

/// Custom Query extractor with error handling
pub struct Query<T>(pub T);

impl<S, T> FromRequestParts<S> for Query<T>
where
    T: DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let query = parts.uri.query().unwrap_or_default();

        let deserializer =
            serde_html_form::Deserializer::new(form_urlencoded::parse(query.as_bytes()));
        let value = serde_path_to_error::deserialize(deserializer)
            .map_err(Self::Rejection::QueryRejection)?;

        Ok(Self(value))
    }
}

pub fn encode_pagination_cursor<T: Serialize>(cursor: &T) -> ApiResult<String> {
    let json = serde_json::to_string(cursor).map_err(anyhow::Error::from)?;
    Ok(general_purpose::URL_SAFE_NO_PAD.encode(json))
}

pub fn decode_pagination_cursor<T: DeserializeOwned>(cursor: &str) -> ApiResult<T> {
    let json = general_purpose::URL_SAFE_NO_PAD
        .decode(cursor)
        .map_err(|error| {
            ApiError::InvalidPaginationCursor(format!("Failed to parse cursor: {error}"))
        })?;
    let decoded = serde_json::from_slice(&json).map_err(|error| {
        ApiError::InvalidPaginationCursor(format!("Failed to parse cursor: {error}"))
    })?;
    Ok(decoded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestCursor {
        id: i32,
        sort: String,
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let cursor = TestCursor {
            id: 42,
            sort: "name".to_string(),
        };
        let encoded = encode_pagination_cursor(&cursor).expect("encode");
        let decoded: TestCursor = decode_pagination_cursor(&encoded).expect("decode");
        assert_eq!(cursor, decoded);
    }

    #[test]
    fn test_decode_invalid_base64() {
        let result: ApiResult<i32> = decode_pagination_cursor("not_base64!!");
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_invalid_json() {
        // valid base64, but not valid JSON for i32
        let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode("not a json");
        let result: ApiResult<i32> = decode_pagination_cursor(&encoded);
        assert!(result.is_err());
    }
}
