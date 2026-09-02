//! CLI（§9 三级加载的命令行面）：
//!
//! - `mddag <file.md | ->`            全量输出（§8 契约 JSON，默认）
//! - `mddag --skeleton <file.md | ->`  L1 骨架（节点表 + 边 + 诊断 + 图）
//! - `mddag --body <id> <file.md | ->` L2 定点正文（纯文本）
//! - `mddag --projection <file.md | ->` 附录 A 边投影标签
//! - `mddag --review <file.md | ->`    人类审查面（分歧域 + 警告概览）

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
            // L1 = 节点表 + 规范化边集合 + 诊断（+ 可 derives 的全局图）。
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
