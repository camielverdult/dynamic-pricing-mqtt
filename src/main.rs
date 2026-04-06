extern crate chrono;

use chrono::{DateTime, Datelike, Timelike};
use chrono_tz::Tz;
use reqwest;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio;
use tokio::time;

use dynamic_pricing_mqtt::{
    Leverancier, config,
    home_assistant::get_ha_device_discovery_payload,
    pricing_data::{PricingData, PricingDataResponse},
};

const MQTT_SETTLE_DELAY: u64 = 5;

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
        .await?;

    Ok(PricingData {
        date: *time,
        pricings: resp,
    })
}

fn index_at_time(time: &DateTime<Tz>) -> i32 {
    let hour = time.hour(); // 0-23
    let minute = time.minute(); // 0-59

    // Integer division by 15 gives us 0, 1, 2, or 3!
    let quarter = minute / 15;

    (hour * 4 + quarter) as i32
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
    let config = config::get_config();

    println!("Starting with config: {:#?}", config);

    let req_client = reqwest::Client::new();

    let mut mqttoptions = MqttOptions::new("rumqtt-async", config.host.to_string(), config.port);

    if !config.username.is_empty() || !config.password.is_empty() {
        mqttoptions.set_credentials(config.username, config.password);
    }
    mqttoptions.set_keep_alive(Duration::from_secs(MQTT_SETTLE_DELAY));

    let (mqtt_client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    let error_state = Arc::new(AtomicBool::new(false));
    let error_state_clone = Arc::clone(&error_state);

    // Spawn a background task to constantly poll the event loop
    tokio::spawn(async move {
        loop {
            // This actually drives the connection, sends packets, and receives ACKs
            if let Err(e) = eventloop.poll().await {
                error_state_clone.store(true, Ordering::Relaxed);
                println!("MQTT connection error: {:?}", e);
                // Backoff a bit before retrying the poll on error
                time::sleep(Duration::from_secs(1)).await;
            } else {
                // If poll succeeds, clear the error flag
                error_state_clone.store(false, Ordering::Relaxed);
            }
        }
    });

    time::sleep(Duration::from_secs(MQTT_SETTLE_DELAY)).await;

    let discovery_payload = get_ha_device_discovery_payload(&config.leverancier);
    let json_payload = serde_json::to_string(&discovery_payload.payload).unwrap();

    while error_state.load(Ordering::Relaxed) {
        println!("Cannot send discovery payload because of MQTT error");
        time::sleep(Duration::from_secs(MQTT_SETTLE_DELAY)).await;
    }

    print!("Sending discovery payload:");
    println!("{:#?}", discovery_payload);
    mqtt_client
        .publish(
            discovery_payload.topic,
            QoS::AtLeastOnce,
            true,
            json_payload,
        )
        .await
        .unwrap();

    // Wait after publishing discovery data before continuing with sending prices.
    time::sleep(Duration::from_secs(MQTT_SETTLE_DELAY)).await;

    let mut last_data_fetched = chrono::Utc::now().with_timezone(&config.timezone);

    // initialise data without any value, we will fetch it in the loop
    let mut data = get_data(&req_client, &last_data_fetched, &config.leverancier)
        .await
        .unwrap();

    let price_topic = format!("{}/now", config::TOPIC);

    loop {
        while error_state.load(Ordering::Relaxed) {
            println!("Cannot publish price data because of MQTT error");
            time::sleep(Duration::from_secs(MQTT_SETTLE_DELAY)).await;
        }

        let now = chrono::Utc::now().with_timezone(&config.timezone);

        if last_data_fetched.day() != now.day() {
            data = get_data(&req_client, &now, &config.leverancier)
                .await
                .unwrap();
            last_data_fetched = chrono::Utc::now().with_timezone(&config.timezone);
        }

        let price_now = get_price_at_time(&data.pricings, &now).unwrap();

        println!("{:02}:{:02} = €{}", now.hour(), now.minute(), price_now);

        let price_payload = format!(r#"{{"price": {}}}"#, price_now);

        println!("Sending payload {:#?}", price_payload);

        mqtt_client
            .publish(&price_topic, QoS::AtLeastOnce, false, price_payload)
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
