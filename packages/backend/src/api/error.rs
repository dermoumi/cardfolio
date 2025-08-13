use std::{fmt::Display, result};

use axum::{
    Json,
    body::Body,
    extract::rejection::PathRejection,
    http::{Response, StatusCode},
    response::IntoResponse,
};
use serde::{Serialize, ser::SerializeMap};
use serde_json::Value;

/// Shortcut for the Result types
pub type ApiResult<T, E = ApiError> = result::Result<T, E>;

/// API error types
#[derive(thiserror::Error, Debug, Serialize)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum ApiError {
    #[error("Resource {resource} not found")]
    NotFound { resource: Value },

    #[error(transparent)]
    #[serde(serialize_with = "a_message", rename = "path_error")]
    PathRejection(#[from] PathRejection),

    #[error(transparent)]
    #[serde(serialize_with = "a_message", rename = "query_error")]
    QueryRejection(#[from] axum::extract::rejection::QueryRejection),

    #[error("Cannot parse pagination cursor: {0}")]
    #[serde(serialize_with = "a_message", rename = "query_error")]
    InvalidPaginationCursor(String),

    #[error(transparent)]
    #[serde(serialize_with = "no_content", rename = "database_error")]
    Postgres(#[from] tokio_postgres::Error),

    #[error(transparent)]
    #[serde(serialize_with = "no_content", rename = "database_error")]
    Bb8(#[from] bb8::RunError<tokio_postgres::Error>),

    #[error(transparent)]
    #[serde(serialize_with = "no_content", rename = "internal_error")]
    Anyhow(#[from] anyhow::Error),
}

impl ApiError {
    /// Returns the HTTP status code for the error
    pub fn status_code(&self) -> StatusCode {
        match self {
            ApiError::NotFound { .. } => StatusCode::NOT_FOUND,
            ApiError::PathRejection(_) => StatusCode::BAD_REQUEST,
            ApiError::QueryRejection(_) => StatusCode::BAD_REQUEST,
            ApiError::InvalidPaginationCursor(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// Axum allows returning errors as long as they implement IntoResponse
impl IntoResponse for ApiError {
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
    use serde::Deserialize;

    use super::*;
    use crate::{
        api::{Path, Query},
        test_utils::*,
    };

    #[test]
    fn test_serialize_anyhow_errors() {
        let error = ApiError::Anyhow(anyhow::anyhow!("Test error"));
        let json = serde_json::to_string(&error).unwrap();
        assert_eq!(json, r#"{"error":"internal_error"}"#);
    }

    #[tokio::test]
    async fn test_app_http_response() {
        with_app_state(async move |state| {
            let router = Router::new()
                .route(
                    "/test",
                    get(async || -> ApiResult<()> {
                        Err(ApiError::Anyhow(anyhow::anyhow!("Test error")))
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

    #[tokio::test]
    async fn test_not_found_error() {
        with_app_state(async move |state| {
            let router = Router::new()
                .route(
                    "/test",
                    get(async || -> ApiResult<()> {
                        Err(ApiError::NotFound {
                            resource: 42.into(),
                        })
                    }),
                )
                .with_state(state.as_ref().clone());

            let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

            let response = router.oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::NOT_FOUND);

            let body = response.into_body().collect().await.unwrap().to_bytes();
            assert_eq!(body, r#"{"error":"not_found","resource":42}"#);
        })
        .await;
    }

    #[tokio::test]
    async fn test_path_rejection_error() {
        with_app_state(async move |state| {
            let router = Router::new()
                .route(
                    "/test/{id}",
                    get(async |Path(_): Path<i32>| -> ApiResult<()> { Ok(()) }),
                )
                .with_state(state.as_ref().clone());

            let request = Request::builder()
                .uri("/test/not_i32")
                .body(Body::empty())
                .unwrap();

            let response = router.oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);

            let body = response.into_body().collect().await.unwrap().to_bytes();
            assert_eq!(
                body,
                r#"{"error":"path_error","message":"Cannot parse `not_i32` to a `i32`"}"#
            );
        })
        .await;
    }

    #[tokio::test]
    async fn test_query_rejection_error() {
        with_app_state(async move |state| {
            #[derive(Debug, Deserialize)]
            pub struct Pagination {
                #[allow(dead_code)]
                pub page: Option<u32>,
            }

            let router = Router::new()
                .route(
                    "/test",
                    get(async |Query(_): Query<Pagination>| -> ApiResult<()> { Ok(()) }),
                )
                .with_state(state.as_ref().clone());

            let request = Request::builder()
                .uri("/test?page=invalid")
                .body(Body::empty())
                .unwrap();

            let response = router.oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);

            let body = response.into_body().collect().await.unwrap().to_bytes();
            assert_eq!(
                body,
                r#"{"error":"query_error","message":"Failed to deserialize query string: page: invalid digit found in string"}"#
            );
        })
        .await;
    }

    #[tokio::test]
    async fn test_invalid_pagination_cursor() {
        with_app_state(async move |state| {
            let router = Router::new()
                .route(
                    "/test",
                    get(async || -> ApiResult<()> {
                        Err(ApiError::InvalidPaginationCursor(
                            "not a cursor".to_string(),
                        ))
                    }),
                )
                .with_state(state.as_ref().clone());
            let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

            let response = router.oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);

            let body = response.into_body().collect().await.unwrap().to_bytes();
            assert_eq!(body, r#"{"error":"query_error","message":"not a cursor"}"#);
        })
        .await;
    }
}
