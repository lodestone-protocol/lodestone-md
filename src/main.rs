//! CLI：`mddag <file.md | ->`，向 stdout 输出 §8 契约 JSON。

use std::env;
use std::fs;
use std::io::{self, Read};
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: mddag <file.md | ->");
        process::exit(2);
    }
    let input = if args[1] == "-" {
        let mut buf = String::new();
        match io::stdin().read_to_string(&mut buf) {
            Ok(_) => buf,
            Err(e) => {
                eprintln!("error reading stdin: {}", e);
                process::exit(2);
            }
        }
    } else {
        match fs::read(&args[1]) {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(s) => s,
                Err(_) => {
                    eprintln!("error: input is not valid UTF-8");
                    process::exit(2);
                }
            },
            Err(e) => {
                eprintln!("error reading {}: {}", args[1], e);
                process::exit(2);
            }
        }
    };

    let result = mddag::parse(&input);
    match serde_json::to_string_pretty(&result) {
        Ok(json) => println!("{}", json),
        Err(e) => {
            eprintln!("error serializing output: {}", e);
            process::exit(1);
        }
    }
}
