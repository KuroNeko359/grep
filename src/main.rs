use std::env;
use std::process;
use minigrep::Config;

fn main() {
    let args: Vec<String> = env::args().collect();

    let config: Config = Config::new(&args).unwrap_or_else(|err|{
        eprintln!("Problem parsing arguments: {}",err);
        process::exit(1);
    });

    //println!("From {} search {}.", config.filename, config.query);

    if let Err(e) = minigrep::run(config) {
        eprintln!("Application Error: {}",e);
        process::exit(1);
    }
}