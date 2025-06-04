use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use std::{fs, io, path::PathBuf};
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
}

#[derive(Debug)]
pub enum FileOp {
    CopyFile {
        src_path: PathBuf,
        dst_path: PathBuf,
    },
    MoveFile {
        src_path: PathBuf,
        dst_path: PathBuf,
    },
    DeleteFile {
        path: PathBuf,
    },
}

pub fn get_hashes_map(
    file_sys: &impl FileSystem,
    paths: Vec<PathBuf>,
) -> HashMap<String, Vec<PathBuf>> {
    let mut hashes: HashMap<String, Vec<PathBuf>> = HashMap::new();

    for file in paths {
        hashes
            .entry(file_sys.hash_file(&file).unwrap())
            .or_default()
            .push(file);
    }
    hashes
}

pub fn plan_file_movements(
    dst_dir: &PathBuf,
    src_hashes: &HashMap<String, Vec<PathBuf>>,
    dst_hashes: &HashMap<String, Vec<PathBuf>>,
) -> Vec<FileOp> {
    let mut file_ops = Vec::new();

    for (hash, src_paths) in src_hashes {
        let mut src_iter = src_paths.iter();
        let src_path = src_iter.next().unwrap();

        for src_extra in src_iter {
            file_ops.push(FileOp::DeleteFile {
                path: src_extra.clone(),
            });
        }

        match dst_hashes.get(hash) {
            Some(dst_paths) => {
                let mut dst_iter = dst_paths.iter();
                let dst_path = dst_iter.next().unwrap();

                // delete first to avoid renaming a file and then deleting it
                for dst_extra in dst_iter {
                    file_ops.push(FileOp::DeleteFile {
                        path: dst_extra.clone(),
                    });
                }

                if src_path.file_name() != dst_path.file_name() {
                    file_ops.push(FileOp::MoveFile {
                        src_path: dst_path.clone(),
                        dst_path: dst_path.with_file_name(src_path.file_name().unwrap()),
                    });
                }
            }
            _ => {
                let dst_file = PathBuf::from(dst_dir).join(src_path.file_name().unwrap());
                file_ops.push(FileOp::CopyFile {
                    src_path: src_path.clone(),
                    dst_path: dst_file,
                });
            }
        }
    }

    for (hash, dst_paths) in dst_hashes {
        if !src_hashes.contains_key(hash) {
            for dst_path in dst_paths {
                file_ops.push(FileOp::DeleteFile {
                    path: dst_path.clone(),
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
            FileOp::MoveFile { src_path, dst_path } => file_sys.move_file(&src_path, &dst_path),
            FileOp::CopyFile { src_path, dst_path } => file_sys.copy_file(&src_path, &dst_path),
            FileOp::DeleteFile { path } => file_sys.delete_file(&path),
        }
        .expect("{op} operation failed");
    }
    Ok(())
}
