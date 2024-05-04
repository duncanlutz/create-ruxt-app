use env_extract::{ConfigStruct, EnvVar};

#[derive(EnvVar)]
pub enum Environment {
    #[default]
    Development,
    Production,
}

#[derive(ConfigStruct)]
pub struct Config {
    #[enumerated]
    pub environment: Environment,
}
