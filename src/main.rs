pub mod db;
pub mod models;
use sqlx::MySqlPool;
use db::connect_database;
use models::{CreateUserRequest, UpdateUserRequest, User};
use actix_web::{App, HttpResponse, HttpServer, delete, get, post, put, web};

#[get("/users")]
async fn list_users(pool: web::Data<MySqlPool>) -> HttpResponse {
    match sqlx::query_as::<_, User>("SELECT id, name, email FROM users")
        .fetch_all(pool.get_ref())
        .await
    {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

#[post("/user")]
async fn create_new_user(pool: web::Data<MySqlPool>, body: web::Json<CreateUserRequest>) -> HttpResponse {
    match sqlx::query("INSERT INTO users (name, email) VALUES (?, ?)")
        .bind(&body.name)
        .bind(&body.email)
        .execute(pool.get_ref())
        .await
    {
        Ok(_) => HttpResponse::Ok().json("User created"),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

#[put("/user/{id}")]
async fn update_user(pool: web::Data<MySqlPool>, path: web::Path<i32>, body: web::Json<UpdateUserRequest>) -> HttpResponse {
    let id = path.into_inner();
    match sqlx::query("UPDATE users SET name = COALESCE(?, name), email = COALESCE(?, email) WHERE id = ?")
        .bind(&body.name)
        .bind(&body.email)
        .bind(id)
        .execute(pool.get_ref())
        .await
    {
        Ok(_) => HttpResponse::Ok().json("User updated"),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

#[delete("/user/{id}")]
async fn delete_user(pool: web::Data<MySqlPool>, path: web::Path<i32>) -> HttpResponse {
    let id = path.into_inner();
    match sqlx::query("DELETE FROM users WHERE id = ?").bind(id).execute(pool.get_ref()).await {
        Ok(_) => HttpResponse::Ok().json("User deleted"),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = connect_database().await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(list_users)
            .service(create_new_user)
            .service(update_user)
            .service(delete_user)
    })
    .bind("localhost:8080")?
    .run()
    .await
}
