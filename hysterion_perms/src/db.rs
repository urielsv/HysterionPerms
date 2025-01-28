use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use std::sync::Arc;
use tokio::sync::OnceCell;
use std::path::Path;

static DB_INSTANCE: OnceCell<Arc<DB>> = OnceCell::const_new();

pub struct DB {
    pub pool: SqlitePool,
}

impl DB {
    #[allow(dead_code)]
    pub async fn init(path: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let db_url = format!("sqlite:{}", path);
        
        if !Path::new(path).exists() {
            log::info!("Creating database at {}", path);
            Sqlite::create_database(&db_url).await?;
        }

        let pool = SqlitePool::connect(&db_url).await?;
        log::info!("Database connection established");

        Ok(DB { pool })
    }
}

#[allow(dead_code)]
pub async fn setup_db(path: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let db = DB::init(path).await?;
    if let Err(e) = DB_INSTANCE.set(Arc::new(db)) {
        return Err(format!("Failed to set DB instance: {}", e).into());
    }
    Ok(())
}

#[allow(dead_code)]
pub async fn get_db() -> Arc<DB> {
    DB_INSTANCE.get().expect("Database not initialized").clone()
} 