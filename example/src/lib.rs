extern crate ldd;

use std::fs::File;
use std::error::Error;
use std::io::Read;
use std::collections::HashMap;

fn read_u32(file: &mut File) -> Result<u32, Box<dyn Error>>
{
    let mut buffer: [u8; 4] = Default::default();
    file.read_exact(&mut buffer)?;

    Ok(u32::from_le_bytes(buffer))
}

fn read_u64(file: &mut File) -> Result<u64, Box<dyn Error>>
{
    let mut buffer: [u8; 8] = Default::default();
    file.read_exact(&mut buffer)?;

    Ok(u64::from_le_bytes(buffer))
}

fn find_node(storage: &mut ldd::Storage, indexed_set: &mut HashMap::<u64,ldd::Ldd>, index: u64) -> ldd::Ldd
{
    if index == 0
    {
        storage.empty_set().clone()
    }
    else if index == 1
    {
        storage.empty_vector().clone()
    }
    else
    {
        indexed_set.get(&index).unwrap().clone()
    }
}

fn read_ldd(storage: &mut ldd::Storage, file: &mut File) -> Result<ldd::Ldd, Box<dyn Error>>
{
    let count = read_u64(file)?;
    println!("node count = {}", count);  

    let mut indexed_set: HashMap::<u64,ldd::Ldd> = HashMap::new();

    for i in 0..count
    {
        // Read a single MDD node. It has the following structure: u64 | u64
        // RmRR RRRR RRRR VVVV | VVVV DcDD DDDD DDDD (little endian)
        // Every character is 4 bits, V = value, D = down, R = right, m = marked, c = copy.
        let a = read_u64(file)?;
        let b = read_u64(file)?;
        //println!("{:064b} | {:064b}", a, b);

        let right = (a & 0x0000ffffffffffff) >> 1;
        let down = b >> 17;

        let mut bytes: [u8; 4] = Default::default();   
        bytes[0..2].copy_from_slice(&a.to_le_bytes()[6..8]); 
        bytes[2..4].copy_from_slice(&b.to_le_bytes()[0..2]); 
        let value = u32::from_le_bytes(bytes);

        //println!("node({}, {}, {})", value, down, right);
        
        let down = find_node(storage, &mut indexed_set, down);
        let right = find_node(storage, &mut indexed_set, right);

        let ldd = storage.insert(value.try_into().unwrap(), &down, &right);
        indexed_set.insert(i+2, ldd);
    }

    let result = read_u64(file)?;
    Ok(indexed_set.get(&result).unwrap().clone())
}

fn load_model(storage: &mut ldd::Storage, filename: &str) -> Result<(), Box<dyn Error>>
{    
    let mut file = File::open(filename)?;

    let vector_length = read_u32(&mut file)?;
    println!("Length of vector {}", vector_length);

    let _unused = read_u32(&mut file)?; // This is called 'k' in Sylvan's ldd2bdd.c, but unused.
    let initial_state = read_ldd(storage, &mut file)?;

    Ok(())
}

pub fn run(config: &Config) -> Result<(), Box<dyn Error>>
{
    // Initialize the library.
    let mut storage = ldd::Storage::new();

    load_model(&mut storage, &config.filename)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_result() {
    }
}