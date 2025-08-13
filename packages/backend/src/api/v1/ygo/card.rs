use axum::{Json, extract::State, response::IntoResponse};

use crate::api::utils::{decode_pagination_cursor, encode_pagination_cursor};
use crate::api::{ApiError, ApiResult, Path, Query};
use crate::prelude::AppState;
use crate::services::ygo as service;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Pagination {
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Page {
    pub cards: Vec<crate::models::ygo::Card>,
    pub next: Option<String>,
}

/// Lists yugioh cards (paginated)
pub async fn get_cards(
    State(state): State<AppState>,
    Query(pagination): Query<Pagination>,
) -> ApiResult<impl IntoResponse> {
    let client = state.db.get().await?;

    let limit = pagination.limit.unwrap_or(100).min(100);
    let cursor = pagination
        .cursor
        .as_ref()
        .map(|c| decode_pagination_cursor(c))
        .transpose()?;

    let (cards, next_cursor) = service::card::get_page(&client, limit, cursor).await?;

    let as_page = Page {
        cards,
        next: next_cursor
            .as_ref()
            .map(encode_pagination_cursor)
            .transpose()?,
    };

    Ok(Json(as_page).into_response())
}

/// Get card by ID
pub async fn get_by_id(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> ApiResult<impl IntoResponse> {
    let client = state.db.get().await?;

    let card = service::card::get_by_id(&client, id)
        .await?
        .ok_or(ApiError::NotFound {
            resource: id.into(),
        })?;

    Ok(Json(card).into_response())
}

/// Import yugioh cards
pub async fn import(State(state): State<AppState>) -> ApiResult<impl IntoResponse> {
    let client = state.db.get().await?;

    let cards = service::card::seed_cards(&client, 80).await?;

    Ok(Json(cards).into_response())
}

#[cfg(test)]
mod tests {
    use crate::models::ygo;
    use crate::test_utils::*;

    use super::*;
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
        routing::get,
    };

    #[tokio::test]
    async fn test_get_cards() {
        with_app_state(async move |state| {
            // Seed sample cards
            let added_cards = {
                let client = state.db.get().await.expect("db");
                service::card::seed_cards(&client, 1).await.expect("seed")
            };

            let router = Router::new()
                .route("/ygo/cards", get(get_cards))
                .with_state(state.as_ref().clone());
            let request = Request::builder()
                .uri("/ygo/cards")
                .body(Body::empty())
                .unwrap();

            let response = router.oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);

            let body = response.into_body().collect().await.unwrap().to_bytes();
            let paginated: super::Page =
                serde_json::from_slice(&body).expect("Unable to parse response body");
            assert_eq!(paginated.cards.len(), added_cards.len());
        })
        .await
    }

    #[tokio::test]
    async fn test_get_by_id() {
        with_app_state(async move |state| {
            // Seed sample cards
            {
                let client = state.db.get().await.expect("db");
                service::card::seed_cards(&client, 1).await.expect("seed")
            };

            let router = Router::new()
                .route("/ygo/cards/{id}", get(get_by_id))
                .with_state(state.as_ref().clone());
            let request = Request::builder()
                .uri("/ygo/cards/1")
                .body(Body::empty())
                .unwrap();

            let response = router.oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);

            let body = response.into_body().collect().await.unwrap().to_bytes();
            let retrieved_card: ygo::Card =
                serde_json::from_slice(&body).expect("Unable to parse response body");
            assert_eq!(retrieved_card.id, 1);
            assert_eq!(retrieved_card.data.name, "Blue-Eyes White Dragon");
        })
        .await
    }

    #[tokio::test]
    async fn test_get_by_id_not_found() {
        with_app_state(async move |state| {
            let router = Router::new()
                .route("/ygo/cards/{id}", get(get_by_id))
                .with_state(state.as_ref().clone());
            let request = Request::builder()
                .uri("/ygo/cards/144")
                .body(Body::empty())
                .unwrap();

            let response = router.oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::NOT_FOUND);

            let body = response.into_body().collect().await.unwrap().to_bytes();
            assert_eq!(body, r#"{"error":"not_found","resource":144}"#);
        })
        .await
    }

    #[tokio::test]
    async fn test_get_cards_invalid_query() {
        with_app_state(async move |state| {
            let router = Router::new()
                .route("/ygo/cards", get(get_cards))
                .with_state(state.as_ref().clone());
            let request = Request::builder()
                .uri("/ygo/cards?limit=invalid")
                .body(Body::empty())
                .unwrap();

            let response = router.oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);

            let body = response.into_body().collect().await.unwrap().to_bytes();
            assert_eq!(
                body,
                r#"{"error":"query_error","message":"Failed to deserialize query string: limit: invalid digit found in string"}"#
            );
        })
        .await
    }
}
