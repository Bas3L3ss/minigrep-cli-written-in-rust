use std::{collections::HashSet, env, error::Error, fs};
use colored::Colorize;
use std::borrow::Cow;
use strsim::levenshtein;

pub struct Config {
    pub query: String,
    pub file_path: String,
    pub ignore_case: bool,
    pub no_color: bool,
    pub line_number: bool,
}


impl Config {
    pub fn build(args: &[String]) -> Result<Config, String> {
        if args.len() < 3 {
            return Err("Not enough arguments!".to_string());
        }

        let query = args[1].clone();
        let file_path = args[2].clone();

        let mut flags = HashSet::new();
        flags.extend(env::vars().map(|(k, _)| k.to_uppercase()));  

        // more flags here
        let allowed_flags: [&str; 3] = ["ignore-case", "no-color", "line-number"];
        let mut cli_flags = HashSet::new();

        for arg in &args[3..] {
            if let Some(flag) = arg.strip_prefix("--") {
                if allowed_flags.contains(&flag) {
                    cli_flags.insert(flag);
                } else {
                    let suggestion = allowed_flags
                        .iter()
                        .min_by_key(|known| levenshtein(flag, known))
                        .unwrap();

                    return Err(format!(
                        "Unrecognized flag '--{}'. Did you mean '--{}'?",
                        flag, suggestion
                    ));
                }
            } else {
                return Err(format!("Invalid flag format '{}'. Flags must start with '--'", arg));
            }
        }

        // more flags here
        let ignore_case = flags.contains("IGNORE_CASE") || cli_flags.contains("ignore-case");
        let no_color = flags.contains("NO_COLOR") || cli_flags.contains("no-color");
        let line_number = flags.contains("LINE_NUMBER") || cli_flags.contains("line-number");

        Ok(Config {
            query,
            file_path,
            ignore_case,
            no_color,
            line_number,
        })
    }   

}


pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(&config.file_path)?;
    let (res, found) = search(&contents, &config);

    for (line, idx) in res.iter().zip(found.iter()) {
        if config.line_number {
            if !config.no_color {
                println!("{}: {}", format!("line {}", idx + 1).blue().bold(), line);
            } else {
                println!("line {}: {}", idx + 1, line);
            }
        } else {
            println!("{}", line);
        }
    }


    Ok(())
}


// This function does the search and highlights the matches.
pub fn search(contents: &str, config: &Config) -> (Vec<String>, Vec<usize>) {
    let query = conditional_lowercase(&config.query, config.ignore_case);

    let mut found_indexes = Vec::new();

    let results: Vec<String> = contents
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let haystack = conditional_lowercase(line, config.ignore_case);

            if haystack.contains(&*query) {
                found_indexes.push(index); // Track matched line number

                let highlighted_line = line
                    .split_whitespace()
                    .zip(haystack.split_whitespace())
                    .map(|(original, lowered)| {
                        if let Some(pos) = lowered.find(&*query) {
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
        .collect();

    (results, found_indexes)
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

    fn create_config(
        query: &str,
        ignore_case: bool,
        no_color: bool,
        line_number: bool,
    ) -> Config {
        Config {
            query: query.to_string(),
            file_path: "fake_path.txt".to_string(),
            ignore_case,
            no_color,
            line_number,
        }
    }

    #[test]
    fn one_result() {
        let config = create_config("duct", false, true, false);
        let contents = "\
Rust:
safe, fast, productive.
Pick three.";

        let (results, indexes) = search(contents, &config);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0], "safe, fast, productive.");
        assert_eq!(indexes[0], 1); // line index (0-based)
    }

    #[test]
    fn case_insensitive() {
        let config = create_config("rUsT", true, true, false);
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Trust me.";

        let (results, indexes) = search(contents, &config);

        assert_eq!(results, vec!["Rust:", "Trust me."]);
        assert_eq!(indexes, vec![0, 3]);
    }

    #[test]
    fn no_matches() {
        let config = create_config("missing", false, true, false);
        let contents = "\
This text
does not
contain your word.";

        let (results, indexes) = search(contents, &config);

        assert!(results.is_empty());
        assert!(indexes.is_empty());
    }

    #[test]
    fn highlight_disabled() {
        let config = create_config("fast", false, true, false);
        let contents = "safe, fast, productive.";

        let (results, _) = search(contents, &config);

        assert_eq!(results[0], "safe, fast, productive.");
    }

    #[test]
    fn highlight_enabled() {
        let config = create_config("fast", false, false, false);
        let contents = "safe, fast, productive.";

        let (results, _) = search(contents, &config);

        assert!(results[0].contains("\u{1b}")); // ANSI escape for color
        assert!(results[0].contains("fast")); // Still contains matched text
    }

    #[test]
    fn line_number_enabled() {
        let config = create_config("safe", false, true, true);
        let contents = "safe, fast, productive.";

        let (results, indexes) = search(contents, &config);
        assert_eq!(indexes, vec![0]);
        assert_eq!(results[0], "safe, fast, productive.");
    }
}
