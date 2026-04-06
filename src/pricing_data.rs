use chrono::DateTime;
use chrono_tz::Tz;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct PricingDataResponse {
    pub purchase_price: Vec<f32>,
    pub taxes: Vec<f32>,
    pub average_purchase_price: f32,
    pub purchasing_fee: f32,
}

#[derive(Debug)]
pub struct PricingData {
    pub date: DateTime<Tz>,
    pub pricings: PricingDataResponse,
}
