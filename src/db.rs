use dotenvy::dotenv;
use sqlx::{mysql, Pool, Error};

pub async fn connect_database() -> Result<Pool<mysql::MySql>, Error> {
    dotenv().ok();
    let url = std::env::var("DATABASE_URL").unwrap();
    let pool = mysql::MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await?;
    Ok(pool)
}
