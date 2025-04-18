use super::kernel;
use crate::{errors, mlua, regex, Inject};
use kernel::Kernel;
use mlua::prelude::*;
use regex::Regex;
use std::fs;

#[derive(Clone, Debug)]
pub(crate) struct Host {
    pub model: String,
}

impl Host {
    pub fn new(k: &Kernel) -> Option<Self> {
        if k.name != "Linux" {
            return None;
        }

        let raw = fs::read_to_string("/sys/devices/virtual/dmi/id/product_name").ok()?;
        let cleaned = Self::clean_product_name(&raw);

        if cleaned.is_empty() {
            None
        } else {
            Some(Host { model: cleaned })
        }
    }

    fn clean_product_name(s: &str) -> String {
        // Compile once, statically
        static PATTERN: &[&str] = &[
            "To Be Filled By O.E.M.",
            "Not Applicable",
            "System Product Name",
            "Undefined",
            "Default string",
            "Not Specified",
            "INVALID",
            "�",
        ];
        static REGEX: once_cell::sync::Lazy<Regex> =
            once_cell::sync::Lazy::new(|| Regex::new(r"(?i)To Be Filled.*?").unwrap());

        let mut result = s.trim().to_owned();

        for pat in PATTERN {
            result = result.replace(pat, "");
        }

        result = REGEX.replace_all(&result, "").into_owned();
        result.trim().to_string()
    }
}

impl Inject for Host {
    fn inject(&self, lua: &mut Lua) {
        let globals = lua.globals();
        if let Ok(t) = lua.create_table() {
            if let Err(e) = t.set("model", self.model.as_str()) {
                errors::handle(&format!("{}{}", errors::LUA, e));
            }
            if let Err(e) = globals.set("host", t) {
                errors::handle(&format!("{}{}", errors::LUA, e));
            }
        } else if let Err(e) = lua.create_table() {
            errors::handle(&format!("{}{}", errors::LUA, e));
        }
    }
}
