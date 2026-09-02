//! CLI (the command-line face of §9 three-level loading):
//!
//! - `mddag <file.md | ->`            full output (§8 contract JSON, default)
//! - `mddag --skeleton <file.md | ->`  L1 skeleton (node table + edges + diagnostics + graph)
//! - `mddag --body <id> <file.md | ->` L2 targeted body text (plain text)
//! - `mddag --projection <file.md | ->` appendix A edge projection labels
//! - `mddag --review <file.md | ->`    human review surface (disputes + warning summary)

use std::env;
use std::fs;
use std::io::{self, Read};
use std::process;

enum Mode {
    Full,
    Skeleton,
    Body(String),
    Projection,
    Review,
}

fn usage() -> ! {
    eprintln!(
        "usage:\n  mddag <file.md | ->\n  mddag --skeleton <file.md | ->\n  mddag --body <id> <file.md | ->\n  mddag --projection <file.md | ->\n  mddag --review <file.md | ->"
    );
    process::exit(2);
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let (mode, path) = match args.as_slice() {
        [p] => (Mode::Full, p.clone()),
        [flag, p] if flag == "--skeleton" => (Mode::Skeleton, p.clone()),
        [flag, id, p] if flag == "--body" => (Mode::Body(id.clone()), p.clone()),
        [flag, p] if flag == "--projection" => (Mode::Projection, p.clone()),
        [flag, p] if flag == "--review" => (Mode::Review, p.clone()),
        _ => usage(),
    };

    let input = if path == "-" {
        let mut buf = String::new();
        match io::stdin().read_to_string(&mut buf) {
            Ok(_) => buf,
            Err(e) => {
                eprintln!("error reading stdin: {}", e);
                process::exit(2);
            }
        }
    } else {
        match fs::read(&path) {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(s) => s,
                Err(_) => {
                    eprintln!("error: input is not valid UTF-8");
                    process::exit(2);
                }
            },
            Err(e) => {
                eprintln!("error reading {}: {}", path, e);
                process::exit(2);
            }
        }
    };

    fn emit(value: &impl serde::Serialize) {
        match serde_json::to_string_pretty(value) {
            Ok(json) => println!("{}", json),
            Err(e) => {
                eprintln!("error serializing output: {}", e);
                process::exit(1);
            }
        }
    }

    match mode {
        Mode::Full => {
            let result = mddag::parse(&input);
            emit(&result);
        }
        Mode::Skeleton => {
            // L1 = node table + normalized edge set + diagnostics (plus the
            // derivable global graph).
            let result = mddag::parse(&input);
            emit(&serde_json::json!({
                "nodes": result.nodes,
                "edges": result.edges,
                "diagnostics": result.diagnostics,
                "graph": result.graph,
            }));
        }
        Mode::Body(id) => match mddag::body_text(&input, &id) {
            Ok(text) => print!("{}", text),
            Err(_) => {
                eprintln!("error: node {:?} not found", id);
                process::exit(1);
            }
        },
        Mode::Projection => {
            let result = mddag::parse(&input);
            emit(&mddag::projection::project(&result));
        }
        Mode::Review => {
            let result = mddag::parse(&input);
            emit(&mddag::projection::review(&result));
        }
    }
}
