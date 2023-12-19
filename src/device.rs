use std::collections::HashMap;

use std::sync::OnceLock;

use serde::Deserialize;

use serde::Serialize;

use serde_json::Value;

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewPort {
    pub width: u32,
    pub height: u32,
    pub deviceScaleFactor: f32,
    pub isMobile: bool,
    pub hasTouch: bool,
    pub isLandscape: bool,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceJson {
    pub id: String,
    pub name: String,
    pub userAgent: String,
    pub viewport: ViewPort,
}

pub static DEVICE_LIST: OnceLock<HashMap<String, DeviceJson>> = OnceLock::new();

pub fn init_device_list() {
    let inner = || {
        let buffer = include_str!("resource/device.json");
        let all: Value = serde_json::from_str(buffer).unwrap();
        let d_vec = all["devices"].as_array().unwrap().to_owned();
        let mut dmap = HashMap::new();

        let _ = d_vec
            .into_iter()
            .map(|value| {
                let result: DeviceJson = serde_json::from_value(value).unwrap();
                let id = result.id.clone();
                dmap.insert(id, result);
            })
            .collect::<Vec<()>>();

        dmap
    };

    DEVICE_LIST.get_or_init(inner);
}
