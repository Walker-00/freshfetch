use crate::{cmd_lib::run_fun, errors, mlua, Inject};
use mlua::prelude::*;
use std::env;

use super::{distro::Distro, kernel::Kernel};

pub(crate) struct De(pub String, pub String);

impl De {
    #[inline(always)]
    pub fn new(k: &Kernel, d: &Distro) -> Option<Self> {
        let name = match k.name.as_str() {
            "Mac OS X" | "macOS" => return Some(De("Aqua".into(), String::new())),
            _ if d.short_name.starts_with("Windows") => {
                if d.short_name.starts_with("Windows 8") || d.short_name.starts_with("Windows 10") {
                    "Modern UI/Metro"
                } else {
                    "Aero"
                }
            }
            _ => {
                if env::var("DESKTOP_SESSION").is_ok_and(|v| v == "regolith") {
                    "Regolith"
                } else if let Ok(current) = env::var("XDG_CURRENT_DESKTOP") {
                    return Some(De(current.replace("X-", ""), String::new()));
                } else if env::var("GNOME_DESKTOP_SESSION_ID").is_ok() {
                    "GNOME"
                } else if env::var("MATE_DESKTOP_SESSION_ID").is_ok() {
                    "MATE"
                } else if env::var("TDE_FULL_SESSION").is_ok() {
                    "Trinity"
                } else {
                    return None;
                }
            }
        };

        let name = name.replace("KDE", "Plasma");

        let version = match name.as_str() {
            n if n.starts_with("Plasma") => run_fun!(plasmashell - -version)
                .ok()
                .unwrap_or_default()
                .replace("plasmashell ", "")
                .replace('\n', ""),
            n if n.starts_with("MATE") => {
                run_fun!(mate - session - -version).ok().unwrap_or_default()
            }
            n if n.starts_with("Xfce") => run_fun!(xfce4 - session - -version)
                .ok()
                .unwrap_or_default(),
            n if n.starts_with("GNOME") => {
                run_fun!(gnome - shell - -version).ok().unwrap_or_default()
            }
            n if n.starts_with("Cinnamon") => {
                run_fun!(cinnamon - -version).ok().unwrap_or_default()
            }
            n if n.starts_with("Budgie") => run_fun!(budgie - desktop - -version)
                .ok()
                .unwrap_or_default(),
            n if n.starts_with("LXQt") => {
                run_fun!(lxqt - session - -version).ok().unwrap_or_default()
            }
            n if n.starts_with("Unity") => run_fun!(unity - -version).ok().unwrap_or_default(),
            _ => String::new(),
        };

        Some(De(name, version))
    }
}

impl Inject for De {
    #[inline(always)]
    fn inject(&self, lua: &mut Lua) {
        if let Ok(table) = lua.create_table() {
            let _ = table
                .set("name", self.0.as_str())
                .map_err(|e| errors::handle(&format!("{}{}", errors::LUA, e)));
            let _ = table
                .set("version", self.1.as_str())
                .map_err(|e| errors::handle(&format!("{}{}", errors::LUA, e)));
            let _ = lua
                .globals()
                .set("de", table)
                .map_err(|e| errors::handle(&format!("{}{}", errors::LUA, e)));
        } else {
            errors::handle("Failed to create Lua table for DE.");
        }
    }
}
