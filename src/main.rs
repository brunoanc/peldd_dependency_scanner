use clap::Parser;
use std::{collections::BTreeSet, path::Path, process::Command};

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
    pub fn get_all_dependencies(&self) -> BTreeSet<String> {
        let mut visited = BTreeSet::new();
        self.get_all_dependencies_inner(&self.file, &mut visited);
        visited
    }

    fn get_all_dependencies_inner(&self, file: &str, visited: &mut BTreeSet<String>) {
        if visited.insert(file.to_string()) {
            if self.verbose > 1 {
                println!("File {} already tested.", file);
            }
            return;
        }

        let filepath = if let Some(path) = get_filepath(file, &self.dll_path) {
            path
        } else {
            if self.verbose > 0 {
                println!("Couldn't find library {}", file);
            }
            return;
        };

        if self.verbose > 0 {
            println!("Checking file {:?}.", &filepath);
        }
        let mut files = Command::new("peldd");
        let files = files.arg(&filepath);
        if let Ok(out) = files.output() {
            let string = String::from_utf8_lossy(&out.stdout);
            for file in string.lines() {
                if self.full_path {
                    if let Some(a) = get_filepath(file, &self.dll_path) {
                        visited.insert(a);
                    } else {
                        if self.verbose > 0 {
                            println!("Couldn't find library {}", file);
                        }
                        continue;
                    }
                } else {
                    visited.insert(file.to_string());
                }
                self.get_all_dependencies_inner(file, visited);
            }
        }
    }
}

fn main() {
    let args = Args::parse();
    let deps = args.get_all_dependencies();
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
