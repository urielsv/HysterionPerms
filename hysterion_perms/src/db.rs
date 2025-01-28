use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use std::sync::Arc;
use tokio::sync::OnceCell;

static DB_INSTANCE: OnceCell<Arc<DB>> = OnceCell::const_new();

pub struct DB {
    pub pool: SqlitePool,
}

impl DB {
    pub async fn init(path: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut requires_setup = false;
        if !Sqlite::database_exists(path).await? {
            Sqlite::create_database(path).await?;
            requires_setup = true;
        }

        let pool = SqlitePool::connect(path).await?;

        if requires_setup {
            log::info!("Setting up database...");
        }

        Ok(DB { pool })
    }
}

pub async fn setup_db(path: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let db = DB::init(path).await?;
    if let Err(e) = DB_INSTANCE.set(Arc::new(db)) {
        return Err(format!("Failed to set DB instance: {}", e).into());
    }
    Ok(())
}

pub async fn get_db() -> Arc<DB> {
    DB_INSTANCE.get().unwrap().clone()
} 