use crate::assets;
use crate::assets::defaults;
use crate::errors;
use crate::mlua;
use crate::regex;
use crate::sysinfo;
use mlua::prelude::*;
use regex::Regex;
use std::fs;
use std::path::Path;
// use std::process::Command;
// use std::sync::OnceLock;
use std::thread;
use sysinfo::SystemExt;

pub(crate) mod context;
pub(crate) mod cpu;
pub(crate) mod de;
pub(crate) mod distro;
pub(crate) mod gpu;
pub(crate) mod host;
pub(crate) mod kernel;
pub(crate) mod memory;
pub(crate) mod motherboard;
pub(crate) mod package_managers;
pub(crate) mod resolution;
pub(crate) mod shell;
pub(crate) mod uptime;
pub(crate) mod utils;
pub(crate) mod wm;

use crate::Inject;
use assets::{ANSI, PRINT};
use context::Context;
use cpu::Cpu;
use de::De;
use defaults::INFO;
use distro::Distro;
use gpu::Gpus;
use host::Host;
use kernel::Kernel;
use memory::Memory;
use motherboard::Motherboard;
use package_managers::PackageManagers;
use resolution::Resolution;
use shell::Shell;
use uptime::Uptime;
use utils::get_system;
use wm::Wm;

// static LSPCI_CACHE: OnceLock<String> = OnceLock::new();
// static GPU_REGEX: OnceLock<Regex> = OnceLock::new();
//
// fn cached_lspci_output() -> String {
//     LSPCI_CACHE
//         .get_or_init(|| {
//             Command::new("sh")
//                 .arg("-c")
//                 .arg("lspci -mm")
//                 .output()
//                 .ok()
//                 .and_then(|o| String::from_utf8(o.stdout).ok())
//                 .unwrap_or_default()
//         })
//         .clone()
// }

fn fetch_parallel_info() -> (Option<Cpu>, Memory, Option<Gpus>) {
    let cpu_thread = thread::spawn(Cpu::new);

    let memory_thread = thread::spawn(Memory::new);

    let gpu_thread = thread::spawn(Gpus::new);

    let cpu = cpu_thread.join().unwrap();
    let memory = memory_thread.join().unwrap();
    let gpu = gpu_thread.join().unwrap();

    (cpu, memory, gpu)
}

pub(crate) struct Info {
    ctx: Lua,
    rendered: String,
    width: i32,
    height: i32,
    pub context: Option<Context>,
    pub distro: Distro,
    pub kernel: Kernel,
    pub uptime: Uptime,
    pub package_managers: PackageManagers,
    pub shell: Shell,
    pub resolution: Option<Resolution>,
    pub de: Option<De>,
    pub wm: Option<Wm>,
    pub cpu: Option<Cpu>,
    pub gpu: Option<Gpus>,
    pub memory: Memory,
    pub motherboard: Option<Motherboard>,
    pub host: Option<Host>,
}

impl Info {
    pub fn new() -> Self {
        get_system().refresh_all();

        let kernel = Kernel::new();
        let context = Context::new();

        let (cpu, memory, gpu) = fetch_parallel_info();

        let distro = Distro::new(&kernel);
        let de = De::new(&kernel, &distro);
        let resolution = Resolution::new(&kernel);
        let wm = Wm::new(&kernel);
        let shell = Shell::new(&kernel);
        let uptime = Uptime::new(&kernel);
        let package_managers = PackageManagers::new(&kernel);
        let motherboard = Motherboard::new(&kernel);
        let host = Host::new(&kernel);

        Info {
            ctx: Lua::new(),
            rendered: String::new(),
            width: 0,
            height: 0,
            context,
            distro,
            kernel,
            uptime,
            package_managers,
            shell,
            resolution,
            de,
            wm,
            cpu,
            gpu,
            memory,
            motherboard: Some(motherboard.unwrap()),
            host: Some(host.unwrap()),
        }
    }

