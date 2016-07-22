extern crate clap;
extern crate toml;
extern crate regex;

use base::math::*;
use self::clap::{App, Arg, ArgMatches};
use self::regex::Regex;
use std::error::Error as StdError;
use std::path::Path;
use std::fs::File;
use std::io::{Read, Write};
use self::toml::Value;
use std::string::String;

pub struct Config {
    pub resolution: Dimension2u,
    pub window_mode: WindowMode,
    pub window_title: String,
    pub vsync: bool,
    pub seed: u64, /* view range
                    * anti aliasing
                    * Controls
                    * frames
                    * seed
                    * Chunkweite */
}
impl Config {
    /// creates a new Config in three steps:
    /// 1. loads default config
    /// 2. Overrides from toml config file
    /// 3. Overrides from command line
    pub fn load_config() -> Result<Config, Box<StdError>> {
        let matches = App::new("Plantex")
            .version(env!("CARGO_PKG_VERSION"))
            .about("Game about Plants!")
            .arg(Arg::with_name("Resolution")
                .help("(e.g. =1280x720) 'Sets Resolution to new value'")
                .takes_value(true)
                .long("resolution"))
            .arg(Arg::with_name("WindowMode")
                .help("[Windowed, FullScreen] 'Sets WindowMode'")
                .takes_value(true)
                .long("windowmode"))
            .arg(Arg::with_name("Vsync")
                .help("[on/off]")
                .takes_value(true)
                .long("vsync"))
            .arg(Arg::with_name("Seed")
                .help("'Takes a specified seed to generate map'")
                .takes_value(true)
                .long("seed"))
            .arg(Arg::with_name("File")
                .help("Takes config file")
                .takes_value(true)
                .long("config-file"))
            .arg(Arg::with_name("Write_Config_File")
                .help("'Writes Config File with default values'")
                .long("write-config"))
            .get_matches();

        let conf = Config::default();

        if matches.is_present("Write_Config_File") {
            if Path::new("config.toml").exists() {
                return Err("Config.toml already exists".into());
            }
            let toml = r#"
[Graphic]
resolution_width = 800
resolution_height = 600
windowmode = "Windowed"
vsync = true

[Game_settings]
seed = 42
            "#;

            let mut f = match File::create("config.toml") {
                Ok(n) => n,
                _ => return Err("Failed writing config file (e.g. missing write permission".into()),
            };
            match f.write_all(&toml.to_string().into_bytes()) {
                Ok(n) => n,
                _ => return Err("Failed writing config file (e.g. missing write permission".into()),
            };
        };

        let t_conf = try!(config_toml(conf, &matches));
        let conf_final = try!(config_command(t_conf, &matches));
        Ok(conf_final)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            resolution: Dimension2::new(800, 600),
            window_mode: WindowMode::Windowed,
            window_title: format!("Plantex {}", env!("CARGO_PKG_VERSION")),
            vsync: false,
            seed: 42,
        }
    }
}

fn config_toml(mut default_config: Config, matches: &ArgMatches) -> Result<Config, Box<StdError>> {
    let mut name = "config.toml";
    if let Some(file) = matches.value_of("File") {
        let file_reg = Regex::new(".*\\.toml").unwrap();

        if file_reg.is_match(file) && Path::new(file).exists() {
            name = file;
        } else {
            return Err("invalid File in command line".into());
        }
    }
    if Path::new(name).exists() {
        let mut f = try!(File::open(name));
        let mut s = String::new();

        try!(f.read_to_string(&mut s));

        let value: Value = match s.parse() {
            Ok(n) => n,
            _ => return Err("corrupted config file".into()),
        };

        let mut res_toml = match value.lookup("Graphic.resolution_width") {
            Some(n) => n,
            None => return Err("resolution_width in config file is invalid".into()),
        };
        let int_res_width = match res_toml.as_integer() {
            Some(n) => {
                if n < 1 {
                    return Err("resolution can not be negative".into());
                } else {
                    n as u32
                }
            }

            None => return Err("resolution_width in config file has no integer".into()),
        };

        res_toml = match value.lookup("Graphic.resolution_height") {
            Some(n) => n,
            None => return Err("resolution_height in config file is invalid".into()),
        };
        let int_res_height = match res_toml.as_integer() {
            Some(n) => {
                if n < 1 {
                    return Err("resolution can not be negative".into());
                } else {
                    n as u32
                }
            }

            None => return Err("resolution_height in config file has no integer".into()),
        };

        default_config.resolution = Dimension2::new(int_res_width, int_res_height);

        let window = match value.lookup("Graphic.windowmode") {
            Some(n) => n,
            None => return Err("resolution_height in config file is invalid".into()),
        };

        match window.as_str() {
            Some(n) => {
                match n {
                    "Windowed" => default_config.window_mode = WindowMode::Windowed,
                    "FullScreen" => default_config.window_mode = WindowMode::FullScreen,
                    _ => return Err("invalid Window Mode in config file".into()),
                }
            }
            _ => return Err("invalid Window Mode in config file".into()),
        }

        let sync = match value.lookup("Graphic.vsync") {
            Some(n) => n,
            None => return Err("resolution_height in config file is invalid".into()),
        };

        match sync.as_bool() {
            Some(n) => default_config.vsync = n,
            None => return Err("vsync value in config file is invalid".into()),
        };

        let seed = match value.lookup("Game_settings.seed") {
            Some(n) => n,
            None => return Err("seed in config file is invalid".into()),
        };

        match seed.as_integer() {
            Some(n) => {
                if n < 0 {
                    return Err("seed can not be negative".into());
                } else {
                    default_config.seed = n as u64
                }
            }
            None => return Err("seed in config file is invalid".into()),
        };

    }


    Ok(default_config)
}


fn config_command(mut toml_config: Config, matches: &ArgMatches) -> Result<Config, Box<StdError>> {


    if let Some(res) = matches.value_of("Resolution") {
        let reg_res = Regex::new(r"^([1-9]\d{1,4})x([1-9]\d{1,4})").unwrap();


        if reg_res.is_match(res) {
            for cap in reg_res.captures_iter(res) {
                let res_x = cap.at(1).unwrap().parse::<u32>().unwrap();
                let res_y = cap.at(2).unwrap().parse::<u32>().unwrap();
                toml_config.resolution = Dimension2::new(res_x, res_y);
            }
        } else {
            return Err("invalid resolution in command line argument".into());
        }
    }


    if let Some(mode) = matches.value_of("WindowMode") {
        match mode {
            "Windowed" => toml_config.window_mode = WindowMode::Windowed,
            "FullScreen" => toml_config.window_mode = WindowMode::FullScreen,
            _ => return Err("invalid Window Mode in command line argument".into()),
        }
    }

    if let Some(sync) = matches.value_of("Vsync") {
        match sync {
            "on" => toml_config.vsync = true,
            "off" => toml_config.vsync = false,
            _ => return Err("Vsync can only be set on or off on command line".into()),
        }
    }

    if let Some(seed) = matches.value_of("Seed") {
        match seed.parse::<u64>() {
            Ok(n) => toml_config.seed = n,
            _ => return Err("Seed from command line is invalid".into()),
        }
    }
    Ok(toml_config)
}
pub enum WindowMode {
    Windowed,
    // FullScreenWindow, // TODO: maybe add this
    FullScreen,
}
