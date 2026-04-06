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

pub fn get_config() -> Config {
    let tz_str = std::env::var("TIMEZONE").unwrap_or_else(|_| "Europe/Amsterdam".to_string());

    let leverancier_str = std::env::var("LEVERANCIER").unwrap_or_else(|_| "Generic".to_string());

    let leverancier = match leverancier_str.as_str() {
        "Generic" => Leverancier::Generic,
        "All_in_power" => Leverancier::All_in_power,
        "ANWB_Energie" => Leverancier::ANWB_Energie,
        "BudgetEnergie" => Leverancier::BudgetEnergie,
        "CoolblueEnergie" => Leverancier::CoolblueEnergie,
        "DeltaEnergie" => Leverancier::DeltaEnergie,
        "easyEnergy" => Leverancier::easyEnergy,
        "Eneco" => Leverancier::Eneco,
        "EnergieVanOns" => Leverancier::EnergieVanOns,
        "Energiedirect" => Leverancier::Energiedirect,
        "Energiek" => Leverancier::Energiek,
        "EnergyZero" => Leverancier::EnergyZero,
        "Engie" => Leverancier::Engie,
        "Essent" => Leverancier::Essent,
        "FrankEnergie" => Leverancier::FrankEnergie,
        "GroeneStroomLokaal" => Leverancier::GroeneStroomLokaal,
        "NextEnergy" => Leverancier::NextEnergy,
        "Oxxio" => Leverancier::Oxxio,
        "Tibber" => Leverancier::Tibber,
        "Vandebron" => Leverancier::Vandebron,
        "Vattenfall" => Leverancier::Vattenfall,
        "Vrijopnaam" => Leverancier::Vrijopnaam,
        "Zonneplan" => Leverancier::Zonneplan,
        _ => panic!("Invalid LEVERANCIER value: {}", leverancier_str),
    };

    Config {
        timezone: tz_str.parse().unwrap_or(chrono_tz::Europe::Amsterdam),
        host: std::env::var("MQTT_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
        port: std::env::var("MQTT_PORT")
            .unwrap_or_else(|_| 1883.to_string())
            .parse::<u16>()
            .expect("MQTT_PORT must be a valid unsigned 16-bit integer"),
        username: std::env::var("MQTT_USERNAME").unwrap_or_else(|_| "".to_string()),
        password: std::env::var("MQTT_PASSWORD").unwrap_or_else(|_| "".to_string()),
        leverancier: leverancier,
    }
}
