use env_extract::{ConfigStruct, EnvVar};

#[derive(EnvVar, Debug)]
#[case(convert = "lowercase")]
pub enum Environment {
    Development,

    #[default]
    Production,
}

#[derive(ConfigStruct, Debug)]
pub struct Config {
    #[enumerated]
    pub environment: Environment,
}
