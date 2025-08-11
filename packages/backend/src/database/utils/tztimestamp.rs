use chrono::{DateTime, NaiveDateTime, Utc};
use postgres_types::{FromSql, Type};

/// Utility to convert a timestamp with timezone to a DateTime<Utc>
#[derive(Debug)]
pub struct TzTimestamp(pub DateTime<Utc>);

impl<'a> FromSql<'a> for TzTimestamp {
    fn from_sql(
        ty: &Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        if ty.name() == "timestamp" {
            let naive_datetime = NaiveDateTime::from_sql(ty, raw)?;
            Ok(TzTimestamp(DateTime::from_naive_utc_and_offset(
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
