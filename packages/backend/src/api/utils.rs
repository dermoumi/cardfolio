use axum::extract::FromRequestParts;
use serde::de::DeserializeOwned;

use super::ApiError;

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
