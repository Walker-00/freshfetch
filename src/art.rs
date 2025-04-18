use crate::{assets, errors, info, mlua, regex, Arguments, Inject};
use assets::{ascii_art, ANSI, PRINT};
use info::distro::DistroColors;
use info::Info;

use std::path::Path;
use std::{env, fs};

use mlua::prelude::*;
use regex::Regex;

pub(crate) struct Art {
    inner: String,
    width: i32,
    height: i32,
}

impl Art {
    pub fn new(info: &mut Info, arguments: &Arguments) -> Self {
        let mut art = Art {
            inner: String::new(),
            width: 0,
            height: 0,
        };

        art.inner = if let Some(distro_name) = &arguments.ascii_distro {
            let (ascii, colors) = ascii_art::get(distro_name);
            info.distro.colors = DistroColors::from(colors);
            ascii.into()
        } else {
            let path = Path::new("/home/")
                .join(env::var("USER").unwrap_or_default())
                .join(".config/freshfetch/art.lua");

            if path.exists() {
                match fs::read_to_string(&path) {
                    Ok(file) => Self::exec_lua(&file).unwrap_or_else(|e| {
                        errors::handle(&format!("{}{}", errors::LUA, e));
                        panic!()
                    }),
                    Err(e) => {
                        errors::handle(&format!(
                            "{}~/.config/freshfetch/art.lua{}{}",
                            errors::io::READ.0,
                            errors::io::READ.1,
                            e
                        ));
                        panic!()
                    }
                }
            } else {
                let (ascii, colors) = ascii_art::get(&info.distro.short_name);
                info.distro.colors = DistroColors::from(colors);
                ascii.into()
            }
        };

        art.measure();
        art
    }

    #[inline(always)]
    fn exec_lua(script: &str) -> Result<String, String> {
        let lua = Lua::new();

        lua.load(PRINT)
            .exec()
            .map_err(|e| format!("{}{}", errors::LUA, e))?;
        lua.load(ANSI)
            .exec()
            .map_err(|e| format!("{}{}", errors::LUA, e))?;
        lua.load(script)
            .exec()
            .map_err(|e| format!("{}{}", errors::LUA, e))?;

        let bind = lua
            .globals()
            .get::<_, String>("__freshfetch__")
            .map_err(|e| format!("{}{}", errors::LUA, e));

        bind
    }

    #[inline(always)]
    fn measure(&mut self) {
        static STRIP_ANSI: once_cell::sync::Lazy<Regex> =
            once_cell::sync::Lazy::new(|| Regex::new(r"(?i)\x1B\[(?:[\d;]*\d+[a-z])").unwrap());

        let mut w = 0;
        let mut h = 0;

        for line in STRIP_ANSI.replace_all(&self.inner, "").lines() {
            let len = line.len(); // faster than collecting chars unless you need Unicode width
            if len > w {
                w = len;
            }
            h += 1;
        }

        self.width = w as i32;
        self.height = h;
    }
}

impl Inject for Art {
    #[inline(always)]
    fn inject(&self, lua: &mut Lua) {
        let globals = lua.globals();
        if globals.set("art", self.inner.as_str()).is_err()
            || globals.set("artWidth", self.width).is_err()
            || globals.set("artHeight", self.height).is_err()
        {
            errors::handle("Failed to inject art into Lua globals.");
        }
    }
}
