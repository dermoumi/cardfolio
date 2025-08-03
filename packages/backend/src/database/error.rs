#[derive(thiserror::Error, Debug)]
pub enum DatabaseError {
    #[error("Migration conflict at index {0}: expected '{1}', found '{2}'")]
    MigrationConflict(usize, String, String),

    #[error(transparent)]
    Postgres(#[from] tokio_postgres::Error),

    #[error(transparent)]
    Bb8Error(#[from] bb8::RunError<tokio_postgres::Error>),
}
