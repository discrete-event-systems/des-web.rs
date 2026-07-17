//! SeaORM connection handling. The pool is lazy (house pattern, cf.
//! athleto-app-rs): the server boots and serves every page even when Postgres
//! is unreachable; DB-backed sections degrade to a notice instead.

use std::time::Duration;

use sea_orm::{ConnectOptions, Database, DatabaseConnection};

pub async fn connect_lazy(url: &str) -> Result<DatabaseConnection, sea_orm::DbErr> {
    let mut opt = ConnectOptions::new(url.to_string());
    opt.max_connections(8)
        .min_connections(0)
        .connect_timeout(Duration::from_secs(5))
        .acquire_timeout(Duration::from_secs(5))
        .sqlx_logging(false)
        .connect_lazy(true);
    Database::connect(opt).await
}

pub async fn ping(db: &Option<DatabaseConnection>) -> bool {
    match db {
        Some(db) => db.ping().await.is_ok(),
        None => false,
    }
}
