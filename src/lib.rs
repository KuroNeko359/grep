use std::error::Error;
use std::{env, fs};

// Box<dyn Error>: Retrun a data type witch implement trait of Error
pub fn run(config : Config) -> Result<(),Box<dyn Error>> {
    let contents = fs::read_to_string(config.filename)?;
    let results = if config.case_sensitive {
        search(&config.query, &contents)
    }else {
        search_case_insensitive(&config.query, &contents)
    };
    
    for line in results {
        println!("{}",line);
    }
    // println!("With text:\n{}",contents);
    Ok(())
}

pub struct Config {
    pub query: String,
    pub filename: String,
    pub case_sensitive: bool
}

impl Config {
    pub fn new(mut args: std::env::Args ) -> Result<Config,&'static str> {
        if args.len() < 3 {
            return Err("Not enough arguments")
        }
        args.next();
        let query = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a query string")
        };
        let filename = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a filename string")
        };
        let case_sensitive = env::var("CASE_INSENSITIVE").is_err();
        Ok(Config {
            query: query,
            filename: filename,
            case_sensitive: case_sensitive
        })
    }
}

// Use an iterator to iterate through the content of the file
pub fn search<'a>(query:&str, contents:&'a str) -> Vec<&'a str> {
    contents.lines()
        .filter(|line| line.contains(query))
        .collect()
}

pub fn search_case_insensitive<'a>(query:&str,contents:&'a str) -> Vec<&'a str> {
    let query = query.to_lowercase();
    contents.lines()
        .filter(|line| line.to_lowercase().contains(&query))
        .collect()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_result () {
        let query = "duct";
        let contents = "\
Rust:
safe,fast,productive
Pick three.";
        assert_eq!(vec!["safe,fast,productive"],search(query,contents));
    }

    #[test]
    fn case_insensitive() {
        let query = "rUsT";
        let contents = "\
Rust:
safe,fast,productive
Pick three.
Trust me.";
        assert_eq!(
            vec!["Rust:","Trust me."],
            search_case_insensitive(query, contents)
        )
    }

}

