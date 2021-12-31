use std::env;
use std::process;

use example::{run, Config};

fn main()
{
    let args: Vec<String> = env::args().collect();

    let config = Config::new(&args).unwrap_or_else(
        |err| 
        { 
            eprintln!("Problem parsing input: {}", err); 
            process::exit(-1); 
        }
    );

    if let Err(err) = run(&config)
    {
        eprintln!("Problem parsing input: {}", err); 
        process::exit(-1);
    }
}