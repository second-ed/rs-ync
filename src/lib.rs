use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::error::Error;
use std::hash::Hash;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::{fs, io};
use walkdir::WalkDir;

pub trait FileSystem {
    fn list_files<'a>(
        &'a mut self,
        path: &'a Path,
    ) -> Box<dyn Iterator<Item = std::path::PathBuf> + 'a>;
    fn move_file(&mut self, from: &Path, to: &Path) -> std::io::Result<()>;
    fn copy_file(&mut self, from: &Path, to: &Path) -> std::io::Result<()>;
    fn delete_file(&mut self, path: &Path) -> std::io::Result<()>;
    fn hash_file(&self, path: &Path) -> std::io::Result<String>;
    fn size(&self, path: &Path) -> std::io::Result<u64>;
}

pub struct RealFileSystem;

impl FileSystem for RealFileSystem {
    fn list_files<'a>(
        &'a mut self,
        path: &'a Path,
    ) -> Box<dyn Iterator<Item = std::path::PathBuf> + 'a> {
        Box::new(
            WalkDir::new(path)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| e.file_type().is_file())
                .map(|e| e.path().to_path_buf()),
        )
    }

    fn move_file(&mut self, from: &Path, to: &Path) -> std::io::Result<()> {
        fs::rename(from, to)
    }

    fn copy_file(&mut self, from: &Path, to: &Path) -> std::io::Result<()> {
        fs::copy(from, to).map(|_| ())
    }
    fn delete_file(&mut self, path: &Path) -> std::io::Result<()> {
        fs::remove_file(path)
    }
    fn hash_file(&self, path: &Path) -> std::io::Result<String> {
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
        Ok(format!("{:x}", hasher.finalize()))
    }
    fn size(&self, path: &Path) -> std::io::Result<u64> {
        let metadata = fs::metadata(&path)?;
        Ok(metadata.len())
    }
}

pub struct FakeFileSystem {
    pub files: HashMap<PathBuf, String>,
    pub operations: Vec<String>,
}

impl FakeFileSystem {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
            operations: Vec::new(),
        }
    }
}

impl FileSystem for FakeFileSystem {
    fn list_files<'a>(
        &'a mut self,
        path: &'a Path,
    ) -> Box<dyn Iterator<Item = std::path::PathBuf> + 'a> {
        self.operations.push(format!("list: `{}`", &path.display()));
        let base = path.to_path_buf();
        Box::new(
            self.files
                .keys()
                .filter(move |p| p.starts_with(&base))
                .cloned(),
        )
    }

    fn move_file(&mut self, from: &Path, to: &Path) -> std::io::Result<()> {
        if let Some(content) = self.files.remove(from) {
            self.files.insert(to.to_path_buf(), content);
            self.operations
                .push(format!("move: `{}` -> `{}`", from.display(), to.display()));
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "File not found"))
        }
    }

    fn copy_file(&mut self, from: &Path, to: &Path) -> std::io::Result<()> {
        if let Some(content) = self.files.get(from) {
            self.files.insert(to.to_path_buf(), content.clone());
            self.operations
                .push(format!("copy: `{}` -> `{}`", from.display(), to.display()));
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "File not found"))
        }
    }

    fn delete_file(&mut self, path: &Path) -> std::io::Result<()> {
        if self.files.remove(path).is_some() {
            self.operations
                .push(format!("delete: `{}`", path.display()));
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "File not found"))
        }
    }

    fn hash_file(&self, path: &Path) -> io::Result<String> {
        let content = self
            .files
            .get(path)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "File not found"))?;

        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        Ok(format!("{:x}", hasher.finalize()))
    }

    fn size(&self, path: &Path) -> std::io::Result<u64> {
        if let Some(content) = self.files.get(path) {
            Ok(content.len().try_into().unwrap())
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "File not found"))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Blob {
    pub path: PathBuf,
    pub basename: PathBuf,
    pub dir: PathBuf,
    hash: String,
    pub id: String,
    size: u64,
}

impl Blob {
    fn new(path: &PathBuf, file_sys: &impl FileSystem) -> Result<Blob, Box<dyn std::error::Error>> {
        let basename: PathBuf = path.file_name().ok_or("Missing file name in path")?.into();
        let dir: PathBuf = path.parent().ok_or("Missing directory in path")?.into();

        let hash = file_sys.hash_file(&path)?;
        let size = file_sys.size(&path)?;
        let id = format!(
            "{}-{}-{}",
            basename.to_string_lossy().into_owned(),
            hash,
            size
        );
        Ok(Blob {
            path: path.to_path_buf(),
            basename,
            dir,
            hash,
            id,
            size,
        })
    }
}

pub fn paths_to_blobs(
    paths: &Vec<PathBuf>,
    file_sys: &mut impl FileSystem,
) -> Result<Vec<Blob>, Box<dyn Error>> {
    paths.iter().map(|path| Blob::new(path, file_sys)).collect()
}

/// Creates a `HashMap` from a collection of items, keyed by a field extracted via `key_fn`.
pub fn struct_to_hashmap<K, T, I, F>(items: I, key_fn: F) -> HashMap<K, T>
where
    K: Eq + Hash,
    I: IntoIterator<Item = T>,
    F: Fn(&T) -> K,
{
    let mut map = HashMap::new();
    for item in items {
        let key = key_fn(&item);
        map.insert(key, item);
    }
    map
}

pub fn get_struct_map(root_dir: &PathBuf, file_sys: &mut impl FileSystem) -> HashMap<String, Blob> {
    let paths: Vec<PathBuf> = file_sys.list_files(Path::new(root_dir)).collect();
    let blobs = paths_to_blobs(&paths, file_sys).expect("Failed to parse blobs");
    struct_to_hashmap(blobs, |s| s.id.clone())
}

#[derive(Debug)]
pub enum FileOp {
    CopyFile {
        src_path: PathBuf,
        dst_path: PathBuf,
    },
    DeleteFile {
        path: PathBuf,
    },
}

pub fn plan_file_movements(
    dst_dir: &PathBuf,
    src_map: &HashMap<String, Blob>,
    dst_map: &HashMap<String, Blob>,
) -> Vec<FileOp> {
    let mut file_ops = Vec::new();

    for (key, blob) in src_map {
        match dst_map.get(key) {
            Some(_) => {}
            None => {
                file_ops.push(FileOp::CopyFile {
                    src_path: blob.path.clone(),
                    dst_path: PathBuf::from(dst_dir).join(blob.basename.clone()),
                });
            }
        }
    }

    for (key, blob) in dst_map {
        match src_map.get(key) {
            Some(_) => {}
            None => {
                file_ops.push(FileOp::DeleteFile {
                    path: blob.path.clone(),
                });
            }
        }
    }
    file_ops
}

pub fn execute_file_movement_plan(
    file_sys: &mut impl FileSystem,
    file_plan: Vec<FileOp>,
) -> Result<(), io::Error> {
    for op in file_plan {
        match op {
            FileOp::CopyFile { src_path, dst_path } => file_sys.copy_file(&src_path, &dst_path),
            FileOp::DeleteFile { path } => file_sys.delete_file(&path),
        }
        .expect("{op} operation failed");
    }
    Ok(())
}
