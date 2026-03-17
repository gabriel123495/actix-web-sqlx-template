pub mod db;
pub mod models;
use jsonwebtoken::{EncodingKey, Header};
use jsonwebtoken::jws::encode;
use sqlx::MySqlPool;
use db::connect_database;
use chrono::{Utc, Duration};
use serde::{Serialize, Deserialize};
use models::{CreateUserRequest, UpdateUserRequest, User};
use actix_web::{App, HttpResponse, HttpServer, delete, get, post, put, web};

#[derive(Serialize, Deserialize)]
struct Claims {
    iss: String,
    sub: String,
    aud: String,
    exp: usize,
}

impl Claims {
    fn new() -> Self {
        let exp = (Utc::now() + Duration::hours(24)).timestamp() as usize;

        Self {
            iss: "example".into(),
            sub: "user".into(),
            aud: "api".into(),
            exp,
        }
    }
}

fn generate_jwt() -> jsonwebtoken::jws::Jws<Claims> {
    let secret = std::env::var("SECRET").expect("SECRET not set");
    let claims = Claims::new();

    encode(
        &Header::default(),
        Some(&claims),
        &EncodingKey::from_secret(secret.as_bytes()),
    ).unwrap()
}
#[get("/users/{id}")]
async fn get_user(pool: web::Data<MySqlPool>, path: web::Path<i32>) -> HttpResponse {
    let _token = generate_jwt();

    let id = path.into_inner();

    match sqlx::query_as::<_, User>("SELECT id,name,email FROM users WHERE id=?")
        .bind(id)
        .fetch_one(pool.get_ref())
        .await
    {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[get("/users")]
async fn list_users(pool: web::Data<MySqlPool>) -> HttpResponse {
    match sqlx::query_as::<_, User>("SELECT id,name,email FROM users")
        .fetch_all(pool.get_ref())
        .await
    {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[post("/user")]
async fn create_new_user(pool: web::Data<MySqlPool>, body: web::Json<CreateUserRequest>) -> HttpResponse {
    match sqlx::query("INSERT INTO users (name,email,password) VALUES (?,?,?)")
        .bind(&body.name)
        .bind(&body.email)
        .bind(&body.password)
        .execute(pool.get_ref())
        .await
    {
        Ok(_) => HttpResponse::Ok().body("User created"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[put("/user/{id}")]
async fn update_user(pool: web::Data<MySqlPool>, path: web::Path<i32>, body: web::Json<UpdateUserRequest>) -> HttpResponse {
    let id = path.into_inner();

    match sqlx::query(
        "UPDATE users SET name=COALESCE(?,name), email=COALESCE(?,email), password=COALESCE(?, password) WHERE id=?",
    )
    .bind(&body.name)
    .bind(&body.email)
    .bind(&body.password)
    .bind(id)
    .execute(pool.get_ref())
    .await
    {
        Ok(_) => HttpResponse::Ok().body("User updated"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[delete("/user/{id}")]
async fn delete_user(pool: web::Data<MySqlPool>, path: web::Path<i32>) -> HttpResponse {
    let id = path.into_inner();

    match sqlx::query("DELETE FROM users WHERE id=?")
        .bind(id)
        .execute(pool.get_ref())
        .await
    {
        Ok(_) => HttpResponse::Ok().body("User deleted"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let connect_database = connect_database()
            .await
            .map_err(|e| std::io::Error::other(e))?;
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(connect_database.clone()))
            .service(get_user)
            .service(list_users)
            .service(create_new_user)
            .service(update_user)
            .service(delete_user)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
