extern crate chrono;

use chrono::{DateTime, Datelike, Local, Timelike};
use chrono_tz::Tz;
use reqwest;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use serde::Deserialize;
use std::time::Duration;
use tokio;
use tokio::time;


#[derive(Deserialize, Debug)]
struct PricingDataResponse {
    purchase_price: Vec<f32>,
    taxes: Vec<f32>,
    average_purchase_price: f32,
    purchasing_fee: f32,
}

#[derive(Debug)]
struct PricingData {
    date: DateTime<Tz>,
    pricings: PricingDataResponse,
}
use dynamic_pricing_mqtt::{Leverancier, TOPIC};

#[derive(Debug)]
struct Config {
    timezone: Tz,
    host: String,
    port: u16,
    username: String,
    password: String,
    leverancier: Leverancier,
}

fn get_config() -> Config {
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

async fn get_data(
    client: &reqwest::Client,
    time: &DateTime<Tz>,
    leverancier: &Leverancier,
) -> Result<PricingData, reqwest::Error> {
    let date_string = time.format("%Y-%m-%d");

    let url = format!(
        "https://www.stroomperuur.nl/ajax/tarieven.php?leverancier={}&datum={}&kwartier=1",
        *leverancier as u8, date_string
    );

    let resp = client
        .get(url)
        .send()
        .await?
        .json::<PricingDataResponse>()
        .await;

    Ok(PricingData {
        date: *time,
        pricings: resp?,
    })
}

fn index_at_time(time: &DateTime<Tz>) -> i32 {
    let hour = time.hour(); // 0-23
    let minute = time.minute(); // 0-59

    // Integer division by 15 gives us 0, 1, 2, or 3!
    let quarter = minute / 15;

    let index = (hour * 4 + quarter) as i32;

    index
}

fn get_price_at_time(prices: &PricingDataResponse, time: &DateTime<Tz>) -> Option<f32> {
    let index = index_at_time(time) as usize;

    assert!(prices.purchase_price.len() >= index);
    assert!(prices.taxes.len() >= index);

    let inkoop_prijs = prices.purchase_price.get(index).copied()?;

    let inkoop_vergoeding = prices.purchasing_fee;
    let taxes = prices.taxes.get(index).copied()?;

    let cent_price = inkoop_prijs + inkoop_vergoeding + taxes;

    Some(cent_price)
}

#[tokio::main]
async fn main() {
    let config = get_config();

    println!("Starting with config: {:#?}", config);

    let req_client = reqwest::Client::new();

    let mut mqttoptions = MqttOptions::new("rumqtt-async", config.host.to_string(), config.port);

    if !config.username.is_empty() || !config.password.is_empty() {
        mqttoptions.set_credentials(config.username, config.password);
    }
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (mqtt_client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    // Spawn a background task to constantly poll the event loop
    tokio::spawn(async move {
        loop {
            // This actually drives the connection, sends packets, and receives ACKs
            if let Err(e) = eventloop.poll().await {
                println!("MQTT connection error: {:?}", e);
                // Backoff a bit before retrying the poll on error
                time::sleep(Duration::from_secs(1)).await;
            }
        }
    });

    let last_data_fetched = chrono::Utc::now().with_timezone(&config.timezone);

    // initialise data without any value, we will fetch it in the loop
    let mut data = get_data(&req_client, &last_data_fetched, &config.leverancier)
        .await
        .unwrap();

    loop {
        let now = chrono::Utc::now().with_timezone(&config.timezone);

        if last_data_fetched.day() != now.day() {
            data = get_data(&req_client, &now, &config.leverancier)
                .await
                .unwrap();
        }

        let price_now = get_price_at_time(&data.pricings, &now).unwrap();

        // format hour:minute with leading zeros
        let hour = now.hour();
        let minute = now.minute();

        let hour_str = if hour < 10 {
            format!("0{}", hour)
        } else {
            hour.to_string()
        };

        let minute_str = if minute < 10 {
            format!("0{}", minute)
        } else {
            minute.to_string()
        };

        println!("{}:{} = €{}", hour_str, minute_str, price_now);

        mqtt_client
            .publish(
                format!("{}/now", TOPIC),
                QoS::AtLeastOnce,
                false,
                price_now.to_string(),
            )
            .await
            .unwrap();

        // Calculate time to sleep until minute value is incremented by 1 and seconds are 0
        let next_minute = now + chrono::Duration::minutes(1);
        let next_minute = next_minute
            .with_second(0)
            .unwrap()
            .with_nanosecond(0)
            .unwrap();

        // Add a 10ms safety buffer to ensure we completely cross the minute boundary
        let time_until_next = next_minute - chrono::Utc::now().with_timezone(&config.timezone)
            + chrono::Duration::milliseconds(10);

        let ms = time_until_next.num_milliseconds();
        if ms > 0 {
            let sleep_time = Duration::from_millis(ms as u64);
            println!("Sleeping {} ms", ms);
            time::sleep(sleep_time).await;
        }
    }
}
