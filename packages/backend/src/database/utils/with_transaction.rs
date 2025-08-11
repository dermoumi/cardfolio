use futures_util::FutureExt;
use std::panic;
use std::{convert::From, result::Result};
use tokio_postgres::Client;

#[cfg(test)]
/// Generates a random alphanumeric string of the given length
pub fn random_string(length: usize) -> String {
    use rand::distr::{Alphanumeric, SampleString as _};

    Alphanumeric.sample_string(&mut rand::rng(), length)
}

/// Utility to run a function inside a database transaction.
/// Automatically rolls back if the function returns an error.
/// If a checkpoint is provided, it will be used instead of a transaction.
/// When testing, we'll assume that we're using a transaction and simulate checkpoints instead.
pub async fn with_transaction<'a, R, E, F, Fut>(
    db: &'a Client,
    checkpoint: Option<String>,
    f: F,
) -> Result<R, E>
where
    E: From<tokio_postgres::Error>,
    F: FnOnce(&'a Client) -> Fut,
    Fut: Future<Output = Result<R, E>>,
{
    #[allow(unused_mut)]
    let mut checkpoint = checkpoint;

    // When testing, we usually are inside a transaction already,
    // so we simulate a checkpoint instead
    #[cfg(test)]
    {
        use tokio_postgres::error::SqlState;

        let checkpoint_id = checkpoint.unwrap_or_else(|| format!("T{}", random_string(16)));
        let query = format!("SAVEPOINT {checkpoint_id}");
        checkpoint = Some(checkpoint_id);

        let result = db.execute(&query, &[]).await;
        if let Err(e) = result {
            if let Some(db_error) = e.as_db_error()
                && *db_error.code() == SqlState::NO_ACTIVE_SQL_TRANSACTION
            {
                // If we are not in a transaction, we start a transaction
                db.execute("BEGIN", &[]).await?;
                checkpoint = None; // No checkpoint in this case
            }
        }
    }

    #[cfg(not(test))]
    {
        let query = if let Some(checkpoint_id) = &checkpoint {
            format!("SAVEPOINT {checkpoint_id}")
        } else {
            "BEGIN".to_string()
        };

        db.execute(&query, &[]).await.map_err(|e| e.into())?;
    }

    // Run the function and save the result
    let result = panic::AssertUnwindSafe(f(db)).catch_unwind().await;

    let should_rollback = result.as_ref().map_or(true, |r| r.is_err());

    // Rollback if the function returned an error, commit otherwise
    if should_rollback {
        let query = if let Some(checkpoint_id) = &checkpoint {
            format!("ROLLBACK TO {checkpoint_id}")
        } else {
            "ROLLBACK".to_string()
        };

        db.execute(&query, &[]).await.map_err(|e| e.into())?;
    } else if checkpoint.is_none() {
        db.execute("COMMIT", &[]).await.map_err(|e| e.into())?;
    }

    // Forward the result or panic
    match result {
        Ok(res) => res,
        Err(panic) => panic::resume_unwind(panic),
    }
}

#[cfg(test)]
mod tests {

    use crate::test_utils::with_db_pool;

    use super::*;

    #[tokio::test]
    async fn test_with_transaction_commits() {
        with_db_pool(async |pool| {
            let client = pool.get().await.unwrap();

            // Create a table for testing
            client
                .execute(
                    "CREATE TABLE test_transaction_commit (id SERIAL PRIMARY KEY, name TEXT NOT NULL)",
                    &[],
                )
                .await
                .expect("Failed to create test table");

            // Insert a test row while in a transaction
            with_transaction(&client, None, async |client| {
                client
                    .execute("INSERT INTO test_transaction_commit (name) VALUES ($1)", &[&"Test"])
                    .await
            })
            .await
            .expect("Failed to insert row");

            // Verify the test row was inserted
            let row = client
                .query_one("SELECT name FROM test_transaction_commit WHERE id = $1", &[&1])
                .await;
            assert_eq!(row.unwrap().get::<_, String>("name"), "Test");
        })
        .await
    }

    #[tokio::test]
    async fn test_with_transaction_rolls_back() {
        with_db_pool(async |pool| {
            let client = pool.get().await.unwrap();

            // Create a table for test
            client
                .execute(
                    "CREATE TABLE test_transaction_rollback (id SERIAL PRIMARY KEY, name TEXT NOT NULL)",
                    &[],
                )
                .await
                .expect("Failed to create test table");

            // Insert a test row while in a transaction, but return an error
            with_transaction(&client, None, async |client| -> anyhow::Result<()> {
                client
                    .execute("INSERT INTO test_transaction_rollback (name) VALUES ($1)", &[&"Test"])
                    .await
                    .expect("Failed to insert row");

                anyhow::bail!("Simulated error to trigger rollback");
            })
            .await
            .unwrap_err();

            // Verify the test row was not inserted
            let row = client
                .query_opt("SELECT name FROM test_transaction_rollback WHERE id = $1", &[&1])
                .await
                .expect("Failed to query test row");
            assert!(row.is_none());
        })
        .await
    }

    #[tokio::test]
    async fn test_with_transaction_rolls_back_on_panic() {
        with_db_pool(async |pool| {
            let client = pool.get().await.unwrap();

            // Create a table for test
            client
                .execute(
                    "CREATE TABLE test_transaction_panic (id SERIAL PRIMARY KEY, name TEXT NOT NULL)",
                    &[],
                )
                .await
                .expect("Failed to create test table");

            // Insert a test row while in a transaction, but panic
            let result = panic::AssertUnwindSafe(with_transaction(
                &client,
                None,
                async |client| -> anyhow::Result<()> {
                    // To suppress verbose output in tests
                    panic::set_hook(Box::new(|_| {}));

                    client
                        .execute("INSERT INTO test_transaction_panic (name) VALUES ($1)", &[&"Test"])
                        .await
                        .expect("Failed to insert row");

                    panic!("Simulated panic to trigger rollback");
                },
            ))
            .catch_unwind()
            .await;

            assert!(result.is_err());

            // Verify the test row was not inserted
            let row = client
                .query_opt("SELECT name FROM test_transaction_panic WHERE id = $1", &[&1])
                .await
                .expect("Failed to query test row");
            assert!(row.is_none());
        })
        .await
    }
}
