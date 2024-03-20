#[derive(Debug, Clone)]
#[cfg_attr(test, derive(Default))]
pub struct Config {
    pub app_socket: String,
    pub buses_url: String,
    pub schedule_url: String,
    pub stops_url: String,
    pub update_interval: chrono::TimeDelta,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
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
            update_interval: chrono::TimeDelta::try_minutes(config.get_int("update_interval_min")?)
                .ok_or_else(|| anyhow::anyhow!("Invalid update interval"))?,
        })
    }
}
