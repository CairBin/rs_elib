use std::time::Duration;

use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use sea_orm_migration::MigratorTrait;

pub const DEFAULT_DB_CONN_STR: &str = "sqlite:elib.db?mode=rwc";

/// 初始化默认数据库连接(默认连接字符串: sqlite:elib.db?mode=rwc)
pub async fn init_db_default() -> Result<DatabaseConnection, DbErr>{
    init_db(DEFAULT_DB_CONN_STR).await
}

pub async fn init_db(conn_str: &str) -> Result<DatabaseConnection, DbErr>{
    let mut opt = ConnectOptions::new(conn_str);

    opt.max_connections(5)
        .min_connections(1)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(60))
        .max_lifetime(Duration::from_secs(600))
        .sqlx_logging(false);

    let db = Database::connect(opt).await?;
    crate::migration::Migrator::up(&db, None).await?;

    Ok(db)
}