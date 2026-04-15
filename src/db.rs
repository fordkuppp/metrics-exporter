use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use tracing::info;

const DB_NAME: &str = "metrics_exporter";

pub async fn init_pool(database_url: &str) -> Result<PgPool> {
    let mut conn = PgConnection::connect(database_url).await?;

    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname = $1)",
    )
    .bind(DB_NAME)
    .fetch_one(&mut conn)
    .await?;

    if !exists {
        conn.execute(format!("CREATE DATABASE \"{}\"", DB_NAME).as_str())
            .await?;
        info!("Created database '{}'", DB_NAME);
    }

    let db_url = format!("{}/{}", database_url.trim_end_matches('/'), DB_NAME);
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    sqlx::migrate!().run(&pool).await?;

    Ok(pool)
}
