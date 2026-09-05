//! mddag CLI (v2.0-draft) — read-protocol commands, choices not free text:
//!   mddag balls <file>            L0 lodestone list
//!   mddag ball <slug> <file>      L1 single lodestone expansion
//!   mddag body <slug> <file>      L2 body fragment
//!   mddag sediment <file>         sediment index
//!   mddag check <file>            parse + diagnostics (exit 1 on errors)
//!
//! Output is the projection text only (stdout); diagnostics go to stderr.

use std::process::exit;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let code = match args.get(1).map(String::as_str) {
        Some("balls") => cmd_read(&args, |d| mddag::project::l0(d)),
        Some("ball") => cmd_ball(&args),
        Some("body") => cmd_body(&args),
        Some("sediment") => cmd_read(&args, |d| mddag::project::sediment_index(d)),
        Some("check") => cmd_check(&args),
        Some("decay") => cmd_decay(&args),
        Some("strip") => cmd_strip(&args),
        Some("library") => cmd_library(&args),
        Some("version") => {
            println!("mddag {}", mddag::PROTOCOL_VERSION);
            0
        }
        _ => {
            eprintln!("usage: mddag <balls|ball|body|sediment|check|decay|strip|library|version> ...");
            2
        }
    };
    exit(code);
}

fn cmd_read(args: &[String], f: impl Fn(&mddag::Doc) -> String) -> i32 {
    let Some(path) = args.get(2) else {
        eprintln!("usage: mddag {} <file>", args.get(1).map(String::as_str).unwrap_or("?"));
        return 2;
    };
    match std::fs::read_to_string(path) {
        Ok(text) => {
            let doc = mddag::scan(&text);
            print_diags(&doc);
            print!("{}", f(&doc));
            has_errors(&doc) as i32
        }
        Err(e) => {
            eprintln!("mddag: 读取失败 {path}: {e}");
            1
        }
    }
}

fn cmd_ball(args: &[String]) -> i32 {
    let (Some(slug), Some(path)) = (args.get(2), args.get(3)) else {
        eprintln!("usage: mddag ball <slug> <file>");
        return 2;
    };
    match std::fs::read_to_string(path) {
        Ok(text) => {
            let doc = mddag::scan(&text);
            print_diags(&doc);
            match mddag::project::l1(&doc, slug) {
                Some(out) => {
                    print!("{out}");
                    has_errors(&doc) as i32
                }
                None => {
                    eprintln!("mddag: 磁石不存在: {slug}");
                    1
                }
            }
        }
        Err(e) => {
            eprintln!("mddag: 读取失败 {path}: {e}");
            1
        }
    }
}

fn cmd_body(args: &[String]) -> i32 {
    let (Some(slug), Some(path)) = (args.get(2), args.get(3)) else {
        eprintln!("usage: mddag body <slug> <file>  （可选: body <slug> <file> <anchor>）");
        return 2;
    };
    let anchor = args.get(4).map(String::as_str);
    match std::fs::read_to_string(path) {
        Ok(text) => {
            let doc = mddag::scan(&text);
            print_diags(&doc);
            match mddag::project::l2(&doc, slug, anchor) {
                Some(out) => {
                    print!("{out}");
                    has_errors(&doc) as i32
                }
                None => {
                    eprintln!("mddag: 磁石或锚点不存在: {} {}", slug, anchor.unwrap_or(""));
                    1
                }
            }
        }
        Err(e) => {
            eprintln!("mddag: 读取失败 {path}: {e}");
            1
        }
    }
}

fn cmd_check(args: &[String]) -> i32 {
    let Some(path) = args.get(2) else {
        eprintln!("usage: mddag check <file>");
        return 2;
    };
    match std::fs::read_to_string(path) {
        Ok(text) => {
            let doc = mddag::scan(&text);
            if doc.diagnostics.is_empty() {
                println!("[ok] {} 磁石, 无诊断", doc.lodestones.len());
            } else {
                print_diags(&doc);
            }
            has_errors(&doc) as i32
        }
        Err(e) => {
            eprintln!("mddag: 读取失败 {path}: {e}");
            1
        }
    }
}

