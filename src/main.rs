use clap::Parser;
use std::{collections::BTreeSet, path::Path, process::Command};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(min_values = 1)]
    files: Vec<String>,
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
    /// End each output line with \0 instead of \n
    /// Not setting this will error if any file paths contain \n
    #[clap(short = '0', long)]
    zero: bool,
}

impl Args {
    pub fn get_all_dependencies(&self) -> Result<BTreeSet<String>, ()> {
        let mut visited = BTreeSet::new();
        for file in &self.files {
            self.get_all_dependencies_inner(file, &mut visited)?;
        }
        Ok(visited)
    }

    fn get_all_dependencies_inner(
        &self,
        file: &str,
        visited: &mut BTreeSet<String>,
    ) -> Result<(), ()> {
        if visited.insert(file.to_string()) {
            if self.verbose > 1 {
                eprintln!("File {} already tested.", file);
            }
            return Ok(());
        }

        let filepath = if let Some(path) = get_filepath(file, &self.dll_path) {
            path
        } else {
            if self.verbose > 0 {
                eprintln!("Couldn't find library {}", file);
            }
            return Ok(());
        };

        if self.verbose > 0 {
            eprintln!("Checking file {:?}.", &filepath);
        }
        if let Ok(out) = Command::new("peldd").arg(&filepath).output() {
            let string = match String::from_utf8(out.stdout) {
                Ok(string) => string,
                Err(_) => {
                    eprintln!("`peldd` output is not valid UTF-8");
                    return Err(());
                }
            };
            for file in string.lines() {
                if self.full_path {
                    if let Some(a) = get_filepath(file, &self.dll_path) {
                        visited.insert(a);
                    } else {
                        if self.verbose > 0 {
                            eprintln!("Couldn't find library {}", file);
                        }
                        continue;
                    }
                } else {
                    visited.insert(file.to_string());
                }
                self.get_all_dependencies_inner(file, visited)?;
            }
        } else {
            eprintln!("`peldd` command exited with non-zero status");
            return Err(());
        }
        Ok(())
    }
}

fn main() {
    let args = Args::parse();
    let deps = match args.get_all_dependencies() {
        Ok(deps) => deps,
        Err(()) => std::process::exit(2),
    };

    for v in deps {
        if args.zero {
            print!("{}\0", v);
        } else {
            if v.contains('\n') {
                eprintln!("Path '{}' contains line breaks, call with --zero", v);
                std::process::exit(2);
            }
            println!("{}", v);
        }
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
