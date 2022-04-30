use clap::Parser;
use lazy_static::lazy_static;
use std::{path::Path, process::Command, sync::Mutex};

lazy_static! {
    static ref ACCESSED_FILES: Mutex<Vec<String>> = Mutex::new(vec![]);
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    file: String,
    #[clap(
        short,
        long,
        default_value = "/usr/x86_64-w64-mingw32/sys-root/mingw/bin/"
    )]
    dll_path: String,
    #[clap(short, long, parse(from_occurrences))]
    verbose: u8,
    #[clap(short, long)]
    full_path: bool,
}

impl Args {
    pub fn get_all_dependencies(&self) -> Vec<String> {
        self.get_all_dependencies_inner(&self.file)
    }

    fn get_all_dependencies_inner(&self, file: &str) -> Vec<String> {
        if let Ok(mut a) = ACCESSED_FILES.lock() {
            if a.contains(&file.to_string()) {
                if self.verbose > 1 {
                    println!("File {} already tested.", file);
                }
                return vec![];
            } else {
                a.push(file.to_string());
            }
        } else {
            if self.verbose > 0 {
                println!("Mutex has been poisoned!");
            }
            return vec![];
        }

        let filepath = if let Some(path) = get_filepath(file, &self.dll_path) {
            path
        } else {
            if self.verbose > 0 {
                println!("Couldn't find library {}", file);
            }
            return vec![];
        };

        if self.verbose > 0 {
            println!("Checking file {:?}.", &filepath);
        }
        let mut f = vec![];
        let mut files = Command::new("peldd");
        let files = files.arg(&filepath);
        if let Ok(out) = files.output() {
            let string = String::from_utf8_lossy(&out.stdout);
            for file in string.lines() {
                if self.full_path {
                    if let Some(a) = get_filepath(file, &self.dll_path) {
                        f.push(a);
                    } else {
                        if self.verbose > 0 {
                            println!("Couldn't find library {}", file);
                        }
                        continue;
                    }
                } else {
                    f.push(file.to_string());
                }
                f.append(&mut self.get_all_dependencies_inner(file));
            }
        }
        f
    }
}

fn main() {
    let args = Args::parse();
    let mut deps = args.get_all_dependencies();
    deps.sort();
    deps.dedup();
    for v in deps {
        println!("{}", v);
    }
}

fn get_filepath(file: &str, dll_path: &str) -> Option<String> {
    let mut filepath = Path::new(file);
    let fp = format!("{}/{}", dll_path, file);
    if !filepath.exists() {
        filepath = Path::new(&fp);
        if !filepath.exists() {
            return None;
        }
    }
    Some(filepath.canonicalize().unwrap().display().to_string())
}
