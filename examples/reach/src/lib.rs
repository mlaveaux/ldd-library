extern crate ldd;

mod sylvan_io;

use std::error::Error;

/// Performs state space exploration of the given model and reports the number of states.
pub fn run(config: &Config) -> Result<usize, Box<dyn Error>>
{
    // Initialize the library.
    let mut storage = ldd::Storage::new();

    let (initial_state, transitions) = sylvan_io::load_model(&mut storage, &config.filename)?;

    let mut todo = initial_state.clone();
    let mut states = initial_state.clone(); // The state space.

    while todo != *storage.empty_set()
    {
        let todo1 = storage.empty_set().clone();
        for transition in transitions.iter()
        {
            //let result = ldd::relational_product(&mut storage, &todo, &transition.relation);
            //ldd::union(&mut storage, &todo1, &result);
        }

        todo = ldd::minus(&mut storage, &todo1, &states);
        states = ldd::union(&mut storage, &states, &todo);
    }

    println!("The model has {} states", ldd::len(&storage, &states));

    Ok(ldd::len(&storage, &states))
}

pub struct Config
{
  pub filename: String,
}

impl Config
{
    /// Parses the provided arguments and fills in the configuration.
    pub fn new(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str>
    {
        args.next(); // The first argument is the executable's location.

        let filename = match args.next() {
            Some(arg) => arg,
            None => return Err("Requires model filename")
        };

        Ok(Config { filename })
    }
}