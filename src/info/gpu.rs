use std::fs::{self, read_to_string};
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Gpu {
    pub brand: String,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct Gpus(pub Vec<Gpu>);

impl Gpus {
    pub fn new() -> Option<Self> {
        let mut gpus = Vec::new();

        let drm_dir = PathBuf::from("/sys/class/drm/");
        let Ok(entries) = fs::read_dir(drm_dir) else {
            return None;
        };

        for entry in entries.flatten() {
            let path = entry.path();

            if !path.file_name()?.to_string_lossy().starts_with("card") {
                continue;
            }

            let device_path = path.join("device");
            let vendor_path = device_path.join("vendor");
            let device_path_file = device_path.join("device");

            let vendor_id = read_to_string(vendor_path).ok()?.trim().to_string();
            let device_id = read_to_string(device_path_file).ok()?.trim().to_string();

            let brand = match vendor_id.as_str() {
                "0x8086" => "Intel",
                "0x10de" => "NVIDIA",
                "0x1002" | "0x1022" => "AMD",
                _ => "Unknown",
            };

            gpus.push(Gpu {
                brand: brand.to_string(),
                name: format!("Device ID: {}", device_id),
            });
        }

        if gpus.is_empty() {
            None
        } else {
            Some(Gpus(gpus))
        }
    }
}
