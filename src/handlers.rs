use super::DbPool;

use actix_web::{get, post, web, HttpResponse, Responder};
use diesel::prelude::*;
use diesel::result::Error;

use crate::models::*;

// type DbError = Box<dyn std::error::Error + Send + Sync>;

fn insert_order_detail(conn: &mut PgConnection, order_detail_payload: OrderDetailPayload) -> Result<OrderDetail, Error> {
    use crate::schema::order_details::dsl::*;

    let result = diesel::insert_into(order_details)
        .values(&order_detail_payload.clone())
        .returning((id, location, timestamp, signature, material, a, b, c, d))
        .get_result::<OrderDetail>(conn);

    result
}

#[post("/order")]
pub async fn order(
    payload: web::Json<OrderPayload>,
    pool: web::Data<DbPool>,
) -> impl Responder {
    let order = payload.into_inner();
    
    let client = reqwest::Client::new();
    let resp_res = client
        .post("http://localhost:8200/api/inventory")
        .json(&order)
        .send()
        .await
        .unwrap()
        .json::<OrderDetailFromInventory>()
        .await;
    
    if let Err(resp_err) = resp_res {
        eprintln!("{}", resp_err);
        return HttpResponse::InternalServerError().finish();
    }

    let resp = resp_res.unwrap();

    let order_detail_payload = OrderDetailPayload {
        location: resp.location,
        timestamp: chrono::DateTime::parse_from_rfc3339(&resp.timestamp).unwrap(),
        signature: resp.signature,
        material: resp.material,
        a: resp.data.a,
        b: resp.data.b,
        c: resp.data.c,
        d: resp.data.d,
    };
    
    let insert_result = web::block(move || {
        let mut conn = pool.get().unwrap();
        insert_order_detail(&mut conn, order_detail_payload)
    })
        .await
        .unwrap();
        
    if let Err(insert_err) = insert_result {
        eprintln!("{}", insert_err);
        return HttpResponse::InternalServerError().finish();
    }

    let inserted_order = insert_result.unwrap();

    let order_record = OrderRecord::from(inserted_order);

    HttpResponse::Ok().json(order_record)
}

#[get("/record")]
pub async fn record(pool: web::Data<DbPool>) -> impl Responder {
    use crate::schema::order_details::dsl::*;

    let query_result = web::block(move || {
        let mut conn = pool.get().unwrap();
        order_details.load::<OrderDetail>(&mut conn)
    })
        .await
        .unwrap();

    if let Err(err) = query_result {
        eprintln!("{}", err);
        return HttpResponse::InternalServerError().finish();
    }

    let queried_order_details = query_result.unwrap();

    let order_records: Vec<OrderRecord> = queried_order_details.into_iter().map(|order_detail| OrderRecord::from(order_detail)).collect();

    HttpResponse::Ok().json(order_records)
}