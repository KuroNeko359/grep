use std::env;
use std::process;
use grep::Search;
use grep::Config;

fn main() {
    let config: Config = Config::new(env::args()).unwrap_or_else(|err|{
        eprintln!("Problem parsing arguments: {}",err);
        process::exit(1);
    });
    let search:Search = Search::new(config);
    if let Err(e) = search.run() {
        eprintln!("Application Error: {}",e);
        process::exit(1);
    }
}
