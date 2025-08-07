use tokio_postgres::Client;

use crate::database::{DatabaseError, Pool, with_transaction};

pub type Migration<'a> = (&'a str, &'a str, Option<&'a str>);

pub struct Migrate {
    tablename: String,
}

impl Migrate {
    pub fn new(tablename: &str) -> Self {
        Self {
            tablename: tablename.to_string(),
        }
    }

    /// Migrate all scripts up
    pub async fn run<'a>(
        &self,
        db_pool: &Pool,
        migrations: &[Migration<'a>],
    ) -> Result<(), DatabaseError> {
        tracing::info!("Migrating up into {}", self.tablename);

        let mut client = db_pool.get().await?;
        self.create_table(&client).await?;

        let applied_migrations_count = self.check_migrations(&client, migrations).await?;

        if applied_migrations_count > migrations.len() {
            tracing::info!("Applying migrations down");
            self.migrate_down(&mut client, migrations).await?;
        } else if applied_migrations_count < migrations.len() {
            tracing::info!("Applying migrations up");
            self.migrate_up(&mut client, migrations).await?;
        } else {
            tracing::info!("No migrations to apply, already at the latest version");
        }

        Ok(())
    }

    async fn create_table(&self, client: &Client) -> Result<(), DatabaseError> {
        tracing::debug!("Creating migration table {}", self.tablename);
        let query = format!(
            r#"CREATE TABLE IF NOT EXISTS {} (
                id SERIAL PRIMARY KEY,
                name TEXT NOT NULL,
                down_script TEXT NULL,
                executed_at TIMESTAMP NOT NULL DEFAULT now()
            )"#,
            self.tablename
        );

