use super::kernel;
use crate::{errors, mlua, Inject};

use kernel::Kernel;
use mlua::prelude::*;
use std::{env, path::Path, process::Command};

pub(crate) struct Shell {
    pub name: String,
    pub version: String,
}

impl Shell {
    pub fn new(k: &Kernel) -> Self {
        if !matches!(k.name.as_str(), "Linux" | "BSD" | "Windows") {
            return Shell {
                name: String::new(),
                version: String::new(),
            };
        }

        let shell_env = match env::var("SHELL") {
            Ok(val) => val,
            Err(_) => {
                return Shell {
                    name: String::new(),
                    version: String::new(),
                }
            }
        };

        let shell_name = Path::new(&shell_env)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();

        let version = if shell_name == "zsh" {
            match Command::new("zsh")
                .arg("-c")
                .arg("printf $ZSH_VERSION")
                .output()
            {
                Ok(output) if output.status.success() => {
                    String::from_utf8_lossy(&output.stdout).trim().to_string()
                }
                _ => String::new(),
            }
        } else {
            String::new()
        };

        Shell {
            name: shell_name,
            version,
        }
    }
}

impl Inject for Shell {
    fn inject(&self, lua: &mut Lua) {
        if let Ok(table) = lua.create_table() {
            if let Err(e) = table.set("name", &*self.name) {
                errors::handle(&format!("{}{}", errors::LUA, e));
            }
            if let Err(e) = table.set("version", &*self.version) {
                errors::handle(&format!("{}{}", errors::LUA, e));
            }
            if let Err(e) = lua.globals().set("shell", table) {
                errors::handle(&format!("{}{}", errors::LUA, e));
            }
        }
    }
}

