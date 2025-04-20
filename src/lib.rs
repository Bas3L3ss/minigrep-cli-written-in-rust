use std::{collections::HashSet, env, error::Error, fs};
use colored::Colorize;
use std::borrow::Cow;
use strsim::levenshtein;
use std::io::{stdin, stdout, Write};
use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::IntoAlternateScreen;
use termion::{clear, cursor, terminal_size};
 


pub struct Config {
    pub query: String,
    pub file_path: String,
    pub ignore_case: bool,
    pub no_color: bool,
    pub line_number: bool,
    pub stats: bool,
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
        let allowed_flags: [&str; 4] = ["ignore-case", "no-color", "line-number","stats"];
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
        let stats = flags.contains("STATS") || cli_flags.contains("stats");

        Ok(Config {
            query,
            file_path,
            ignore_case,
            no_color,
            line_number,
            stats
        })
    }   

}

fn conditional_lowercase<'a>(s: &'a str, ignore_case: bool) -> Cow<'a, str> {
    if ignore_case {
        Cow::Owned(s.to_lowercase()) 
    } else {
        Cow::Borrowed(s)  
    }
}




pub fn search(contents: &str, config: &Config) -> (Vec<String>, Vec<usize>,i32,i32) {
    let query = conditional_lowercase(&config.query, config.ignore_case);
    let mut scanned_lines = 0;
    let mut matched_words = 0;
    let mut found_indexes = Vec::new();

    let results: Vec<String> = contents
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            scanned_lines += 1;

            let haystack = conditional_lowercase(line, config.ignore_case);

            if haystack.contains(&*query) {
                found_indexes.push(index);  
                let highlighted_line = line
                    .split_whitespace()
                    .zip(haystack.split_whitespace())
                    .map(|(original, lowered)| {
                        if let Some(pos) = lowered.find(&*query) {
                            matched_words += 1;


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

    (results, found_indexes,scanned_lines,matched_words)
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(&config.file_path)?;
    let (res, found, scanned_lines, matched_words) = search(&contents, &config);

    if config.stats {
        println!("Matching lines: {}, Matching words: {}, Lines Scanned: {}", 
                 res.len(), matched_words, scanned_lines);
    }
    
    if res.is_empty() {
        println!("No results found.");
        return Ok(());
    }
    
    // Use pagination for displaying results
    paginate(&res, &found, &config)?;

    Ok(())
}

 

pub fn paginate(
    results: &[String],
    indexes: &[usize],
    config: &Config,
) -> Result<(), Box<dyn Error>> {
    let mut screen = stdout().into_raw_mode()?.into_alternate_screen()?;
    
    // Get terminal dimensions
    let (width, height) = terminal_size()?;
    let page_height = height.saturating_sub(3) as usize;  
    
    let mut current_offset = 0;
    let total_lines = results.len();
    
    // Initial render
    render_page(&mut screen, results, indexes, config, current_offset, page_height, width, total_lines)?;
    
    // Handle input events
    let stdin = stdin();
    for evt in stdin.events() {
        match evt? {
            // Exit on Escape or Ctrl+C
            Event::Key(Key::Esc) | Event::Key(Key::Ctrl('c')) => break,
            
            // Scroll up
            Event::Key(Key::Up) | Event::Key(Key::Char('k')) => {
                if current_offset > 0 {
                    current_offset -= 1;
                }
            },
            
            // Scroll down
            Event::Key(Key::Down) | Event::Key(Key::Char('j')) | Event::Key(Key::Char('\n')) => {
                if current_offset + page_height < total_lines {
                    current_offset += 1;
                }
            },
            
            // Page up
            Event::Key(Key::PageUp) => {
                current_offset = current_offset.saturating_sub(page_height);
            },
            
            // Page down
            Event::Key(Key::PageDown) | Event::Key(Key::Char(' ')) => {
                current_offset = (current_offset + page_height).min(total_lines.saturating_sub(page_height));
            },
            
            // Home key - go to top
            Event::Key(Key::Home) => {
                current_offset = 0;
            },
            
            // End key - go to bottom
            Event::Key(Key::End) => {
                current_offset = total_lines.saturating_sub(page_height);
            },
     
            
            _ => {} // Ignore other events
        }
        
        // Re-render the page after each event
        render_page(&mut screen, results, indexes, config, current_offset, page_height, width, total_lines)?;
    }
    
    // Restore cursor before exiting
    write!(screen, "{}", cursor::Show)?;
    screen.flush()?;
    
    Ok(())
}

fn render_page<W: Write>(
    screen: &mut W,
    results: &[String],
    indexes: &[usize],
    config: &Config,
    offset: usize,
    page_height: usize,
    width: u16,
    total_lines: usize
) -> Result<(), Box<dyn Error>> {
    // Clear screen and hide cursor
    write!(screen, "{}{}", clear::All, cursor::Hide)?;
    
    // Draw header
    write!(
        screen,
        "{}↑/↓: Scroll | Space: Page Down | Home/End: Jump | ESC/Ctrl+C: Exit",
        cursor::Goto(1, 1)
    )?;
    
    // Draw separator line
    write!(screen, "{}", cursor::Goto(1, 2))?;
    for _ in 0..width {
        write!(screen, "-")?;
    }
    
    // Draw content
    for (display_idx, content_idx) in (offset..offset + page_height).enumerate()
        .take_while(|(_, idx)| *idx < total_lines)
    {
        let line = &results[content_idx];
        let index = indexes[content_idx];
        
        write!(screen, "{}", cursor::Goto(1, display_idx as u16 + 3))?;
        
        if config.line_number {
            let formatted_line = format!("| {:>3} |", index + 1);
            write!(screen, "{} {}", formatted_line.black(), line)?;
        } else {
            write!(screen, "{}", line)?;
        }
    }
    
    // Draw footer with pagination info
    let footer_pos = (page_height + 3) as u16;
    write!(
        screen,
        "{}Page: {}/{} | Showing lines {}-{} of {}",
        cursor::Goto(1, footer_pos),
        offset / page_height + 1,
        (total_lines + page_height - 1) / page_height,
        offset + 1,
        (offset + page_height).min(total_lines),
        total_lines
    )?;
    
    screen.flush()?;
    Ok(())
}




#[cfg(test)]
mod tests {

    use super::*;

    fn create_config(
        query: &str,
        ignore_case: bool,
        no_color: bool,
        line_number: bool,
        stats:bool
    ) -> Config {
        Config {
            query: query.to_string(),
            file_path: "fake_path.txt".to_string(),
            ignore_case,
            no_color,
            line_number,
            stats
        }
    }

    #[test]
    fn one_result() {
        let config = create_config("duct", false, true, false,false);
        let contents = "\
Rust:
safe, fast, productive.
Pick three.";

        let (results, indexes,_,_) = search(contents, &config);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0], "safe, fast, productive.");
        assert_eq!(indexes[0], 1); // line index (0-based)
    }

    #[test]
    fn case_insensitive() {
        let config = create_config("rUsT", true, true, false, false);
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Trust me.";

        let (results, indexes,_,_) = search(contents, &config);

        assert_eq!(results, vec!["Rust:", "Trust me."]);
        assert_eq!(indexes, vec![0, 3]);
    }

    #[test]
    fn no_matches() {
        let config = create_config("missing", false, true, false, false);
        let contents = "\
This text
does not
contain your word.";

        let (results, indexes,_,_) = search(contents, &config);

        assert!(results.is_empty());
        assert!(indexes.is_empty());
    }

    #[test]
    fn highlight_disabled() {
        let config = create_config("fast", false, true, false, false);
        let contents = "safe, fast, productive.";

        let (results, _,_,_) = search(contents, &config);

        assert_eq!(results[0], "safe, fast, productive.");
    }

    #[test]
    fn highlight_enabled() {
        let config = create_config("fast", false, false, false, false);
        let contents = "safe, fast, productive.";

        let (results, _,_,_) = search(contents, &config);

        assert!(results[0].contains("\u{1b}")); // ANSI escape for color
        assert!(results[0].contains("fast")); // Still contains matched text
    }

    #[test]
    fn line_number_enabled() {
        let config = create_config("safe", false, true, true, false);
        let contents = "safe, fast, productive.";

        let (results, indexes,_,_) = search(contents, &config);
        assert_eq!(indexes, vec![0]);
        assert_eq!(results[0], "safe, fast, productive.");
    }
}
