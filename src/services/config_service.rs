#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct ConfigService {
    app_socket: String,
    buses_url: String,
    schedule_url: String,
    stops_url: String,
}

impl ConfigService {
    pub fn new() -> anyhow::Result<Self> {
        let config = config::Config::builder()
            .add_source(config::File::new("config.toml", config::FileFormat::Toml))
            .build()?;

        let resource = config.get_string("resource")?;
        let api_key = config.get_string("api_key")?;

        Ok(Self {
            app_socket: config.get_string("app_socket")?,
            buses_url: format!(
                "https://sheets.googleapis.com/v4/spreadsheets/{resource}/values/{}/?key={api_key}",
                config.get_string("buses")?
            ),
            schedule_url: format!(
                "https://sheets.googleapis.com/v4/spreadsheets/{resource}/values/{}/?key={api_key}",
                config.get_string("schedule")?
            ),
            stops_url: format!(
                "https://sheets.googleapis.com/v4/spreadsheets/{resource}/values/{}/?key={api_key}",
                config.get_string("stops")?
            ),
        })
    }

    pub fn app_socket(&self) -> &str {
        &self.app_socket
    }

    pub fn buses_url(&self) -> &str {
        &self.buses_url
    }

    pub fn schedule_url(&self) -> &str {
        &self.schedule_url
    }

    pub fn stops_url(&self) -> &str {
        &self.stops_url
    }
}
