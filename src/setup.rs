//
// Last Modification: 2024-07-26 18:40:13
//

use std::process::exit;
use anyhow;
use std::path::PathBuf;
use ini::Ini;

pub struct DatabaseConf {
    pub host: String,
    pub user: String,
    pub password: String,
    pub name: String,
    pub max_connections: u32,
}

pub struct Config {
    ini_file: PathBuf,
}

impl Config {

    fn saved_config(&self) -> Result<DatabaseConf, anyhow::Error> {

        let config = match Ini::load_from_file(&self.ini_file) {
            Ok(config) => config,
            Err(err) => {
                return Err(anyhow::anyhow!("failed to parse config file: {}", err));
            }
        };
    
        let db_settings = match config.section(Some("database")) {
            Some(settings) => settings,
            None => return Err(anyhow::anyhow!("database section not found in config file")),
        };
    
        let db_conf = DatabaseConf {
            host: match db_settings.get("host") {
                Some(value) => value.to_string(),
                None => return Err(anyhow::anyhow!("invalid db host value")),
            },
            user: match db_settings.get("user") {
                Some(value) => value.to_string(),
                None => return Err(anyhow::anyhow!("invalid db uservalue")),
            },
            password: match db_settings.get("password") {
                Some(value) => value.to_string(),
                None => return Err(anyhow::anyhow!("invalid db password value")),
            },
            name: match db_settings.get("name") {
                Some(value) => value.to_string(),
                None => return Err(anyhow::anyhow!("invalid db name value")),
            },
            max_connections: match db_settings.get("max_connections") {
                Some(value) => match value.parse() {
                    Ok(value) => value,
                    Err(err) => {
                        return Err(anyhow::anyhow!("invalid max_connections: {}", err));
                    }
                },
                None => 5,
            },
        };
    
        Ok(db_conf)
    }

    fn default_config_file(&self) -> Result<(), anyhow::Error> {

        let mut conf = Ini::new();
    
        conf.with_section(None::<String>)
            .set("encoding", "utf-8");
    
        conf.with_section(Some("database"))
            .set("host", "localhost")
            .set("user", "admin")
            .set("password", "unknown")
            .set("name", "unknown")
            .set("max_connections", "5");
    
        conf.write_to_file(&self.ini_file)?;

        Ok(())
    }

    pub fn load(&self) -> Result<DatabaseConf, anyhow::Error> {
        if !&self.ini_file.exists() {
            match self.default_config_file() {
                Ok(_) => {
                    println!("Take some time to check the configuration file: {}", self.ini_file.display());
                    exit(0);
                },
                Err(e) => {
                    panic!("Error creating default configuration file: {}", e);
                }
            };
        }

        let db_conf= match self.saved_config() {
            Ok(conf) => conf,
            Err(e) => {
                panic!("Error loading database configuration: {}", e);
            }
        };

        Ok(db_conf)
    }

    pub fn new(app_name: &str, config_dir: &PathBuf) -> Self {
        // Implementation to create a new instance of Config

        if !config_dir.exists() {
            std::fs::create_dir(&config_dir)
                .expect("Failed to create config directory");
        }

        let ini_file = config_dir.join(format!("{}.ini", app_name));

        // println!("Config Directory: {:?}", ini_file.display());

        Config {
            ini_file
        }
    }
}