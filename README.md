# Dynamic pricing MQTT

Usage:
```bash
cargo build --release

# For testing purposes, you can use the public MQTT broker at test.mosquitto.org.
# In production, you should use your own MQTT broker and set the MQTT_HOST environment variable accordingly.
export MQTT_HOST="test.mosquitto.org"

# Use Zonneplan as the supplier, this determines the purchasing fee used in the dynamic pricing calculation.
# In production, you should set this to your actual supplier.
export LEVERANCIER="Zonneplan"

./target/release/dynamic-pricing-mqtt
```
