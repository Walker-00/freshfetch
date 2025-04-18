use crate::term_size;
use crate::{errors, mlua, Inject};
use mlua::prelude::*;

pub(crate) struct Terminal {
    pub width: i32,
    pub height: i32,
}

impl Terminal {
    #[inline(always)]
    pub fn new() -> Self {
        match term_size::dimensions() {
            Some((w, h)) => Terminal {
                width: w as i32,
                height: h as i32,
            },
            None => {
                errors::handle("Failed to get terminal dimensions.");
                Terminal {
                    width: 0,
                    height: 0,
                }
            }
        }
    }
}

impl Inject for Terminal {
    #[inline(always)]
    fn inject(&self, lua: &mut Lua) {
        if let Ok(table) = lua.create_table() {
            let _ = table
                .set("width", self.width)
                .map_err(|e| errors::handle(&format!("{}{}", errors::LUA, e)));
            let _ = table
                .set("height", self.height)
                .map_err(|e| errors::handle(&format!("{}{}", errors::LUA, e)));
            let _ = lua
                .globals()
                .set("terminal", table)
                .map_err(|e| errors::handle(&format!("{}{}", errors::LUA, e)));
        } else {
            errors::handle("Failed to create Lua table for terminal.");
        }
    }
}
