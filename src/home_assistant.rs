use serde::Serialize;
use std::collections::HashMap;

use crate::{config::TOPIC, leverancier::Leverancier};

#[derive(Serialize, Debug)]
pub struct HaDevice {
    pub ids: String,
    pub name: String,
    pub mf: String,
    pub mdl: String,
    pub sw: String,
    pub sn: String,
    pub hw: String,
}

#[derive(Serialize, Debug)]
pub struct HaOrigin {
    pub name: String,
    pub sw: String,
    pub url: String,
}

#[derive(Serialize, Debug)]
pub struct HaComponent {
    pub p: String,
    pub device_class: String,
    pub unit_of_measurement: String,
    pub value_template: String,
    pub unique_id: String,
}

#[derive(Serialize, Debug)]
pub struct HaDiscoveryPayload {
    pub dev: HaDevice,
    pub o: HaOrigin,
    pub cmps: HashMap<String, HaComponent>,
    pub state_topic: String,
    pub qos: u8,
}

#[derive(Serialize, Debug)]
pub struct HaDiscoveryConfig {
    pub topic: String,
    pub payload: HaDiscoveryPayload,
}

pub fn get_ha_device_discovery_payload(leverancier: &Leverancier) -> HaDiscoveryConfig {
    // https://www.home-assistant.io/integrations/mqtt/#discovery-topic
    // The discovery topic needs to follow a specific format:
    // <discovery_prefix>/<component>/[<node_id>/]<object_id>/config
    let discovery_prefix = "homeassistant";
    let component = "device";
    let object_id = *leverancier as u32;

    let topic = format!("{discovery_prefix}/{component}/{object_id}/config");

    let device_id = format!("{}_01", TOPIC);
    let state_topic = format!("{}/now", TOPIC);

    let mut components = HashMap::new();

    // Add your components
    components.insert(
        TOPIC.to_string(),
        HaComponent {
            p: "sensor".to_string(),
            device_class: "monetary".to_string(),
            unit_of_measurement: "EUR".to_string(),
            value_template: "{{ value_json.price }}".to_string(),
            unique_id: "energy_price_now_01".to_string(),
        },
    );

    let discovery_payload = HaDiscoveryPayload {
        dev: HaDevice {
            ids: device_id,
            name: "Dynamic Energy Pricing".to_string(),
            mf: "Camiel".to_string(),
            mdl: "MQTT Bridge".to_string(),
            sw: "1.0".to_string(),
            sn: "dyn_price_01".to_string(),
            hw: "1.0".to_string(),
        },
        o: HaOrigin {
            name: "dynamic-pricing-mqtt".to_string(),
            sw: "1.0".to_string(),
            url: "https://github.com/your-username/dynamic-pricing-mqtt".to_string(),
        },
        cmps: components,
        state_topic: state_topic.to_string(), // Make sure this matches your script!
        qos: 1,                               // Usually QoS 1 is preferred for discovery
    };

    HaDiscoveryConfig {
        topic: topic,
        payload: discovery_payload,
    }
}
