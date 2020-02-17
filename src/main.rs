use std::env;
mod configuration;
mod monitor;
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("This program receives one and only one argument, the config.json fullpath!");
    }
    monitor::start(&mut configuration::load_config(&args[1]));
    
}