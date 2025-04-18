use crate::{errors, mlua, Inject};
use mlua::prelude::*;

use super::{
    kernel::Kernel,
    utils::{Grep, PsAux},
};

use std::{env, fs, process::Command};

pub(crate) struct Wm(pub String);

impl Wm {
    #[inline(always)]
    pub fn new(k: &Kernel) -> Option<Self> {
        if env::var("WAYLAND_DISPLAY").is_ok() {
            let wm = PsAux::new().grep(Grep {
                max: Some(1),
                search: None,
                searches: Some(
                    vec![
                        "arcan",
                        "asc",
                        "clayland",
                        "dwc",
                        "fireplace",
                        "gnome-shell",
                        "greenfield",
                        "grefsen",
                        "kwin",
                        "lipstick",
                        "maynard",
                        "mazecompositor",
                        "motorcar",
                        "orbital",
                        "orbment",
                        "perceptia",
                        "rustland",
                        "sway",
                        "ulubis",
                        "velox",
                        "wavy",
                        "way-cooler",
                        "wayfire",
                        "wayhouse",
                        "westeros",
                        "westford",
                        "weston",
                    ]
                    .into_iter()
                    .map(String::from)
                    .collect(),
                ),
                only_matching: Some(true),
            });

            return wm.into_iter().next().map(Wm);
        }

        if env::var("DISPLAY").is_ok()
            && !matches!(k.name.as_str(), "macOS" | "Mac OS X" | "FreeMiNT")
        {
            if let Ok(output) = Command::new("bash")
                .arg("-c")
                .arg(r#"id=$(xprop -root -notype _NET_SUPPORTING_WM_CHECK) && id=${id##* } && wm=$(xprop -id "$id" -notype -len 100 -f _NET_WM_NAME 8t) && wm=${wm/*WM_NAME = } && wm=${wm/\"} && wm=${wm/\"*} && printf $wm"#)
                .output()
            {
                if let Ok(stdout) = String::from_utf8(output.stdout.clone()) {
                    return Some(Wm(stdout));
                } else {
                    errors::handle(&format!(
                        "{}{:?}{}String{}{}",
                        errors::PARSE.0,
                        output.stdout,
                        errors::PARSE.1,
                        errors::PARSE.2,
                        " - Invalid UTF-8"
                    ));
                    panic!();
                }
            } else {
                errors::handle("Failed to run xprop command for WM detection.");
                panic!();
            }
        }

        match k.name.as_str() {
            "Mac OS X" | "macOS" => {
                let res = PsAux::new().grep(Grep {
                    max: Some(1),
                    search: None,
                    searches: Some(
                        vec![
                            "spectacle",
                            "amethyst",
                            "kwm",
                            "chunkwm",
                            "abai",
                            "rectangle",
                        ]
                        .into_iter()
                        .map(String::from)
                        .collect(),
                    ),
                    only_matching: Some(true),
                });

                Some(Wm(res
                    .into_iter()
                    .next()
                    .unwrap_or_else(|| "Quartz Compositor".into())))
            }

            "Windows" => {
                let mut res = PsAux::new().grep(Grep {
                    max: Some(1),
                    search: None,
                    searches: Some(
                        vec!["bugn", "Windawesome", "blackbox", "emerge", "litestep"]
                            .into_iter()
                            .map(String::from)
                            .collect(),
                    ),
                    only_matching: Some(true),
                });

                let name = if let Some(first) = res.get_mut(0) {
                    if first == "blackbox" {
                        *first = "bbLean (Blackbox)".into();
                    }
                    format!("{}, Explorer", first)
                } else {
                    "Explorer".into()
                };

                Some(Wm(name))
            }

            "FreeMiNT" => match fs::read_dir("/proc/") {
                Ok(dir) => {
                    for entry in dir.flatten() {
                        if let Some(name) = entry.file_name().to_str() {
                            if name.contains("xaaes") || name.contains("xaloader") {
                                return Some(Wm("XaAES".into()));
                            }
                            if name.contains("myaes") {
                                return Some(Wm("MyAES".into()));
                            }
                            if name.contains("naes") {
                                return Some(Wm("N.AES".into()));
                            }
                            if name.contains("geneva") {
                                return Some(Wm("Geneva".into()));
                            }
                        }
                    }
                    Some(Wm("Atari AES".into()))
                }
                Err(_) => Some(Wm("Atari AES".into())),
            },

            _ => None,
        }
    }
}

impl Inject for Wm {
    #[inline(always)]
    fn inject(&self, lua: &mut Lua) {
        if let Err(e) = lua.globals().set("wm", self.0.as_str()) {
            errors::handle(&format!("{}{}", errors::LUA, e));
            panic!();
        }
    }
}
