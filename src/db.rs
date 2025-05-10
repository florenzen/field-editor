use sqlx::{FromRow, Pool, Sqlite, SqlitePool};
use std::sync::Arc;
use serde::{Serialize, Deserialize};

// Our data model
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Fields {
    pub id: i64,
    pub field1: String,
    pub field2: String,
    pub field3: String,
    pub field4: String,
    pub version: i64,
}

// Database connection manager
pub struct DbManager {
    connection_string: String,
    pool: Option<Arc<Pool<Sqlite>>>,
}

impl DbManager {
    pub fn new(connection_string: &str) -> Self {
        DbManager {
            connection_string: connection_string.to_string(),
            pool: None,
        }
    }

    // Initialize the database and create tables if they don't exist
    pub async fn initialize(&mut self) -> Result<(), sqlx::Error> {
        // Create a connection pool
        let pool = SqlitePool::connect(&self.connection_string).await?;

        // Create our fields table with a version column for concurrency control
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS fields (
                id INTEGER PRIMARY KEY,
                field1 TEXT NOT NULL,
                field2 TEXT NOT NULL,
                field3 TEXT NOT NULL,
                field4 TEXT NOT NULL,
                version INTEGER NOT NULL DEFAULT 1
            )
            "#,
        )
        .execute(&pool)
        .await?;

        // Insert default data if the table is empty
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM fields")
            .fetch_one(&pool)
            .await?;

        if count == 0 {
            sqlx::query(
                r#"
                INSERT INTO fields (id, field1, field2, field3, field4, version)
                VALUES (1, ?, ?, ?, ?, 1)
                "#,
            )
            .bind("Default value 1")
            .bind("Default value 2")
            .bind("Default value 3")
            .bind("Default value 4")
            .execute(&pool)
            .await?;
        }

        self.pool = Some(Arc::new(pool));
        Ok(())
    }

    // Get all field values with their current version
    pub async fn get_fields(&self) -> Result<Fields, sqlx::Error> {
        let pool = self.pool.as_ref().expect("Database not initialized");
        
        // Fetch fields using query_as instead of the macro
        let fields = sqlx::query_as::<_, Fields>(
            "SELECT id, field1, field2, field3, field4, version FROM fields WHERE id = 1"
        )
        .fetch_one(pool.as_ref())
        .await?;
        
        Ok(fields)
    }

    // Update fields with optimistic concurrency control
    pub async fn update_fields(
        &self, 
        field1: &str, 
        field2: &str, 
        field3: &str, 
        field4: &str, 
        expected_version: i64
    ) -> Result<bool, sqlx::Error> {
        let pool = self.pool.as_ref().expect("Database not initialized");
        
        // Start a transaction
        let mut tx = pool.begin().await?;
        
        // First check if the version matches
        let current_version: Option<i64> = sqlx::query_scalar(
            "SELECT version FROM fields WHERE id = 1 AND version = ?"
        )
        .bind(expected_version)
        .fetch_optional(&mut *tx)
        .await?;
        
        // If the version doesn't match, someone else has updated the record
        if current_version.is_none() {
            tx.rollback().await?;
            return Ok(false); // Concurrency conflict
        }
        
        // Update the fields and increment the version
        let result = sqlx::query(
            r#"
            UPDATE fields
            SET field1 = ?, field2 = ?, field3 = ?, field4 = ?, version = version + 1
            WHERE id = 1 AND version = ?
            "#
        )
        .bind(field1)
        .bind(field2)
        .bind(field3)
        .bind(field4)
        .bind(expected_version)
        .execute(&mut *tx)
        .await?;
        
        // Commit the transaction
        tx.commit().await?;
        
        // Check if the update was successful
        Ok(result.rows_affected() > 0)
    }
}
