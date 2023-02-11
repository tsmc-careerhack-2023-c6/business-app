use std::sync::Arc;

use actix_web::{get, post, web, HttpResponse, Responder};
use bytes::Bytes;
use diesel::prelude::*;

use crate::models::*;

#[post("/order")]
pub async fn order(
    payload: web::Json<OrderPayload>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let i = rand::random::<u8>() % app_state.num_logical_processors as u8;

    let order_payload = payload.into_inner();
    let _ = app_state.nats_client.publish(format!("{}:inventory:{}", app_state.nats_topic_prefix, i), Bytes::from(serde_json::to_string(&order_payload).unwrap())).await.unwrap();

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

        println!("{} {}", &query_ref.date, &query_ref.location);

        let start_time = chrono::NaiveDateTime::parse_from_str(
            format!("{} 00:00:00", &query_ref.date).as_str(),
            "%Y-%m-%d %H:%M:%S",
        )
        .unwrap();
        let end_time = start_time + chrono::Duration::days(1);

        order_details
            .filter(location.eq(&query_ref.location))
            .filter(timestamp.between(start_time, end_time))
            .load::<OrderDetail>(&mut conn)
    })
    .await;

    if let Err(_) = web_block_result {
        // eprintln!("{}", err);
        return HttpResponse::InternalServerError().finish();
    }

    let query_result = web_block_result.unwrap();

    if let Err(_) = query_result {
        // eprintln!("{}", err);
        return HttpResponse::InternalServerError().finish();
    }

    let queried_order_details = query_result.unwrap();

    let order_records: Vec<OrderRecord> = queried_order_details
        .into_iter()
        .map(|order_detail| OrderRecord::from(order_detail))
        .collect();

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

    let web_block_result = web::block(move || {
        let mut conn = app_state.db_pool.get().unwrap();

        let start_time = chrono::NaiveDateTime::parse_from_str(
            format!("{} 00:00:00", &query_ref.date).as_str(),
            "%Y-%m-%d %H:%M:%S",
        )
        .unwrap();
        let end_time = start_time + chrono::Duration::days(1);

        order_details
            .filter(location.eq(&query_ref.location))
            .filter(timestamp.between(start_time, end_time))
            .load::<OrderDetail>(&mut conn)
    })
    .await;

    if let Err(_) = web_block_result {
        // eprintln!("{}", err);
        return HttpResponse::InternalServerError().finish();
    }

    let query_result = web_block_result.unwrap();

    if let Err(_) = query_result {
        // eprintln!("{}", err);
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

    HttpResponse::Ok().json(order_report)
}
