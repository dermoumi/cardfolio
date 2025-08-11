use axum::{Json, extract::State, response::IntoResponse};

use crate::api::{ApiError, ApiResult, Path};
use crate::prelude::AppState;
use crate::services::ygo as service;

/// Lists yugioh cards
pub async fn get_cards(State(state): State<AppState>) -> ApiResult<impl IntoResponse> {
    let client = state.db.get().await?;

    let cards = service::card::get_all(&client).await?;

    Ok(Json(cards).into_response())
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

    let cards = service::card::import_sample_cards(&client).await?;

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
                service::card::import_sample_cards(&client)
                    .await
                    .expect("seed")
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
            let retrieved_cards: Vec<ygo::Card> =
                serde_json::from_slice(&body).expect("Unable to parse response body");
            assert_eq!(retrieved_cards.len(), added_cards.len());
        })
        .await
    }

    #[tokio::test]
    async fn test_get_by_id() {
        with_app_state(async move |state| {
            // Seed sample cards
            {
                let client = state.db.get().await.expect("db");
                service::card::import_sample_cards(&client)
                    .await
                    .expect("seed")
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
            let body = String::from_utf8(body.to_vec()).expect("Unable to parse response body");
            assert_eq!(body, r#"{"error":"not_found","resource":144}"#);
        })
        .await
    }
}
