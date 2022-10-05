use serde::Deserialize;

#[derive(Deserialize)]
pub struct Network {
    pub bridge_name: String,
}
