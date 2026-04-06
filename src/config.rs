use chrono_tz::Tz;

use crate::leverancier::Leverancier;

pub const TOPIC: &'static str = "dynamic_energy_price";

#[derive(Debug)]
pub struct Config {
    pub timezone: Tz,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub leverancier: Leverancier,
}
