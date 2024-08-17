//
// Last Modification: 2024-08-17 10:02:44
//

use std::process::exit;
use std::io;
use std::io::Write;
use anyhow;

use std::path::{
    Path,
    PathBuf,
};
use ini::Ini;

#[derive(Debug)]
pub struct DatabaseConf {
    pub host: String,
    pub user: String,
    pub password: String,
    pub name: String,
    pub max_connections: u32,
}

#[derive(Debug)]
pub struct DirectoriesConf {
    pub static_dir: String,
    pub templates_dir: String,
}

#[derive(Debug)]
pub struct StoreConf {
    pub database: DatabaseConf,
    pub directories: DirectoriesConf,
}

pub struct Config {
    ini_file: PathBuf,
}

impl Config {

    // Helper function to prompt for a non-empty string
    fn prompt_valid_str(prompt: &str, default: Option<&str>) -> String {
        loop {
            let mut input = String::new();
            print!("{}", prompt);
            io::stdout().flush().unwrap();
            io::stdin().read_line(&mut input).unwrap();
            let input = input.trim().to_string();
            if input.is_empty() {
                if default.is_some() {
                    println!("The default value is used: {}", default.unwrap());
                    return default.unwrap().to_owned();
                }
            } else {
                return input;
            }
            println!("Input cannot be empty. Please try again.");
        }
    }

    // Helper function to prompt for a valid u32 value
    fn prompt_valid_u32(prompt: &str, default: Option<u32>) -> u32 {
        loop {
            let mut input = String::new();
            print!("{}", prompt);
            io::stdout().flush().unwrap();
            io::stdin().read_line(&mut input).unwrap();
            println!("{} {}", input.is_empty(), default.is_some());
            if input.trim().is_empty() && default.is_some() {
                println!("The default value is used: {}", default.unwrap());
                return default.unwrap();
            }
            match input.trim().parse::<u32>() {
                Ok(value) => return value,
                Err(_) => println!("Please enter a valid u32 value."),
            }
        }
    }

        // Helper function to prompt for a valid directory
        fn prompt_directory(prompt: &str, default: Option<&str>) -> String {
            loop {
                let mut input = String::new();
                print!("{}", prompt);
                io::stdout().flush().unwrap();
                io::stdin().read_line(&mut input).unwrap();
                let input = input.trim().to_string();

                let mut directory = input.clone();
                if input.is_empty() && default.is_some() {
                    directory = default.unwrap().to_owned();
                }

                if !directory.is_empty() {
                    println!("{} {}", directory, default.unwrap().to_owned());
                    if Path::new(&directory).is_dir() {
                        if &directory == default.unwrap() {
                            println!("The default value is used: {}", default.unwrap());
                        }
                        return directory;
                    } else {
                        println!("The {} directory does not exist!", directory);
                    }
                } else {
                    println!("Input cannot be empty. Please try again.");
                }
            }
        }

    fn saved_config(&self) -> Result<StoreConf, anyhow::Error> {
        let config = match Ini::load_from_file(&self.ini_file) {
            Ok(config) => config,
            Err(err) => {
                return Err(anyhow::anyhow!("failed to parse config file: {}", err));
            }
        };
    
        let database_section = match config.section(Some("database")) {
            Some(settings) => settings,
            None => return Err(anyhow::anyhow!("database section not found in config file")),
        };

        let database = DatabaseConf {
            host: match database_section.get("host") {
                Some(value) => value.to_string(),
                None => return Err(anyhow::anyhow!("invalid db host value")),
            },
            user: match database_section.get("user") {
                Some(value) => value.to_string(),
                None => return Err(anyhow::anyhow!("invalid db uservalue")),
            },
            password: match database_section.get("password") {
                Some(value) => value.to_string(),
                None => return Err(anyhow::anyhow!("invalid db password value")),
            },
            name: match database_section.get("name") {
                Some(value) => value.to_string(),
                None => return Err(anyhow::anyhow!("invalid db name value")),
            },
            max_connections: match database_section.get("max_connections") {
                Some(value) => match value.parse() {
                    Ok(value) => value,
                    Err(err) => {
                        return Err(anyhow::anyhow!("invalid max_connections: {}", err));
                    }
                },
                None => 5,
            },
        };

        let directories_section = match config.section(Some("directories")) {
            Some(settings) => settings,
            None => return Err(anyhow::anyhow!("directories section not found in config file")),
        };

        let directories = DirectoriesConf {
            static_dir: match directories_section.get("static") {
                Some(directory) => {
                    if !Path::new(directory).is_dir() {
                        panic!("The {} directory does not exist!", directory);
                    }
                    directory.to_string()
                },
                None => return Err(anyhow::anyhow!("invalid static directory value")),
            },
            templates_dir: match directories_section.get("templates") {
                Some(directory) => {
                    if !Path::new(directory).is_dir() {
                        panic!("The {} directory does not exist!", directory);
                    }
                    directory.to_string()
                },
                None => return Err(anyhow::anyhow!("invalid templates directory value")),
            },
        };

        Ok(StoreConf {
            database,
            directories,
        })
    }

    fn config_from_input(&self) -> Result<StoreConf, anyhow::Error> {
        println!("Configuration file not found. Please enter the following values:");
        println!("Database Configuration:");

        let database = DatabaseConf {
            host: Self::prompt_valid_str("Host (default: localhost): ", Some("localhost")),
            user: Self::prompt_valid_str("User: ", None),
            password: Self::prompt_valid_str("Password: ", None),
            name: Self::prompt_valid_str("Name: ", None),
            max_connections: Self::prompt_valid_u32("Max connections (default 5): ", Some(5)),
        };

        println!("Directories Configuration:");

        let directories = DirectoriesConf {
            static_dir: Self::prompt_directory("Static: ", Some("static")),
            templates_dir: Self::prompt_directory("Templates: ", Some("templates")),
        };

        // println!("{:?} {:?}", database, directories);

        let mut conf = Ini::new();
    
        conf.with_section(None::<String>)
            .set("encoding", "utf-8");
    
        conf.with_section(Some("database"))
            .set("host", &database.host)
            .set("user", &database.user)
            .set("password", &database.password)
            .set("name", &database.name)
            .set("max_connections", &database.max_connections.to_string());

        conf.with_section(Some("directories"))
            .set("static", &directories.static_dir)
            .set("templates", &directories.templates_dir);
    
        conf.write_to_file(&self.ini_file)?;

        println!("The configuration has been successfully saved to {} file.",
            self.ini_file.to_string_lossy());

        Ok(StoreConf {
            database,
            directories,
        })
    }

    pub fn load(&self) -> Result<StoreConf, anyhow::Error> {
        if !&self.ini_file.exists() {
            match self.config_from_input() {
                Ok(conf) => return Ok(conf),
                Err(e) => {
                    println!("Error creating new configuration: {}", e);
                    exit(1);
                }
            };
        }

        let settings= match self.saved_config() {
            Ok(conf) => conf,
            Err(e) => {
                panic!("Error loading store configuration: {}", e);
            }
        };

        Ok(settings)
    }

    pub fn new(ini_file: PathBuf) -> Self {
        // Implementation to create a new instance of Config

        Config {
            ini_file
        }
    }
}