use serde::Deserialize;

#[derive(Deserialize)]
pub struct NetworkInterface {
    #[serde(rename = "network")]
    pub network_id: String,
    pub mac_address: String,
    #[serde(default = "default_network_interface_model")]
    pub model: String,
    pub tap_name: String,
}

fn default_network_interface_model() -> String {
    "virtio-net-pci-non-transitional".to_string()
}
