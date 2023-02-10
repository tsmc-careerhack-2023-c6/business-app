use std::sync::Arc;

use actix_web::{get, post, web, HttpResponse, Responder};
use bytes::Bytes;
use diesel::prelude::*;
// use diesel::result::Error;

use crate::models::*;

#[post("/order")]
pub async fn order(
    payload: web::Json<OrderPayload>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    dotenv::dotenv().ok();
    let i = rand::random::<u8>() % 16;

    let order_payload = payload.into_inner();

    let nats_topic_prefix = std::env::var("NATS_TOPIC_PREFIX").expect("NATS_TOPIC_PREFIX must be set");
    let _ = app_state.nats_client.publish(format!("{}_inventory_to_db.{}", nats_topic_prefix, i), Bytes::from(serde_json::to_string(&order_payload).unwrap())).await.unwrap();

    HttpResponse::Ok().finish()
}

#[get("/record")]
pub async fn record(
    query: web::Query<OrderQuery>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    use crate::schema::order_details::dsl::*;

    let query_ref = Arc::new(query);

    let web_block_result = web::block(move || {
        let mut conn = app_state.db_pool.get().unwrap();

        order_details
            .filter(location.eq(&query_ref.location))
            .filter(timestamp.like(format!("{}%", query_ref.date)))
            .load::<OrderDetail>(&mut conn)
    })
    .await;

    if let Err(err) = web_block_result {
        eprintln!("{}", err);
        return HttpResponse::InternalServerError().finish();
    }

    let query_result = web_block_result.unwrap();

    if let Err(err) = query_result {
        eprintln!("{}", err);
        return HttpResponse::InternalServerError().finish();
    }

    let queried_order_details = query_result.unwrap();

    let order_records: Vec<OrderRecord> = queried_order_details
        .into_iter()
        .map(|order_detail| OrderRecord::from(order_detail))
        .collect();

    // println!("Set cache in redis");

    // let _: () = cloned_app_state
    //     .redis_pool
    //     .set(
    //         &key,
    //         serde_json::to_string(&order_records).unwrap(),
    //         Some(Expiration::EX(10)),
    //         None,
    //         false,
    //     )
    //     .await
    //     .unwrap();

    HttpResponse::Ok().json(order_records)
}

#[get("/report")]
pub async fn report(
    query: web::Query<OrderQuery>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    use crate::schema::order_details::dsl::*;

    let query_ref = Arc::new(query);
    let cloned_query_ref = query_ref.clone();

    // let cloned_app_state = app_state.clone();

    // let key = format!("report:{}:{}", query_ref.date, query_ref.location);

    // let redis_result: RedisValue = app_state
    //     .redis_pool
    //     .get(&key)
    //     .await
    //     .unwrap();

    // if let RedisValue::String(redis_string) = redis_result {
    //     println!("Found cache in redis");

    //     let order_report: OrderReport = serde_json::from_str(&redis_string).unwrap();

    //     return HttpResponse::Ok().json(order_report);
    // }

    let web_block_result = web::block(move || {
        let mut conn = app_state.db_pool.get().unwrap();

        order_details
            .filter(location.eq(&query_ref.location))
            .filter(timestamp.like(format!("{}%", query_ref.date)))
            .load::<OrderDetail>(&mut conn)
    })
    .await;

    if let Err(err) = web_block_result {
        eprintln!("{}", err);
        return HttpResponse::InternalServerError().finish();
    }

    let query_result = web_block_result.unwrap();

    if let Err(err) = query_result {
        eprintln!("{}", err);
        return HttpResponse::InternalServerError().finish();
    }

    let queried_order_details = query_result.unwrap();

    let order_records: Vec<OrderRecord> = queried_order_details
        .into_iter()
        .map(|order_detail| OrderRecord::from(order_detail))
        .collect();

    let order_report = OrderReport {
        location: cloned_query_ref.location.clone(),
        date: cloned_query_ref.date.clone(),
        count: order_records.len(),
        material: order_records
            .iter()
            .map(|order_record| order_record.material)
            .sum(),
        a: order_records
            .iter()
            .map(|order_record| order_record.a)
            .sum(),
        b: order_records
            .iter()
            .map(|order_record| order_record.b)
            .sum(),
        c: order_records
            .iter()
            .map(|order_record| order_record.c)
            .sum(),
        d: order_records
            .iter()
            .map(|order_record| order_record.d)
            .sum(),
    };

    // println!("Set cache in redis");

    // let _: () = cloned_app_state
    //     .redis_pool
    //     .set(
    //         &key,
    //         serde_json::to_string(&order_report).unwrap(),
    //         Some(Expiration::EX(10)),
    //         None,
    //         false,
    //     )
    //     .await
    //     .unwrap();

    HttpResponse::Ok().json(order_report)
}
