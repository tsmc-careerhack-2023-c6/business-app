use chrono::FixedOffset;
use serde::{Deserialize, Serialize};

use crate::schema::order_details;

#[derive(Serialize, Deserialize, Clone)]
pub struct Data {
    pub a: i32,
    pub b: i32,
    pub c: i32,
    pub d: i32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OrderPayload {
    pub location: String,
    pub timestamp: String,
    pub data: Data,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OrderDetailFromInventory {
    pub location: String,
    pub timestamp: String,
    pub signature: String,
    pub material: i32,
    pub data: Data,
}

#[derive(Serialize, Deserialize, Clone, Queryable)]
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

#[derive(Serialize, Deserialize, Clone, Insertable)]
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

#[derive(Serialize, Deserialize, Clone)]
pub struct OrderReport {
    location: String,
    count: i32,
    material: i32,
    data: Data,
}