    pub fn render(&mut self) {
        if let Err(e) = self.ctx.load(PRINT).exec() {
            errors::handle(&format!("{}{}", errors::LUA, e));
            panic!();
        }

        if let Err(e) = self.ctx.load(ANSI).exec() {
            errors::handle(&format!("{}{}", errors::LUA, e));
            panic!();
        }

        let info = Path::new("/home/")
            .join(
                self.context
                    .clone()
                    .unwrap_or(Context {
                        user: String::new(),
                        host: String::new(),
                    })
                    .user,
            )
            .join(".config/freshfetch/info.lua");

        if info.exists() {
            match fs::read_to_string(&info) {
                Ok(file) => {
                    match self.ctx.load(&file).exec() {
                        Ok(_) => (),
                        Err(e) => {
                            errors::handle(&format!("{}{}", errors::LUA, e));
                            panic!();
                        }
                    }
                    match self.ctx.globals().get::<&str, String>("__freshfetch__") {
                        Ok(v) => self.rendered = v,
                        Err(e) => {
                            errors::handle(&format!("{}{}", errors::LUA, e));
                            panic!();
                        }
                    }
                }
                Err(e) => {
                    errors::handle(&format!(
                        "{}{file:?}{}{err}",
                        errors::io::READ.0,
                        errors::io::READ.1,
                        file = info,
                        err = e
                    ));
                    panic!();
                }
            }
        } else {
            match self.ctx.load(INFO).exec() {
                Ok(_) => (),
                Err(e) => {
                    errors::handle(&format!("{}{}", errors::LUA, e));
                    panic!();
                }
            }
            match self.ctx.globals().get::<&str, String>("__freshfetch__") {
                Ok(v) => self.rendered = v,
                Err(e) => {
                    errors::handle(&format!("{}{}", errors::LUA, e));
                    panic!();
                }
            }
        }
    }
}

impl Inject for Info {
    fn prep(&mut self) {
        if let Some(v) = &self.context {
            v.inject(&mut self.ctx);
        }

        self.kernel.inject(&mut self.ctx);
        self.distro.inject(&mut self.ctx);
        self.uptime.inject(&mut self.ctx);
        self.package_managers.inject(&mut self.ctx);
        self.shell.inject(&mut self.ctx);

        if let Some(v) = &self.resolution {
            v.inject(&mut self.ctx);
        }
        if let Some(v) = &self.wm {
            v.inject(&mut self.ctx);
        }
        if let Some(v) = &self.de {
            v.inject(&mut self.ctx);
        }
        if let Some(v) = &self.cpu {
            v.inject(&mut self.ctx);
        }
        if let Some(v) = &self.gpu {
            v.inject(&mut self.ctx);
        }
        self.memory.inject(&mut self.ctx);

        if let Some(v) = &self.motherboard {
            v.inject(&mut self.ctx);
        }
        if let Some(v) = &self.host {
            v.inject(&mut self.ctx);
        }

        self.render();

        // Strip ANSI codes and compute width & height
        let plaintext = {
            let regex = Regex::new(r#"(?i)\x1b\[[\d;]*[a-zA-Z]"#).unwrap();
            regex.replace_all(&self.rendered, "").to_string()
        };

        let mut w = 0usize;
        let mut h = 0usize;

        for line in plaintext.lines() {
            let len = line.chars().count();
            if len > w {
                w = len;
            }
            h += 1;
        }

        self.width = w as i32;
        self.height = h as i32;
    }

    fn inject(&self, lua: &mut Lua) {
        let globals = lua.globals();

        if let Err(e) = globals.set("info", self.rendered.as_str()) {
            errors::handle(&format!("{}{}", errors::LUA, e));
        }
        if let Err(e) = globals.set("infoWidth", self.width) {
            errors::handle(&format!("{}{}", errors::LUA, e));
        }
        if let Err(e) = globals.set("infoHeight", self.height) {
            errors::handle(&format!("{}{}", errors::LUA, e));
        }
    }
}
