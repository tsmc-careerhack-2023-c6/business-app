#[macro_use]
extern crate diesel;

use std::sync::Arc;

use actix_web::{middleware, web, App, HttpServer};
use bytes::Bytes;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use models::{AppState, OrderDetailFromInventory, OrderDetailPayload, OrderPayload};

use futures::StreamExt;

mod handlers;
mod models;
mod schema;

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let num_logical_processors_result = std::env::var("NUM_LOGICAL_PROCESSORS")
        .map(|num_logical_processors| num_logical_processors.parse::<usize>().unwrap());

    let num_logical_processors = match num_logical_processors_result {
        Ok(num_logical_processors) => num_logical_processors,
        Err(_) => num_cpus::get(),
    };

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let database_pool = Arc::new(
        r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool."),
    );

    let nats_url = std::env::var("NATS_URL").expect("NATS_URL must be set");
    let nats_client = Arc::new(async_nats::connect(&nats_url).await.unwrap());

    let nats_topic_prefix =
        std::env::var("NATS_TOPIC_PREFIX").expect("NATS_TOPIC_PREFIX must be set");

    let inventory_url = std::env::var("INVENTORY_URL").expect("INVENTORY_URL must be set");

    for i in 0..num_logical_processors {
        let inventory_url_cloned = inventory_url.clone();

        tokio::spawn({
            let nats_client = nats_client.clone();

            let mut subscribtion = nats_client
                .subscribe(format!("{}:inventory:{}", nats_topic_prefix, i).into())
                .await
                .unwrap();

            let nats_topic_prefix = nats_topic_prefix.clone();

            async move {
                while let Some(request) = subscribtion.next().await {
                    let data = request.payload;
                    let order_payload: OrderPayload = serde_json::from_slice(&data).unwrap();

                    let order_detail_from_inventory = loop {
                        let resp_result = reqwest::Client::new()
                            .post(&inventory_url_cloned)
                            .json(&order_payload)
                            .send()
                            .await;

                        if let Err(_) = resp_result {
                            // eprintln!("Error: {}", e);
                            continue;
                        }

                        match resp_result
                            .unwrap()
                            .json::<OrderDetailFromInventory>()
                            .await
                        {
                            Ok(order_detail_from_inventory) => {
                                break order_detail_from_inventory;
                            }
                            Err(_) => {
                                // eprintln!("Error: {}", e);
                                continue;
                            }
                        }
                    };

                    let order_detail_payload =
                        OrderDetailPayload::from(order_detail_from_inventory);
                    let order_detail_payload_string =
                        serde_json::to_string(&order_detail_payload).unwrap();

                    nats_client
                        .publish(
                            format!("{}:db:{}", nats_topic_prefix, i).into(),
                            Bytes::from(order_detail_payload_string),
                        )
                        .await
                        .unwrap();
                }
                Ok::<(), async_nats::Error>(())
            }
        });
    }

    let db_pool = database_pool.clone();

    for i in 0..num_logical_processors {
        tokio::spawn({
            let db_pool = db_pool.clone();
            let nats_client = nats_client.clone();

            let mut subscribtion = nats_client
                .subscribe(format!("{}:db:{}", nats_topic_prefix, i).into())
                .await
                .unwrap();

            async move {
                while let Some(request) = subscribtion.next().await {
                    use crate::schema::order_details::dsl::*;

                    let data = request.payload;
                    let order_detail_payload: OrderDetailPayload =
                        serde_json::from_slice(&data).unwrap();

                    loop {
                        let mut conn = match db_pool.get() {
                            Ok(conn) => conn,
                            Err(_) => {
                                // eprintln!("Error: {}", e);
                                continue;
                            }
                        };

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
                nats_client: nats_client.clone(),
                nats_topic_prefix: nats_topic_prefix.clone(),
                num_logical_processors,
            }))
            .wrap(middleware::Logger::default())
            .service(
                web::scope("/api")
                    .service(handlers::order)
                    .service(handlers::record)
                    .service(handlers::report),
            )
    })
    .workers(num_logical_processors)
    .bind("0.0.0.0:8100")?
    .run()
    .await
}
