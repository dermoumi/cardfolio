use std::{result::Result, str::FromStr, time::Duration};

use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::{Config, NoTls};

/// Alias for the bb8 database pool type
pub type Pool = bb8::Pool<PostgresConnectionManager<NoTls>>;

/// Initialize the database connection pool
pub async fn init<'a>(db_url: &str, pool_size: u32) -> Result<Pool, tokio_postgres::Error> {
    let config = create_db_config(db_url)?;

    create_db_pool(config, pool_size).await
}

/// Create a database config from the database URL string
fn create_db_config(db_url: &str) -> Result<Config, tokio_postgres::Error> {
    let mut config = Config::from_str(db_url)?;

    config
        .connect_timeout(Duration::from_secs(5))
        .tcp_user_timeout(Duration::from_secs(5));

    Ok(config)
}

/// Create a database connection pool
async fn create_db_pool(config: Config, pool_size: u32) -> Result<Pool, tokio_postgres::Error> {
    let manager = PostgresConnectionManager::new(config, NoTls);
    let pool = Pool::builder().max_size(pool_size).build(manager).await?;

    Ok(pool)
}
