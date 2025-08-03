use std::{env, ops::Deref, panic, sync::Arc};

use tracing::level_filters::LevelFilter;

use crate::{database, migrations, prelude::*};

use futures_util::FutureExt;
pub use http_body_util::BodyExt;
pub use tower::ServiceExt; // Import for .oneshot() // Import for .catch_unwind()

/// Utility to create a database connection pool for testing
pub async fn with_db_pool<Fn, Fut>(f: Fn)
where
    Fn: FnOnce(Arc<database::Pool>) -> Fut,
    Fut: Future<Output = ()>,
{
    let db_url = env::var("CARDFOLIO_DB_TEST").expect("CARDFOLIO_DB_TEST must be set");
    let db_pool = database::init(&db_url, 1)
        .await
        .expect("Failed to create test DB pool");

    database::Migrate::new("migrations")
        .run(&db_pool, migrations::MIGRATIONS)
        .await
        .expect("Failed to run migrations");

    // Since we're using just a single connection in our pool
    // We can wrap the test in a transaction to ensure that
    // the database is in a clean state before and after the test
    //
    // When borrowing the connection from the pool, we need to ensure
    // that the connection is returned to the pool before the test.
    {
        db_pool
            .get()
            .await
            .expect("Failed to get database connection")
            .execute("BEGIN;", &[])
            .await
            .expect("Failed to begin transaction");
    }

    let db = Arc::new(db_pool.clone());
    let res = panic::AssertUnwindSafe(f(db)).catch_unwind().await;

    db_pool
        .get()
        .await
        .expect("Failed to get database connection")
        .execute("ROLLBACK;", &[])
        .await
        .expect("Failed to rollback transaction");

    if let Err(panic) = res {
        panic::resume_unwind(panic);
    }
}

/// Utility to create an AppState for testing
pub async fn with_app_state<Fn, Fut>(f: Fn)
where
    Fn: FnOnce(Arc<AppState>) -> Fut,
    Fut: Future<Output = ()>,
{
    with_db_pool(async move |db_pool| {
        let config = AppConfig {
            log_level: LevelFilter::ERROR,
            port: "8000".to_string(),
            db_url: env::var("CARDFOLIO_DB_TEST").expect("CARDFOLIO_DB_TEST must be set"),
            db_pool_size: 1,
            frontend_dir: "frontend/".to_string(),
        };

        let db = db_pool.deref().clone();

        // Run the wrapped function
        let state = Arc::new(AppState { config, db });
        f(state).await
    })
    .await
}
