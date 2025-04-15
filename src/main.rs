use sha2::{Digest, Sha256};
use std::fs;
use std::io;
use std::io::Error;
use std::io::Read;
use std::path::Path;

fn main() {
    let paths = vec!["./.gitignore"];

    for p in paths {
        let path = Path::new(&p);
        dbg!(hash_bytes(&read_file(path).unwrap()));
        dbg!(hash_filestream(path).unwrap());
    }
}

fn read_file(path: &Path) -> Result<Vec<u8>, Error> {
    let bytes = fs::read(path)?;
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

// fn print_type_of<T>(_: &T) {
//     println!("{}", std::any::type_name::<T>());
// }

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
