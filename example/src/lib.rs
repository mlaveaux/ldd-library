extern crate ldd;

mod sylvan_io;

use std::error::Error;
pub fn run(config: &Config) -> Result<(), Box<dyn Error>>
{
    // Initialize the library.
    let mut storage = ldd::Storage::new();

    let (initial_state, transitions) = sylvan_io::load_model(&mut storage, &config.filename)?;
    let mut todo = initial_state.clone();

    while todo != *storage.empty_set()
    {
        let todo1 = storage.empty_set().clone();
        for transition in transitions.iter()
        {
            //let result = ldd::relational_product(&mut storage, &todo, &transition.relation);
            //ldd::union(&mut storage, &todo1, &result);
        }

        todo = todo1;
    }

    Ok(())
}

pub struct Config
{
  pub filename: String,
}

impl Config
{
    pub fn new(args: &[String]) -> Result<Config, &str>
    {
        if args.len() < 2
        {
            Err("Requires at least two arguments")
        }
        else
        {
            Ok(Config { filename: args[1].clone() })
        }
    }
}