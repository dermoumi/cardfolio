use std::fmt::Display;

use axum::{
    Json,
    body::Body,
    extract::rejection::PathRejection,
    http::{Response, StatusCode},
    response::IntoResponse,
};
use serde::{Serialize, ser::SerializeMap};

#[derive(thiserror::Error, Debug, Serialize)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum AppError {
    #[error(transparent)]
    #[serde(serialize_with = "a_message", rename = "path_error")]
    PathRejection(#[from] PathRejection),

    #[error(transparent)]
    #[serde(serialize_with = "no_content", rename = "internal_error")]
    Anyhow(#[from] anyhow::Error),
}

impl AppError {
    /// Returns the HTTP status code for the error
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::PathRejection(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// Axum allows returning errors as long as they implement IntoResponse
impl IntoResponse for AppError {
    fn into_response(self) -> Response<Body> {
        // Log the error
        tracing::error!("{self}");
        tracing::debug!("{self:?}");

        // Return the error as JSON
        (self.status_code(), Json(self)).into_response()
    }
}

/// Serialize the error with no content.
fn no_content<T, S>(_: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_map(Some(0))?.end()
}

/// Serialize an error with a string message
fn a_message<T, S>(entry: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: Display,
    S: serde::Serializer,
{
    let mut map = serializer.serialize_map(Some(1))?;
    map.serialize_entry("message", &entry.to_string())?;
    map.end()
}

#[cfg(test)]
mod tests {
    use axum::{Router, http::Request, routing::get};

    use super::*;
    use crate::prelude::*;
    use crate::test_utils::*;

    #[test]
    fn test_serialize_anyhow_errors() {
        let error = AppError::Anyhow(anyhow::anyhow!("Test error"));
        let json = serde_json::to_string(&error).unwrap();
        assert_eq!(json, r#"{"error":"internal_error"}"#);
    }

    #[tokio::test]
    async fn test_app_http_response() {
        with_app_state(async move |state| {
            let router = Router::new()
                .route(
                    "/test",
                    get(async || -> Result<()> {
                        Err(AppError::Anyhow(anyhow::anyhow!("Test error")))
                    }),
                )
                .with_state(state.as_ref().clone());

            let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

            let response = router.oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

            let body = response.into_body().collect().await.unwrap().to_bytes();
            assert_eq!(body, r#"{"error":"internal_error"}"#);
        })
        .await;
    }
}
