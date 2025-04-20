use std::error::Error;
use std::{env, fs};
use colored::Colorize;
use std::borrow::Cow;

pub struct Config {
    pub query: String,
    pub file_path: String,
    pub ignore_case: bool,
    pub no_color: bool,
}

impl Config {
    pub fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("not enough arguments!");
        }
        let query = args[1].clone();
        let file_path = args[2].clone();

        let mut ignore_case = env::var("IGNORE_CASE").is_ok();
        let mut no_color = env::var("NO_COLOR").is_ok();

            for arg in &args[3..] { // Skip the first 3 items (query, file_path, and the executable name)
                match arg.as_str() {
                    "--ignore-case" => ignore_case = true,
                    "--no-color" => no_color = true,
                    _ => (),
                }
            }


        Ok(Config {
            query,
            file_path,
            ignore_case,
            no_color
        })
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(&config.file_path)?;  
    let res = search(&contents, &config);

    for line in res {
        println!("{line}");
    }

    Ok(())
}

// This function does the search and highlights the matches.
pub fn search(contents: &str, config: &Config) -> Vec<String> {
    let query = conditional_lowercase(&config.query, config.ignore_case);
    
    contents
        .lines()
        .filter_map(|line| {
            let haystack = conditional_lowercase(line, config.ignore_case);

            if haystack.contains(&*query) {
                let highlighted_line = line
                    .split_whitespace()
                    .zip(haystack.split_whitespace())
                    .map(|(original, lowered)| {
                        if let Some(pos) = lowered.find(&*query)   {
                            let before = &original[..pos];
                            let matched = &original[pos..pos + query.len()];
                            let after = &original[pos + query.len()..];
                            if !config.no_color {
                                format!("{}{}{}", before, matched.red().bold(), after)
                            } else {
                                format!("{}{}{}", before, matched, after)
                            }
                        } else {
                            original.to_string()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ");

                Some(highlighted_line)
            } else {
                None
            }
        })
        .collect()
}

fn conditional_lowercase<'a>(s: &'a str, ignore_case: bool) -> Cow<'a, str> {
    if ignore_case {
        Cow::Owned(s.to_lowercase()) 
    } else {
        Cow::Borrowed(s)  
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_result() {
        let config = Config {
            query: "duct".to_string(),
            file_path: "path".to_string(),
            ignore_case: false,
            no_color: true

        };
        let contents = "\
Rust:
safe, fast, productive.
Pick three.";

        assert_eq!(vec!["safe, fast, productive."], search(contents, &config));
    }

    #[test]
    fn case_insensitive() {
        let config = Config {
            query: "rUsT".to_string(),
            file_path: "path".to_string(),
            ignore_case: true,
            no_color: true
        };
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Trust me.";

        assert_eq!(
            vec!["Rust:", "Trust me."],
            search(contents, &config)
        );
    }
}
