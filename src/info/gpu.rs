use std::fs::{self, read_to_string};

use crate::{errors, mlua, Inject};
use mlua::prelude::*;

#[derive(Clone, Debug)]
pub(crate) struct Gpu {
    pub brand: String,
    pub name: String,
}

impl Gpu {
    #[inline]
    pub fn new(name: impl Into<String>, brand: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            brand: brand.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Gpus(pub Vec<Gpu>);

impl Gpus {
    pub fn new() -> Option<Self> {
        let mut gpus = Vec::new();
        let Ok(entries) = fs::read_dir("/sys/class/drm/") else {
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

            let name = format!("PCI ID: {}", device_id);
            gpus.push(Gpu::new(name, brand));
        }

        if gpus.is_empty() {
            None
        } else {
            Some(Gpus(gpus))
        }
    }
}

impl Inject for Gpus {
    fn inject(&self, lua: &mut Lua) {
        let globals = lua.globals();

        match lua.create_table() {
            Ok(gpu_table) => {
                for (i, gpu) in self.0.iter().enumerate() {
                    match lua.create_table() {
                        Ok(t) => {
                            if t.set("name", &*gpu.name).is_err()
                                || t.set("brand", &*gpu.brand).is_err()
                                || gpu_table.raw_set((i + 1) as i64, t).is_err()
                            {
                                errors::handle("Failed to inject GPU info to Lua");
                                panic!();
                            }
                        }
                        Err(e) => {
                            errors::handle(&format!("{}{}", errors::LUA, e));
                            panic!();
                        }
                    }
                }

                if let Err(e) = globals.set("gpus", gpu_table) {
                    errors::handle(&format!("{}{}", errors::LUA, e));
                    panic!();
                }
            }
            Err(e) => {
                errors::handle(&format!("{}{}", errors::LUA, e));
                panic!();
            }
        }
    }
}
