use futures_util::FutureExt;
use std::panic;
use tokio_postgres::Client;

pub async fn with_advisory_lock<'a, R, E, F, Fut>(
    db: &'a Client,
    lock_id: &str,
    f: F,
) -> Result<R, E>
where
    E: From<tokio_postgres::Error>,
    F: FnOnce(&'a Client) -> Fut,
    Fut: Future<Output = Result<R, E>>,
{
    // Acquire the advisory lock
    db.execute("SELECT pg_advisory_lock(hashtext($1))", &[&lock_id])
        .await
        .map_err(|e| e.into())?;

    // Run the function and save the result
    let result = panic::AssertUnwindSafe(f(db)).catch_unwind().await;

    // Release the advisory lock
    db.execute("SELECT pg_advisory_unlock(hashtext($1))", &[&lock_id])
        .await
        .map_err(|e| e.into())?;

    // Forward the result or panic
    match result {
        Ok(res) => res,
        Err(panic) => panic::resume_unwind(panic),
    }
}
