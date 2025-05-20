use hash_files::{FakeFileSystem, FileSystem};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{env, fmt};
use text_colorizer::Colorize;

fn main() {
    let args = Args::new();

    let mut fake_fs = FakeFileSystem::new();

    let values = vec![
        ("dir/file_1.txt", "hello"),
        ("dir/file_2.txt", "hello"),
        ("dir/file_3.txt", "world"),
    ];

    for (name, content) in values {
        fake_fs.files.insert(name.into(), content.to_string());
    }

    let mut hashes: HashMap<String, Vec<String>> = HashMap::new();
    let paths: Vec<_> = fake_fs.list_files(Path::new(&args.root_dir)).collect();
    for file in paths {
        hashes
            .entry(fake_fs.hash_file(&file).unwrap())
            .or_default()
            .push(file.display().to_string());
    }
    dbg!(hashes);
}

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>());
}

#[derive(Debug)]
struct Args {
    root_dir: String,
}

impl Args {
    fn new() -> Args {
        let args: Vec<String> = env::args().skip(1).collect();

        if args.len() != 1 {
            eprintln!(
                "{} - Create directories for each file type",
                "file_sorter".green()
            );
            eprintln!("Usage: file_sorter <ROOT_DIR>");
            eprintln!(
                "{} wrong number of args: expected 1 got {}. ",
                "Error:".bold().red(),
                args.len()
            );
            std::process::exit(1);
        }
        Args {
            root_dir: args[0].clone(),
        }
    }
}

impl fmt::Display for Args {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "root_dir: {}", self.root_dir)
    }
}
