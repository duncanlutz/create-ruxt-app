use clap::Parser;

pub mod structs;
use structs::{Args, Config, Handler};

fn main() {
    let env = Config::get();
    let args = Args::parse();

    Handler::create_ruxt_app(args.path, env.environment);
}
