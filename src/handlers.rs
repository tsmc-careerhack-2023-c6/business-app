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
    
    // use reqwest to send order to inventory service
    // and get the response (OrderDetailFromInventory)
    // and flatten it to (OrderDetailPayload)
    // and insert it to database
    // and return the response (OrderDetail)
    
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
        
    match insert_result {
        Ok(order_detail) => HttpResponse::Ok().json(order_detail),
        Err(e) => {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}