use serde::Serialize;
use std::collections::HashMap;

pub mod leverancier;
use crate::leverancier::Leverancier;

#[derive(Serialize, Debug)]
struct HaDevice {
    ids: String,
    name: String,
    mf: String,
    mdl: String,
    sw: String,
    sn: String,
    hw: String,
}

#[derive(Serialize, Debug)]
struct HaOrigin {
    name: String,
    sw: String,
    url: String,
}

#[derive(Serialize, Debug)]
struct HaComponent {
    p: String,
    device_class: String,
    unit_of_measurement: String,
    value_template: String,
    unique_id: String,
}

#[derive(Serialize, Debug)]
struct HaDiscoveryPayload {
    dev: HaDevice,
    o: HaOrigin,
    cmps: HashMap<String, HaComponent>,
    state_topic: String,
    qos: u8,
}

#[derive(Serialize, Debug)]
struct HaDiscoveryConfig {
    topic: String,
    payload: HaDiscoveryPayload,
}

pub const TOPIC: &'static str = "dynamic_energy_price";

fn get_ha_device_discovery_payload(leverancier: Leverancier) -> HaDiscoveryPayload {
    // https://www.home-assistant.io/integrations/mqtt/#discovery-topic
    // The discovery topic needs to follow a specific format:
    // <discovery_prefix>/<component>/[<node_id>/]<object_id>/config
    let discovery_prefix = "homeassistant";
    let component = "device";
    let object_id = leverancier as u32;

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

    discovery_payload
}
