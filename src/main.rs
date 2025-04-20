use std::process;
use std::env;

use minigrep::run;
use minigrep::Config;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    let config = Config::build(&args).unwrap_or_else(|err| {
        eprintln!("Problem parsing argument: {err}");
        process::exit(1)
    });


    
    
    if let Err(e) = run(config){
        eprintln!("Application error: {e}");
        process::exit(1)

    }

}