        self.execute_script(client, &query).await
    }

    async fn check_migrations(
        &self,
        client: &Client,
        migrations: &[Migration<'_>],
    ) -> Result<usize, DatabaseError> {
        let query = format!("SELECT name FROM {} ORDER BY id ASC", self.tablename);
        let statement = client.prepare(&query).await?;
        let rows = client.query(&statement, &[]).await?;

        let applied_migrations: Vec<String> = rows.iter().map(|row| row.get("name")).collect();

        for (index, (name, _, _)) in migrations.iter().enumerate() {
            if index >= applied_migrations.len() {
                break;
            }

            let applied = &applied_migrations[index];
            if name != applied {
                return Err(DatabaseError::MigrationConflict(
                    index,
                    name.to_string(),
                    applied.to_string(),
                ));
            }
        }

        Ok(applied_migrations.len())
    }

    async fn execute_script(&self, client: &Client, script: &str) -> Result<(), DatabaseError> {
        let statement = client.prepare(script).await?;
        client.execute(&statement, &[]).await?;
        Ok(())
    }

    async fn migrate_up(
        &self,
        mut client: &mut Client,
        migrations: &[Migration<'_>],
    ) -> Result<(), DatabaseError> {
        for (name, up_script, down_script) in migrations {
            // Acquire an advisory lock
            client.execute("SELECT pg_advisory_lock(1)", &[]).await?;

            let result = with_transaction(
                &mut client,
                None,
                async |client| -> Result<(), DatabaseError> {
                    if !self.migration_exists(client, name).await? {
                        tracing::info!("Applying migration '{}'", name);
                        self.execute_script(client, up_script).await?;
                        self.insert_migration(client, name, down_script.as_deref())
                            .await?;
                    };

                    Ok(())
                },
            )
            .await;

            // Release the advisory lock
            client.execute("SELECT pg_advisory_unlock(1)", &[]).await?;

            result?;
        }
        Ok(())
    }

    async fn migrate_down(
        &self,
        mut client: &mut Client,
        migrations: &[Migration<'_>],
    ) -> Result<(), DatabaseError> {
        if let Some(&(last_migration_name, _, _)) = migrations.last() {
            tracing::info!("Reverting migrations down to '{}'", last_migration_name);

            loop {
                let done = with_transaction::<_, DatabaseError, _, _>(
                    &mut client,
                    None,
                    async |client| -> Result<bool, DatabaseError> {
                        let last_applied_migration =
                            self.get_last_applied_migration(client).await?;
                        if let Some((name, down_script)) = last_applied_migration
                            && last_migration_name != name
                        {
                            tracing::info!("Reverting migration '{}'...", name);
                            self.delete_migration(client, &name).await?;
                            if let Some(script) = down_script {
                                self.execute_script(client, &script).await?;
                            }

                            Ok(false)
                        } else {
                            Ok(true)
                        }
                    },
                )
                .await?;

                if done {
                    break;
                }
            }

            tracing::info!("Done!");
        } else {
            tracing::warn!("Migrations list is empty, skipping revert...");
        }

        Ok(())
    }

    async fn migration_exists(&self, client: &Client, name: &str) -> Result<bool, DatabaseError> {
        let query = format!("SELECT 1 FROM {} WHERE name = $1", self.tablename);
        let statement = client.prepare(&query).await?;
        let rows = client.query(&statement, &[&name]).await?;
        Ok(!rows.is_empty())
    }

    async fn insert_migration(
        &self,
        client: &Client,
        name: &str,
        down_script: Option<&str>,
    ) -> Result<(), DatabaseError> {
        let query = format!(
            "INSERT INTO {} (name, down_script) VALUES ($1, $2)",
            self.tablename
        );
        let statement = client.prepare(&query).await?;
        client.execute(&statement, &[&name, &down_script]).await?;
        Ok(())
    }

    async fn delete_migration(&self, client: &Client, name: &str) -> Result<(), DatabaseError> {
        let query = format!("DELETE FROM {} WHERE name = $1", self.tablename);
        let statement = client.prepare(&query).await?;
        client.execute(&statement, &[&name]).await?;
        Ok(())
    }

    async fn get_last_applied_migration(
        &self,
        client: &Client,
    ) -> Result<Option<(String, Option<String>)>, DatabaseError> {
        let query = format!(
            "SELECT name, down_script FROM {} ORDER BY id DESC LIMIT 1",
            self.tablename
        );
        let statement = client.prepare(&query).await?;
        let rows = client.query(&statement, &[]).await?;

        if let Some(row) = rows.first() {
            let name: String = row.get("name");
            let down_script: Option<String> = row.get("down_script");
            Ok(Some((name, down_script)))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{env, panic, sync::Arc};

    use futures_util::FutureExt;

    use super::*;
    use crate::database;

    /// Utility to create a database connection pool for testing
    pub async fn with_db_pool_no_migrations<Fn, Fut>(f: Fn)
    where
        Fn: FnOnce(Arc<Pool>) -> Fut,
        Fut: Future<Output = ()>,
    {
        let db_url = env::var("CARDFOLIO_DB_TEST").expect("CARDFOLIO_DB_TEST must be set");
        let db_pool = database::init(&db_url, 1)
            .await
            .expect("Failed to create test DB pool");

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

    #[tokio::test]
    async fn test_migrate_run_applies_new_migrations() {
        with_db_pool_no_migrations(async |db| {
            {
                let client = db.get().await.expect("Failed to get DB connection");
                let migrator = Migrate::new("test_migrations");
                migrator
                    .create_table(&client)
                    .await
                    .expect("Failed to create migration table");
                migrator
                    .insert_migration(&client, "001_dummy", None)
                    .await
                    .expect("Failed to insert initial migration");
            }

            Migrate::new("test_migrations")
                .run(
                    &db,
                    &[
                        ("001_dummy", "SELECT 1;", None),
                        (
                            "002_initial",
                            r#"
                        DO $$ BEGIN
                            CREATE TABLE test (id SERIAL PRIMARY KEY, name TEXT);
                            INSERT INTO test (name) VALUES ('Initial data');
                        END $$;
                        "#,
                            None,
                        ),
                    ],
                )
                .await
                .expect("Migration failed");

            let client = db.get().await.expect("Failed to get DB connection");
            let rows = client
                .query("SELECT name FROM test_migrations", &[])
                .await
                .expect("Failed to query migrations");

            assert_eq!(rows.len(), 2);
            assert_eq!(rows[0].get::<_, String>("name"), "001_dummy");
            assert_eq!(rows[1].get::<_, String>("name"), "002_initial");

            // Check if the test table exists
            let rows = client
                .query_one("SELECT id, name FROM test", &[])
                .await
                .expect("Failed to check if table exists");

            let id: i32 = rows.get("id");
            let name: String = rows.get("name");

            assert_eq!(id, 1);
            assert_eq!(name, "Initial data");
        })
        .await
    }

    #[tokio::test]
    async fn test_migrate_run_skips_existing_migrations() {
        with_db_pool_no_migrations(async |db| {
            let migrations = &[(
                "001_initial",
                r#"
                    DO $$ BEGIN
                        CREATE TABLE test (id SERIAL PRIMARY KEY, name TEXT);
                        INSERT INTO test (name) VALUES ('Initial data');
                    END $$;
                    "#,
                None,
            )];

            let migrator = Migrate::new("test_migrations");
            migrator
                .run(&db, migrations)
                .await
                .expect("Migration failed");

            // Run the migration again, it should be skipped
            migrator
                .run(&db, migrations)
                .await
                .expect("Migration failed");

            let client = db.get().await.expect("Failed to get DB connection");
            let rows = client
                .query("SELECT name FROM test_migrations", &[])
                .await
                .expect("Failed to query migrations");

            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].get::<_, String>(0), "001_initial");

            // Check if the test table exists
            let rows = client
                .query_one("SELECT id, name FROM test", &[])
                .await
                .expect("Failed to check if table exists");
            let id: i32 = rows.get("id");
            let name: String = rows.get("name");
            assert_eq!(id, 1);
            assert_eq!(name, "Initial data");
        })
        .await;
    }

    #[tokio::test]
    async fn test_migration_run_undoes_migrations() {
        with_db_pool_no_migrations(async |db| {
            let migrations_initial = &[
                (
                    "001_initial",
                    "CREATE TABLE test (id SERIAL PRIMARY KEY, name TEXT)",
                    None,
                ),
                ("002_do_nothing", "SELECT 1", None),
                (
                    "003_add_data",
                    "INSERT INTO test (name) VALUES ('Test data')",
                    Some("DELETE FROM test WHERE name = 'Test data'"),
                ),
            ];

            let migrations = &[(
                "001_initial",
                "CREATE TABLE test (id SERIAL PRIMARY KEY, name TEXT)",
                None,
            )];

            let migrator = Migrate::new("test_migrations");
            migrator
                .run(&db, migrations_initial)
                .await
                .expect("Migration failed");

            // Now run the migration down
            migrator
                .run(&db, migrations)
                .await
                .expect("Migration down failed");

            let client = db.get().await.expect("Failed to get DB connection");
            let rows = client
                .query("SELECT name FROM test", &[])
                .await
                .expect("Failed to query test table");

            assert!(rows.is_empty());
        })
        .await;
    }

    #[tokio::test]
    async fn test_create_table_creates_migration_table() {
        with_db_pool_no_migrations(async |db| {
            let migrator = Migrate::new("test_migrations");
            let client = db.get().await.expect("Failed to get DB connection");

            migrator
                .create_table(&client)
                .await
                .expect("Failed to create migration table");

            // Check if the migrations table exists by querying information_schema
            let row = client
                .query_one(
                    r#"SELECT COUNT(*)
                    FROM information_schema.tables
                    WHERE table_schema = 'public' AND table_name = 'test_migrations'"#,
                    &[],
                )
                .await
                .expect("Failed to query information_schema.tables");

            let count: i64 = row.get(0);
            assert_eq!(count, 1, "migrations table should exist");
        })
        .await;
    }

    #[tokio::test]
    async fn test_execute_script() {
        with_db_pool_no_migrations(async |db| {
            let client = db.get().await.expect("Failed to get DB connection");
            let migrator = Migrate::new("test_migrations");
            migrator
                .create_table(&client)
                .await
                .expect("Failed to create table");

            // Create a simple script to create a test table
            migrator
                .execute_script(
                    &client,
                    "INSERT INTO test_migrations (name, down_script) VALUES ('001_test_script', NULL);",
                )
                .await
                .expect("Failed to execute script");

            // Verify that the migration was inserted
            let row = client
                .query_one(
                    "SELECT name FROM test_migrations WHERE name = '001_test_script'",
                    &[],
                )
                .await
                .expect("Failed to query migrations");
            assert_eq!(row.get::<_, String>("name"), "001_test_script");
        })
        .await;
    }

    #[tokio::test]
    async fn test_check_migrations_only_checks_existing_migrations() {
        with_db_pool_no_migrations(async |db| {
            let client = db.get().await.expect("Failed to get DB connection");

            let migrator = Migrate::new("test_migrations");
            migrator
                .create_table(&client)
                .await
                .expect("Failed to create table");

            migrator
                .execute_script(
                    &client,
                    r#"INSERT INTO test_migrations (name, down_script) VALUES
                        ('001_migration_1', NULL),
                        ('002_migration_2', NULL);"#,
                )
                .await
                .expect("Failed to insert migrations");

            let migrations = &[
                ("001_migration_1", "SELECT 1;", None),
                ("002_migration_2", "SELECT 2;", None),
                ("003_migration_3", "SELECT 1;", None),
            ];
            let applied_count = migrator
                .check_migrations(&client, migrations)
                .await
                .expect("Check failed");

            assert_eq!(applied_count, 2);
        })
        .await;
    }

    #[tokio::test]
    async fn test_check_migrations_fails_on_conflict() {
        with_db_pool_no_migrations(async |db| {
            let client = db.get().await.expect("Failed to get DB connection");

            let migrator = Migrate::new("test_migrations");
            migrator.create_table(&client).await.expect("Failed to create table");

            migrator
                .execute_script(
                    &client,
                    r#"INSERT INTO test_migrations (name, down_script) VALUES
                        ('001_migration_1', NULL),
                        ('002_migration_conflict', NULL);"#,
                )
                .await
                .expect("Failed to insert migrations");

            let migrations = &[
                ("001_migration_1", "SELECT 1;", None),
                ("002_migration_2", "SELECT 2;", None),
                ("003_migration_3", "SELECT 1;", None),
            ];
            let result = migrator
                .check_migrations(&client, migrations)
                .await;

            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err().to_string(),
                "Migration conflict at index 1: expected '002_migration_2', found '002_migration_conflict'"
            );
        })
        .await;
    }

    #[tokio::test]
    async fn test_get_last_applied_migration() {
        with_db_pool_no_migrations(async |db| {
            let client = db.get().await.expect("Failed to get DB connection");

            let migrator = Migrate::new("test_migrations");
            migrator
                .create_table(&client)
                .await
                .expect("Failed to create table");

            migrator
                .execute_script(
                    &client,
                    r#"INSERT INTO test_migrations (name, down_script) VALUES
                        ('001_migration_1', NULL),
                        ('002_migration_2', 'SELECT 2;');"#,
                )
                .await
                .expect("Failed to insert migration");

            let last_migration = migrator
                .get_last_applied_migration(&client)
                .await
                .expect("Failed to get last applied migration");

            assert_eq!(
                last_migration,
                Some(("002_migration_2".to_string(), Some("SELECT 2;".to_string())))
            );
        })
        .await;
    }

    #[tokio::test]
    async fn test_get_last_applied_migration_no_migrations() {
        with_db_pool_no_migrations(async |db| {
            let client = db.get().await.expect("Failed to get DB connection");

            let migrator = Migrate::new("test_migrations");
            migrator
                .create_table(&client)
                .await
                .expect("Failed to create table");

            let last_migration = migrator
                .get_last_applied_migration(&client)
                .await
                .expect("Failed to get last applied migration");

            assert!(last_migration.is_none());
        })
        .await;
    }

    #[tokio::test]
    async fn test_insert_migration() {
        with_db_pool_no_migrations(async |db| {
            let client = db.get().await.expect("Failed to get DB connection");

            let migrator = Migrate::new("test_migrations");
            migrator
                .create_table(&client)
                .await
                .expect("Failed to create table");

            migrator
                .insert_migration(&client, "001_test_migration", None)
                .await
                .expect("Failed to insert migration");

            // Verify that the migration was inserted
            let row = client
                .query_one(
                    "SELECT name FROM test_migrations WHERE name = '001_test_migration'",
                    &[],
                )
                .await
                .expect("Failed to query migrations");
            assert_eq!(row.get::<_, String>("name"), "001_test_migration");
        })
        .await;
    }

    #[tokio::test]
    async fn test_delete_migration() {
        with_db_pool_no_migrations(async |db| {
            let client = db.get().await.expect("Failed to get DB connection");

            let migrator = Migrate::new("test_migrations");
            migrator
                .create_table(&client)
                .await
                .expect("Failed to create table");

            migrator
                .insert_migration(&client, "001_migration_1", None)
                .await
                .expect("Failed to insert migration");
            migrator
                .insert_migration(&client, "002_migration_2", None)
                .await
                .expect("Failed to insert migration");

            // Now delete the migration
            migrator
                .delete_migration(&client, "002_migration_2")
                .await
                .expect("Failed to delete migration");

            // Verify that the migration was deleted
            let result = client
                .query_opt(
                    "SELECT name FROM test_migrations WHERE name = '002_migration_2'",
                    &[],
                )
                .await
                .expect("Failed to query migrations");
            assert!(result.is_none());
        })
        .await;
    }

    #[tokio::test]
    async fn test_migration_down_skips_revert_if_no_migrations() {
        with_db_pool_no_migrations(async |db| {
            let migrator = Migrate::new("test_migrations");
            let mut client = db.get().await.expect("Failed to get DB connection");

            // Create the migration table and a few migrations
            migrator
                .create_table(&client)
                .await
                .expect("Failed to create migration table");
            migrator
                .insert_migration(&client, "001_initial", None)
                .await
                .expect("Failed to insert initial migration");

            // Run the migration down without any migrations
            migrator
                .migrate_down(&mut client, &[])
                .await
                .expect("Migration down failed");

            // Verify that no migrations were reverted
            let rows = client
                .query("SELECT name FROM test_migrations", &[])
                .await
                .expect("Failed to query migrations");
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].get::<_, String>("name"), "001_initial");
        })
        .await;
    }
}
