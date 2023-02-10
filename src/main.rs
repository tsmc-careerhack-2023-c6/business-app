#[macro_use]
extern crate diesel;

use std::sync::Arc;

use actix_web::{middleware, web, App, HttpServer};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use models::{AppState, OrderDetailPayload};

use futures::StreamExt;

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
    let database_pool = Arc::new(
        r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool."),
    );

    // let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
    // let redis_client = RedisConfig::from_url(&redis_url).unwrap();
    // let redis_policy = ReconnectPolicy::default();
    // let redis_pool = Arc::new(RedisPool::new(redis_client, 10).unwrap());
    // redis_pool.connect(Some(redis_policy));

    let nats_url = std::env::var("NATS_URL").expect("NATS_URL must be set");
    let nats_client = Arc::new(async_nats::connect(&nats_url).await.unwrap());

    let db_pool = database_pool.clone();

    for i in 0..16 {
        tokio::spawn({
            let db_pool = db_pool.clone();
            let nats_client = nats_client.clone();

            let mut subscribtion = nats_client
                .subscribe(format!("request.{}", i).into())
                .await
                .unwrap();

            async move {
                while let Some(request) = subscribtion.next().await {
                    use crate::schema::order_details::dsl::*;

                    let data = request.payload;

                    loop {
                        let mut conn = match db_pool.get() {
                            Ok(conn) => conn,
                            Err(e) => {
                                println!("Error: {}", e);
                                continue;
                            }
                        };

                        // deserialize the payload
                        let order_detail_payload: OrderDetailPayload =
                            serde_json::from_slice(&data).unwrap();
                        // insert the payload into database
                        let _ = diesel::insert_into(order_details)
                            .values(&order_detail_payload.clone())
                            .execute(&mut conn);

                        break;
                    }
                }
                Ok::<(), async_nats::Error>(())
            }
        });
    }

    let db_pool = database_pool.clone();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                db_pool: db_pool.clone(),
                // redis_pool: redis_pool.clone(),
                nats_client: nats_client.clone(),
            }))
            .wrap(middleware::Logger::default())
            .service(
                web::scope("/api")
                    .service(handlers::order)
                    .service(handlers::record)
                    .service(handlers::report),
            )
    })
    .workers(16)
    .bind("0.0.0.0:8100")?
    .run()
    .await
}
