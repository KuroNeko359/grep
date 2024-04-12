use std::error::Error;
use std::{fs, io, result};

// Box<dyn Error>: Retrun a data type witch implement trait of Error

pub struct Config {
    query: String,
    filename_list: Vec<String>,
    case_insensitive: ConfigOption,
    case_invert: ConfigOption,
    case_line_number: ConfigOption,
    case_recursive: ConfigOption,
    case_file_name: ConfigOption,
    case_count_line_number: ConfigOption,
}

/*
    -i --ignore-case CaseInsensitive ：Insensitive mode.
    -v --invert-match CaseInvert ：Reverse lookup, print only non-matching rows.
    -n --line-number CaseLineNumber ：Show matched row number.
    -r --recursive CaseRecursive ：Recursive search the file in directory.
    -l --files-with-matches CaseFileName：Print only the matched file name.
    -c --count CaseCountLineNumber：Print only the matched line number.
*/
enum ConfigOption {
    CaseInsensitive(bool),
    CaseInvert(bool),
    CaseLineNumber(bool),
    CaseRecursive(bool),
    CaseFileName(bool),
    CaseCountLineNumber(bool),
}

impl Config {
    pub fn new(mut args: std::env::Args) -> Result<Config, &'static str> {
        // itself comes with a parameter.
        // So if the function takes two arguments.
        // args.len == 3
        if args.len() < 3 {
            print_usage();
            while let Some(args) = args.next() {
                if args.starts_with("-") {
                    match args.as_str() {
                        "--help" => print_help(),
                        _ => return Err("Not enough arguments."),
                    }
                }
            }
        }
        args.next();
        // Init variables
        let mut query = String::new();
        let mut filename_list: Vec<String> = Vec::new();
        let mut case_insensitive = false;
        let mut case_invert = false;
        let mut case_line_number = false;
        let mut case_recursive = false;
        let mut case_file_name = false;
        let mut case_count_line_number = false;

        // Match arguments
        while let Some(args) = args.next() {
            if args.starts_with("-") {
                match args.as_str() {
                    "-i" => case_insensitive = true,
                    "--ignore-case" => case_insensitive = true,
                    "-v" => case_invert = true,
                    "--invert-match" => case_invert = true,
                    "-n" => case_line_number = true,
                    "--line-number" => case_line_number = true,
                    "-r" => case_recursive = true,
                    "--recursive" => case_recursive = true,
                    "-l" => case_file_name = true,
                    "--files-with-matches" => case_file_name = true,
                    "-c" => case_count_line_number = true,
                    "--count" => case_count_line_number = true,
                    _ => eprintln!("unknown option: {}", args),
                }
            } else if query.is_empty() {
                query = args;
            } else {
                filename_list.push(args);
            }

        }

        Ok(Config {
            query: query,
            filename_list: filename_list,
            case_insensitive: ConfigOption::CaseInsensitive(case_insensitive),
            case_invert: ConfigOption::CaseInvert(case_invert),
            case_line_number: ConfigOption::CaseLineNumber(case_line_number),
            case_recursive: ConfigOption::CaseRecursive(case_recursive),
            case_file_name: ConfigOption::CaseFileName(case_file_name),
            case_count_line_number: ConfigOption::CaseCountLineNumber(case_count_line_number),
        })
    }
}

pub struct Search {
    config: Config,
}


#[derive(Debug,PartialEq)]
pub struct MatchedContent {
    pub filename: String,
    pub content : Vec<String>,
}

impl MatchedContent {
    fn new(filename: String,content : Vec<String>) -> MatchedContent{
        MatchedContent {filename,content}
    }
}

impl Search {
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        match self.search() {
            Ok(lines_list) => {
                for lines in &lines_list {
                    // Match ConfigOption::CaseFileName
                    if lines_list.len() == 1 {
                        match &self.config.case_file_name {
                            ConfigOption::CaseFileName(true) => println!("\x1b[1;32m{}\x1b[0m",lines.filename),
                            _ => {
                                if lines.content.len() > 0 {
                                    println!("{}",lines.content[0])
                                }
                            }
                        }
                        
                    }else {
                        for line in &lines.content {
                            match &self.config.case_file_name {
                                ConfigOption::CaseFileName(true) => println!("\x1b[1;32m{}\x1b[0m",lines.filename),
                                _ => println!("\x1b[1;32m{}\x1b[0m:{}",lines.filename,line)
                            }
                        } 
                    }
                    
                }
            }
            Err(err) => {println!("{}",err)}
        }
        // println!("With text:\n{}",contents);
        Ok(())
    }

    pub fn new(config: Config) -> Search {
        Search { config }
    }

    pub fn search(&self) -> Result<Vec<MatchedContent>, io::Error> {
        let mut contents_list = Vec::new();
        // Match  CaseRecursive 
        // read contents from config
        match &self.config.case_recursive {
            ConfigOption::CaseRecursive(true) => {
                for filename in &self.config.filename_list {
                    if let Ok(path_list) =  explore_directory(&filename){
                        for path in path_list{
                            contents_list.push(read_contents(&self.config, &path)?);
                        }
                    };
                }
            },
            ConfigOption::CaseRecursive(false) => {
                for filename in &self.config.filename_list {
                    contents_list.push(read_contents(&self.config, filename)?);
                }
            },
            _ => {}
        }

        

        // Search
        Ok(contents_list)
    }


    
}
fn explore_directory(path: &str) -> Result<Vec<String>, io::Error> {
    let mut path_list: Vec<String> = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        
        // If path is file , push the path to the list
        if entry_path.is_file() {
            if let Some(file_name_str) = entry_path.to_str() {
                    path_list.push(file_name_str.to_string());

            }
        } else if entry_path.is_dir() {
            let dir_path = entry_path.to_str().ok_or(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid directory name",
            ))?;
            // Recursion calling explore_directory
            let sub_dir_files = explore_directory(dir_path)?;
            path_list.extend(sub_dir_files);
        }
    }
    Ok(path_list)
}

