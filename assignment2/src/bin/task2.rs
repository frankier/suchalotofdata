extern crate seahash;
extern crate cardapprox;

use std::io;
use std::io::prelude::*;
use std::env;

use seahash::hash;

use cardapprox::streamcard::StreamCard;
use cardapprox::kmv::KMV;
use cardapprox::hll::HLL;

fn do_stream(s: &mut StreamCard) {
    let stdin = io::stdin();
    let lines = stdin.lock().lines();
    for line in lines {
        let h = hash(line.unwrap().as_bytes());
        s.observe(h);
    }
    println!("{}", s.card());
}

fn main() {
    // Parse args
    let args: Vec<String> = env::args().collect();
    let name = args[1].parse::<String>().unwrap();
    match name.as_str() {
        "hll" => {
            let p = args[2].parse::<u8>().unwrap();
            do_stream(&mut HLL::new(p));
        },
        "kmv" => {
            let k = args[2].parse::<u32>().unwrap();
            do_stream(&mut KMV::new(k));
        },
        _ => {
            panic!("First argument should be hll or kmv")
        }
    }
}