fn cmd_decay(args: &[String]) -> i32 {
    let (Some(path), Some(target)) = (args.get(2), args.get(3)) else {
        eprintln!("usage: mddag decay <file> <target>   # target = 磁石 slug 或 <slug>-full 沉淀条目");
        return 2;
    };
    match std::fs::read_to_string(path) {
        Ok(text) => {
            let r = mddag::ops::decay(&text, target);
            if !r.diagnostics.is_empty() {
                for d in &r.diagnostics {
                    eprintln!("mddag: {:?} {}:{} {}", d.severity, d.code, d.line, d.message);
                }
                return 1;
            }
            // rewrite in place
            match std::fs::write(path, &r.text) {
                Ok(_) => {
                    eprintln!("{}", r.audit);
                    // verify by reparsing
                    let doc = mddag::scan(&r.text);
                    if !doc.diagnostics.is_empty() {
                        eprintln!("mddag: warning: decay 后文档有诊断:");
                        print_diags(&doc);
                    }
                    println!("[ok] decay 完成, 磁石数 {}", doc.lodestones.len());
                    0
                }
                Err(e) => {
                    eprintln!("mddag: 写入失败 {path}: {e}");
                    1
                }
            }
        }
        Err(e) => {
            eprintln!("mddag: 读取失败 {path}: {e}");
            1
        }
    }
}

fn cmd_strip(args: &[String]) -> i32 {
    let Some(path) = args.get(2) else {
        eprintln!("usage: mddag strip <file>   # 剔除全部磁力线（窗口外会话降级）");
        return 2;
    };
    match std::fs::read_to_string(path) {
        Ok(text) => {
            let r = mddag::ops::strip(&text, &[]);
            match std::fs::write(path, &r.text) {
                Ok(_) => {
                    eprintln!("{}", r.audit);
                    let doc = mddag::scan(&r.text);
                    println!("[ok] strip 完成, 磁石数 {} 磁力线数 {}",
                        doc.lodestones.len(),
                        doc.lodestones.iter().map(|l| l.lines.len()).sum::<usize>());
                    0
                }
                Err(e) => { eprintln!("mddag: 写入失败 {path}: {e}"); 1 }
            }
        }
        Err(e) => { eprintln!("mddag: 读取失败 {path}: {e}"); 1 }
    }
}

fn cmd_library(args: &[String]) -> i32 {
    let Some(dir) = args.get(2) else {
        eprintln!("usage: mddag library <dir> [--keep N]   # N 默认 12（示例，可调）");
        return 2;
    };
    let mut keep = 12usize; // example default; --keep overrides (no hardcoding)
    let mut i = 3;
    while i < args.len() {
        if args[i] == "--keep" {
            if let Some(v) = args.get(i + 1) {
                if let Ok(n) = v.parse::<usize>() {
                    keep = n;
                }
            }
            i += 2;
        } else {
            i += 1;
        }
    }
    let mut sessions: Vec<mddag::library::Session> = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        eprintln!("mddag: 无法读取目录 {dir}");
        return 1;
    };
    let mut paths: Vec<std::path::PathBuf> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().map(|x| x == "md").unwrap_or(false))
        .collect();
    paths.sort();
    for p in paths {
        if let Ok(text) = std::fs::read_to_string(&p) {
            let doc = mddag::scan(&text);
            sessions.push(mddag::library::Session { path: p.display().to_string(), doc });
        }
    }
    print!("{}", mddag::library::index(&sessions, keep));
    0
}

fn print_diags(doc: &mddag::Doc) {
    for d in &doc.diagnostics {
        eprintln!("mddag: {:?} {}:{} {}", d.severity, d.code, d.line, d.message);
    }
}

fn has_errors(doc: &mddag::Doc) -> bool {
    doc.diagnostics.iter().any(|d| d.severity == mddag::Severity::Error)
}