// Use an iterator to iterate through the content of the file
fn read_contents(config: &Config, filename: &String) -> Result<MatchedContent, io::Error> {
    let mut contents_string: String = String::new();
    match fs::read_to_string(filename) {
        Ok(result) => {
            contents_string = result
        },
        Err(err) => eprintln!("Error: {}",err)
    }; 
    let contents: Vec<String> = contents_string
        // Create a iterator
        .lines()
        // Get line number
        .enumerate()
        // Warp the line_number in a tuple for future use.
        .map(|(line_number, line)| (line_number, line.to_string()))
        .filter(|(_, line)| {
            // CaseInsensitive And CaseInvert
            search_case_insensitive_and_invert(config, line)
        })
        // Format input information
        .map(|(line_number, line)| {
                match config.case_line_number {
                    ConfigOption::CaseLineNumber(true) => {
                        format!("lines:{},{}", line_number, line)
                    },
                    _ => line.to_string(),
                }
            }
        )
        .enumerate()
        .map(|(line_number,line)|{
            match config.case_line_number {
                ConfigOption::CaseLineNumber(true) => {
                    line_number.to_string()
                },
                _ => line.to_string(),
            }

        })
        .collect();
    // CaseCountLineNumber
    match config.case_count_line_number {
        ConfigOption::CaseCountLineNumber(true) => Ok(MatchedContent::new(filename.to_string(),vec![contents.len().to_string()])),
        _ => Ok(MatchedContent::new(filename.to_string(), contents)),
    }
}

fn search_case_insensitive_and_invert(config: &Config, line: &String) -> bool {
    let query_lowercase = config.query.to_lowercase();
    let lower_line = line.to_lowercase();
    let query_find = lower_line.contains(&query_lowercase);
    
    match config.case_insensitive {
        // If CaseInsensitive is true,
        // continue with CaseInvert comparison;
        // otherwise, proceed with case-sensitive comparison
        ConfigOption::CaseInsensitive(true) => {
            // To evaluate CaseInvert
            match config.case_invert {
                // If CaseInvert is true return !query_find else return query find
                ConfigOption::CaseInvert(true) => !query_find,
                _ => query_find,
            }
        }
        _ => line.contains(&config.query),
    }

}

fn print_usage() {
    println!("usage: grep [options] pattern [files]");
    println!("Try `grep --help` for more information.")
}

fn print_help() {
    println!("Usage: grep [OPTION]... PATTERN [FILE]...");
    println!("    Search for PATTERN in each FILE or standard input.");
    println!("    PATTERN is, by default, a basic regular expression (BRE).");
    println!("    Example: grep -i 'hello world' menu.h main.c\n");
    
    println!("Common selection and interpretation:");
    println!("  -i, --ignore-case         ignore case distinctions");
    println!("  -v, --invert-match        select non-matching lines");
    println!("  -n, --line-number         print line number with output lines");
    println!("  -r, --recursive           like --directories=recurse");
    println!("  -l, --files-with-matches  print only names of FILEs containing matches");
    println!("  -c, --count               print only a count of matching lines per FILE");
    println!("      --help                display this help text and exit\n");
    
    println!("Report bugs to: jz2077056966@gmail.com");
    println!("GitHub home page: <https://github.com/KuroNeko359/minigrep>");
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn one_result() {
        let config: Config = Config {
            query: String::from("thank"),
            filename_list: vec![
                String::from("C:\\Users\\kuroneko\\Documents\\code\\RustProject\\minigrep\\README.md"),
                String::from("C:\\Users\\kuroneko\\Documents\\code\\RustProject\\minigrep\\README2.md")
            ],
            case_insensitive: ConfigOption::CaseInsensitive(true),
            case_invert: ConfigOption::CaseInvert(false),
            case_line_number: ConfigOption::CaseLineNumber(false),
            case_recursive: ConfigOption::CaseRecursive(false),
            case_file_name: ConfigOption::CaseFileName(false),
            case_count_line_number: ConfigOption::CaseCountLineNumber(false),
        };
        let search: Search = Search::new(config);
        match search.search() {
            Ok(result) => {
                println!("{:?}", result.get(0));
                assert_eq!(
                    vec![
                        MatchedContent::new(String::from("C:\\Users\\kuroneko\\Documents\\code\\RustProject\\minigrep\\README.md"), vec![String::from("Thank for use")]),
                        MatchedContent::new(String::from("C:\\Users\\kuroneko\\Documents\\code\\RustProject\\minigrep\\README2.md"), vec![String::from("Thank for use")]),
                    ],
                    result
                )
            }
            Err(err) => {
                eprintln!("Error search: {}", err);
            }
        }
    }
}
