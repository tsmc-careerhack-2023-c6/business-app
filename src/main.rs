#[macro_use]
extern crate diesel;

use actix_web::{web, App, HttpServer, middleware};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

mod handlers;
mod models;
mod schema;

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool: DbPool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(middleware::Logger::default())
            .service(
                web::scope("/api")
                    .service(handlers::order)
                    .service(handlers::record)
                    .service(handlers::report)
            )
    })
    .workers(16)
    .bind("0.0.0.0:8100")?
    .run()
    .await
}
