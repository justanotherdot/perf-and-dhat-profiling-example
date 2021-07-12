#![feature(test)]

use csv::Reader;
#[cfg(feature = "dhat-on")]
use dhat;
use std::collections::HashMap;
use std::convert::TryInto;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

#[cfg(feature = "dhat-on")]
use dhat::{Dhat, DhatAlloc};
#[cfg(feature = "dhat-on")]
#[global_allocator]
static ALLOCATOR: DhatAlloc = DhatAlloc;

const NULL: &'static str = "NULL"; // or whatever.

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Field {
    Unknown,
    String,
    Integer,
    Float,
}

pub fn read_file(data: &PathBuf) -> Result<Vec<u8>, Box<dyn Error>> {
    let file = File::open(data)?;
    let meta = file.metadata()?;
    let mut reader = BufReader::new(file);
    let mut contents = Vec::with_capacity(meta.len().try_into()?);
    reader.read_to_end(&mut contents)?;
    Ok(contents)
}

pub fn read_csv(data: &[u8]) -> Result<Vec<Option<Field>>, Box<dyn Error>> {
    let mut reader = Reader::from_reader(data);
    let _headers = reader.headers().unwrap().clone().into_byte_record();
    let mut fields = vec![];
    let mut record = csv::ByteRecord::new();
    while !reader.is_done() {
        reader.read_byte_record(&mut record).unwrap();
        for value in record.iter() {
            fields.push(parse(&value));
        }
    }
    Ok(fields)
}

fn parse(bytes: &[u8]) -> Option<Field> {
    let string = match std::str::from_utf8(bytes) {
        Ok(v) => v,
        Err(_) => return Some(Field::Unknown),
    };
    if string == NULL {
        return None;
    }
    if string.parse::<i64>().is_ok() {
        return Some(Field::Integer);
    };
    if string.parse::<f64>().is_ok() {
        return Some(Field::Float);
    };
    Some(Field::String)
}

fn histogram(fields: &[Option<Field>]) -> HashMap<Option<Field>, i64> {
    fields
        .into_iter()
        .cloned()
        .fold(HashMap::new(), |mut acc, f| {
            *acc.entry(f).or_default() += 1;
            acc
        })
}

fn go(input: &str) -> Result<(), Box<dyn Error>> {
    let bytes = read_file(&input.into())?;
    let fields = read_csv(&bytes)?;
    println!("{:#?}", histogram(fields.as_slice()));
    Ok(())
}

fn main() {
    #[cfg(feature = "dhat-on")]
    let _dhat = Dhat::start_heap_profiling();

    go("test.csv").unwrap_or_else(|e| {
        eprintln!("[csv-count] {}", e);
        std::process::exit(1);
    });
}

#[cfg(test)]
mod tests {
    extern crate test;

    use super::*;
    use test::{black_box, Bencher};

    #[bench]
    fn bench_read_csv(b: &mut Bencher) {
        let bytes = read_file(&"test.csv".into()).expect("failed to read file");
        b.iter(|| {
            for _ in 1..2 {
                black_box(read_csv(&bytes)).expect("benchmark failure");
            }
        });
    }
}
