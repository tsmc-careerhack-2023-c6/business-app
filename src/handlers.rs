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

    let inventory_resp = app_state.nats_client.request(format!("inventory.{}", i), Bytes::from(serde_json::to_string(&order_payload).unwrap())).await.unwrap();
    let order_detail_from_inevntory_result: Result<OrderDetailFromInventory, serde_json::Error> = serde_json::from_slice(&inventory_resp.payload);
    
    if let Err(err) = order_detail_from_inevntory_result {
        eprintln!("Error: {}", err);
        return HttpResponse::InternalServerError().finish();
    }

    let order_detail_from_inevntory: OrderDetailFromInventory = order_detail_from_inevntory_result.unwrap();
    let order_detail_payload = OrderDetailPayload::from(order_detail_from_inevntory);

    let _  = app_state.nats_client.publish(format!("request.{}", i), Bytes::from(serde_json::to_string(&order_detail_payload).unwrap())).await;

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

        let start_time = chrono::NaiveDateTime::parse_from_str(
            &format!("{} 00:00:00", query_ref.date),
            "%Y-%m-%d %H:%M:%S",
        )
        .unwrap()
            - chrono::Duration::hours(8);
        let end_time = chrono::NaiveDateTime::parse_from_str(
            &format!("{} 23:59:59", query_ref.date),
            "%Y-%m-%d %H:%M:%S",
        )
        .unwrap()
            - chrono::Duration::hours(8);

        order_details
            .filter(location.eq(&query_ref.location))
            .filter(timestamp.ge(start_time))
            .filter(timestamp.le(end_time))
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

        let start_time = chrono::NaiveDateTime::parse_from_str(
            &format!("{} 00:00:00", query_ref.date),
            "%Y-%m-%d %H:%M:%S",
        )
        .unwrap()
            - chrono::Duration::hours(8);
        let end_time = chrono::NaiveDateTime::parse_from_str(
            &format!("{} 23:59:59", query_ref.date),
            "%Y-%m-%d %H:%M:%S",
        )
        .unwrap()
            - chrono::Duration::hours(8);

        order_details
            .filter(location.eq(&query_ref.location))
            .filter(timestamp.ge(start_time))
            .filter(timestamp.le(end_time))
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
