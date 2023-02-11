use std::sync::Arc;

use async_nats::Client;
use chrono::FixedOffset;
use serde::{Deserialize, Serialize};
use chrono_tz::Asia;

use crate::schema::order_details;

pub struct AppState {
    pub db_pool: Arc<crate::DbPool>,
    pub nats_client: Arc<Client>,
    pub nats_topic_prefix: String,
    pub num_logical_processors: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Data {
    pub a: i32,
    pub b: i32,
    pub c: i32,
    pub d: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrderPayload {
    pub location: String,
    pub timestamp: String,
    pub data: Data,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrderDetailFromInventory {
    pub location: String,
    pub timestamp: String,
    pub signature: String,
    pub material: i32,
    pub data: Data,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable)]
pub struct OrderDetail {
    pub id: i32,
    pub location: String,
    pub timestamp: chrono::NaiveDateTime,
    pub signature: String,
    pub material: i32,
    pub a: i32,
    pub b: i32,
    pub c: i32,
    pub d: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Insertable)]
#[diesel(table_name = order_details)]
pub struct OrderDetailPayload {
    pub location: String,
    pub timestamp: chrono::DateTime<FixedOffset>,
    pub signature: String,
    pub material: i32,
    pub a: i32,
    pub b: i32,
    pub c: i32,
    pub d: i32,
}

impl From<OrderDetailFromInventory> for OrderDetailPayload {
    fn from(order_detail_from_inventory: OrderDetailFromInventory) -> Self {
        OrderDetailPayload {
            location: order_detail_from_inventory.location,
            timestamp: chrono::DateTime::parse_from_rfc3339(&order_detail_from_inventory.timestamp).unwrap(),
            signature: order_detail_from_inventory.signature,
            material: order_detail_from_inventory.material,
            a: order_detail_from_inventory.data.a,
            b: order_detail_from_inventory.data.b,
            c: order_detail_from_inventory.data.c,
            d: order_detail_from_inventory.data.d,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrderRecord {
    pub location: String,
    pub timestamp: String,
    pub signature: String,
    pub material: i32,
    pub a: i32,
    pub b: i32,
    pub c: i32,
    pub d: i32,
}

impl From<OrderDetail> for OrderRecord {
    fn from(order_detail: OrderDetail) -> Self {
        OrderRecord {
            location: order_detail.location,
            timestamp: (order_detail.timestamp + chrono::Duration::hours(8)).and_local_timezone(Asia::Taipei).unwrap().to_rfc3339(),
            signature: order_detail.signature,
            material: order_detail.material,
            a: order_detail.a,
            b: order_detail.b,
            c: order_detail.c,
            d: order_detail.d,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrderReport {
    pub location: String,
    pub date: String,
    pub count: usize,
    pub material: i32,
    pub a: i32,
    pub b: i32,
    pub c: i32,
    pub d: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrderQuery {
    pub location: String,
    pub date: String,
}