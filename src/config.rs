
use std::fs::File;
use std::io::{Error, Read, Write};
use crate::structs::Config;

pub struct ConfigManager {
    config: Config,
    file_path: String,
}

impl ConfigManager {
    pub fn new(file_path: &str) -> Result<Self, Error> {
        let file = File::open(&file_path);

        if file.is_err() {
            let mut file = File::create(&file_path)?;
            let config = Config {
                servers: vec![]
            };

            let json = serde_json::to_string(&config)?;

            file.write_all(json.as_bytes())?;

            return Ok(Self {
                config,
                file_path: file_path.to_owned(),
            })
        }

        let mut contents = String::new();
        file?.read_to_string(&mut contents)?;

        let config = serde_json::from_str(&contents)?;

        Ok(Self {
            config,
            file_path: file_path.to_owned(),
        })
    }

    pub fn get_server_config(&self, id: u64) -> Option<&crate::structs::ServerConfig> {
        self.config.servers.iter().find(|s| s.id == id)
    }

    pub fn update_server_config(&mut self, id: u64, grant_role: u64) -> Result<(), Error> {
        let server_config = self.config.servers.iter_mut().find(|s| s.id == id);

        if server_config.is_none() {
            self.config.servers.push(crate::structs::ServerConfig {
                id,
                grant_role_id: grant_role,
            });

            self.save_config()?;

            return Ok(())
        }

        server_config.unwrap().grant_role_id = grant_role;
        
        self.save_config()?;

        Ok(())
    }

    fn save_config(&self) -> Result<(), Error> {
        let json = serde_json::to_string(&self.config)?;
        let mut file = File::create(&self.file_path)?;

        file.write_all(json.as_bytes())?;

        Ok(())
    }
}
