use crate::mlua;
use crate::regex;

// use super::kernel;
use crate::errors;

use std::fs;
use std::path::Path;
use std::sync::OnceLock;

use mlua::prelude::*;
use regex::Regex;

use crate::Inject;

static CLEAN_NAME_REGEXES: OnceLock<Vec<Regex>> = OnceLock::new();

#[derive(Debug)]
pub(crate) struct Cpu {
    pub name: String,
    pub full_name: String,
    pub freq: f32,
    pub cores: i32,
}

impl Cpu {
    pub fn new() -> Option<Self> {
        // Ensure this only runs on supported OS
        if !cfg!(target_os = "linux") && !cfg!(target_os = "windows") {
            return None;
        }

        let cpu_info = fs::read_to_string("/proc/cpuinfo").ok()?;
        let mut name = None;
        let mut freq = None;
        let mut cores = 0;

        for line in cpu_info.lines() {
            if name.is_none()
                && (line.starts_with("model name")
                    || line.starts_with("Hardware")
                    || line.starts_with("Processor")
                    || line.starts_with("cpu model")
                    || line.starts_with("chip type")
                    || line.starts_with("cpu type"))
            {
                if let Some((_, val)) = line.split_once(":") {
                    name = Some(val.trim().to_string());
                }
            } else if freq.is_none() && (line.starts_with("cpu MHz") || line.starts_with("clock")) {
                if let Some((_, val)) = line.split_once(":") {
                    let cleaned = val.trim().replace("MHz", "");
                    freq = cleaned.parse::<f32>().ok().map(|f| f / 1000.0);
                }
            } else if line.starts_with("processor") {
                cores += 1;
            }

            if name.is_some() && freq.is_some() {
                continue;
            }
        }

        // Try cpufreq fallback if needed
        if freq.is_none() && Path::new("/sys/devices/system/cpu/cpu0/cpufreq/").exists() {
            for file in [
                "/sys/devices/system/cpu/cpu0/cpufreq/bios_limit",
                "/sys/devices/system/cpu/cpu0/cpufreq/scaling_max_freq",
                "/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq",
            ] {
                if let Ok(val) = fs::read_to_string(file) {
                    if let Ok(parsed) = val.trim().parse::<f32>() {
                        freq = Some(parsed / 1000.0);
                        break;
                    }
                }
            }
        }

        let (full_name, freq, cores) = (name?, freq?, cores);

        let cleaned_name = Self::clean_name(&full_name);

        Some(Self {
            name: cleaned_name,
            full_name,
            freq,
            cores,
        })
    }

    fn clean_name(original: &str) -> String {
        let mut name = original.to_string();

        for pattern in CLEAN_NAME_REGEXES.get_or_init(|| {
            vec![
                Regex::new(r"(?i)\(TM\)|\(tm\)").unwrap(),
                Regex::new(r"(?i)\(R\)|\(r\)").unwrap(),
                Regex::new(r"\bCPU\b").unwrap(),
                Regex::new(r"\bProcessor\b").unwrap(),
                Regex::new(r"\b\w+-Core\b").unwrap(),
                Regex::new(r"(?i), .*? Compute Cores").unwrap(),
                Regex::new(r#"(?i)\("AuthenticAMD".*?\)"#).unwrap(),
                Regex::new(r#"(?i)with Radeon .*? Graphics"#).unwrap(),
                Regex::new(r"(?i)altivec supported").unwrap(),
                Regex::new(r"(?i)Technologies, Inc").unwrap(),
                Regex::new(r"(?i)FPU.*?").unwrap(),
                Regex::new(r"(?i)Chip Revision.*?").unwrap(),
            ]
        }) {
            name = pattern.replace_all(&name, "").to_string();
        }

        name.replace("Core2", "Core 2")
            .replace("Cores ", " ")
            .trim()
            .to_string()
    }
}

impl Inject for Cpu {
    fn inject(&self, lua: &mut Lua) {
        let globals = lua.globals();
        let t = lua.create_table().unwrap();

        t.set("name", &*self.name).unwrap_or_else(|e| {
            errors::handle(&format!("{}{}", errors::LUA, e));
            panic!();
        });
        t.set("fullName", &*self.full_name).unwrap_or_else(|e| {
            errors::handle(&format!("{}{}", errors::LUA, e));
            panic!();
        });
        t.set("freq", self.freq).unwrap_or_else(|e| {
            errors::handle(&format!("{}{}", errors::LUA, e));
            panic!();
        });
        t.set("cores", self.cores).unwrap_or_else(|e| {
            errors::handle(&format!("{}{}", errors::LUA, e));
            panic!();
        });

        globals.set("cpu", t).unwrap_or_else(|e| {
            errors::handle(&format!("{}{}", errors::LUA, e));
            panic!();
        });
    }
}
