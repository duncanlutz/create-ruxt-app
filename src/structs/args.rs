use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about)]
pub struct Args {
    /// The path to create the Ruxt app in
    pub path: PathBuf,
}
