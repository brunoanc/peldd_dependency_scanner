use lazy_static::lazy_static;
use std::{process::Command, sync::Mutex};

lazy_static! {
    static ref ACCESSED_FILES: Mutex<Vec<String>> = Mutex::new(vec![]);
}

fn main() {
    let args = std::env::args();
    if args.len() > 1 {
        let vec = args.collect::<Vec<String>>();
        let mut deps = vec![];
        for f in vec {
            deps.append(&mut get_all_dependencies(&f));
        }
        deps.sort();
        deps.dedup();
        for v in deps {
            println!("{}", v);
        }
    } else {
        println!("You need to specify the binary to search dependencies for.");
    }
}

fn get_all_dependencies(file: &str) -> Vec<String> {
    if let Ok(mut a) = ACCESSED_FILES.lock() {
        if a.contains(&file.to_string()) {
            return vec![];
        } else {
            a.push(file.to_string());
        }
    } else {
        println!("Mutex has been poisoned!");
        return vec![];
    }
    let mut f = vec![];
    let mut files = Command::new("peldd");
    let files = files.arg(file);
    if let Ok(out) = files.output() {
        let string = String::from_utf8_lossy(&out.stdout);
        for file in string.lines() {
            f.push(file.to_string());
            f.append(&mut get_all_dependencies(file));
        }
    }
    f
}
