use dotenvy::dotenv;
use sqlx::{mysql::MySqlPoolOptions, mysql::MySql, Pool, Error};

pub async fn connect_database() -> Result<Pool<MySql>, Error> {
    dotenv().ok();
    let url = std::env::var("DATABASE_URL").unwrap();
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await?;
    Ok(pool)
}
