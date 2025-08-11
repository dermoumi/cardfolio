use chrono::{DateTime, NaiveDateTime, Utc};
use postgres_types::{FromSql, Type};
use std::{convert::From, result::Result};
use tokio_postgres::Client;

/// Utility to convert a timestamp with timezone to a DateTime<Utc>
#[derive(Debug)]
pub struct TimestampWithTimeZone(pub DateTime<Utc>);

impl<'a> FromSql<'a> for TimestampWithTimeZone {
    fn from_sql(
        ty: &Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        if ty.name() == "timestamp" {
            let naive_datetime = NaiveDateTime::from_sql(ty, raw)?;
            Ok(TimestampWithTimeZone(DateTime::from_naive_utc_and_offset(
                naive_datetime,
                Utc,
            )))
        } else {
            Err("Unexpected column type".into())
        }
    }

    fn accepts(ty: &Type) -> bool {
        ty.name() == "timestamp"
    }
}

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
    let result = f(db).await;

    // Rollback if the function returned an error, commit otherwise
    if result.is_err() {
        let query = if let Some(checkpoint_id) = &checkpoint {
            format!("ROLLBACK TO {checkpoint_id}")
        } else {
            "ROLLBACK".to_string()
        };

        db.execute(&query, &[]).await.map_err(|e| e.into())?;
    } else if checkpoint.is_none() {
        db.execute("COMMIT", &[]).await.map_err(|e| e.into())?;
    }

    // Forward the result
    result
}
