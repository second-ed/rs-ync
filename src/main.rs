use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs;
use std::io;
use std::io::Read;
use std::path::Path;
use text_colorizer::*;
use walkdir;

fn main() {
    let args = Args::new();
    process(args, list_files, hash_filestream)
}

fn process(
    args: Args,
    list_fn: fn(&String) -> walkdir::IntoIter,
    hash_fn: fn(&Path) -> io::Result<String>,
) {
    let hashes: HashMap<String, Vec<String>> = list_fn(&args.root_dir)
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter_map(|file| {
            let path = file.path();
            hash_fn(path)
                .ok()
                .map(|hash| (hash, path.display().to_string()))
        })
        .fold(HashMap::new(), |mut acc, (hash, path)| {
            acc.entry(hash).or_default().push(path);
            acc
        });

    dbg!(hashes);
}

fn list_files(root_dir: &String) -> walkdir::IntoIter {
    return walkdir::WalkDir::new(root_dir).into_iter();
}

fn read_file(path: &Path) -> Result<Vec<u8>, io::Error> {
    let bytes: Vec<u8> = fs::read(path)?;
    Ok(bytes)
}

fn hash_bytes(bytes: &Vec<u8>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let result = hasher.finalize();
    format!("{:x}", result)
}

fn hash_filestream(path: &Path) -> io::Result<String> {
    let file = fs::File::open(path)?;
    let mut reader = io::BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192]; // 8KB

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_bytes() {
        let inp_vec = vec![0, 1, 2, 3, 4];
        let expected_result =
            String::from("08bb5e5d6eaac1049ede0893d30ed022b1a4d9b5b48db414871f51c9cb35283d");

        assert_eq!(hash_bytes(&inp_vec), expected_result);
    }
}
