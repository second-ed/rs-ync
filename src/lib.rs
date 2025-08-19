use indicatif::ProgressBar;
use itertools::Itertools;
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    env,
    error::Error,
    fmt, fs,
    hash::Hash,
    io::{self, Read, Write},
    path::{Path, PathBuf},
};
use text_colorizer::Colorize;

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
            fs::read_dir(path)
                .into_iter()
                .flat_map(|it| it.filter_map(Result::ok))
                .filter(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false))
                .map(|e| e.path()),
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
        let metadata = fs::metadata(path)?;
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
impl Default for FakeFileSystem {
    fn default() -> Self {
        Self::new()
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
    fn new(path: &Path, file_sys: &impl FileSystem) -> Result<Blob, io::Error> {
        let basename: PathBuf = path
            .file_name()
            .ok_or(io::Error::new(
                io::ErrorKind::InvalidData,
                "basename operation failed",
            ))?
            .into();
        let dir: PathBuf = path
            .parent()
            .ok_or(io::Error::new(
                io::ErrorKind::InvalidData,
                "parent operation failed",
            ))?
            .to_path_buf();

        let hash = file_sys.hash_file(path)?;
        let size = file_sys.size(path)?;
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
    paths: &[PathBuf],
    file_sys: &mut impl FileSystem,
) -> Result<Vec<Blob>, io::Error> {
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
    let blobs: Vec<Blob> = paths_to_blobs(&paths, file_sys).expect("Failed to parse blobs");
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

    for key in src_map.keys().sorted() {
        let blob = src_map.get(key).expect("expected key not in src_map");
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

    for key in dst_map.keys().sorted() {
        let blob = dst_map.get(key).expect("expected key not in dst_map");
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
    let bar = ProgressBar::new(file_plan.len().try_into().unwrap());
    for op in file_plan {
        match op {
            FileOp::CopyFile { src_path, dst_path } => file_sys.copy_file(&src_path, &dst_path),
            FileOp::DeleteFile { path } => file_sys.delete_file(&path),
        }
        .expect("{op} operation failed");
        bar.inc(1);
    }
    bar.finish();
    Ok(())
}

pub fn execute_rsync(args: Args, file_sys: &mut impl FileSystem) -> Result<(), io::Error> {
    let src_map = get_struct_map(&args.src_dir, file_sys);
    let dst_map = get_struct_map(&args.dst_dir, file_sys);

    let ops_plan = plan_file_movements(&args.dst_dir, &src_map, &dst_map);

    execute_file_movement_plan(file_sys, ops_plan)
}

// cli stuff
#[derive(Debug)]
pub struct Args {
    pub src_dir: PathBuf,
    pub dst_dir: PathBuf,
}

impl Args {
    pub fn new() -> Args {
        let args: Vec<String> = env::args().skip(1).collect();

        if args.len() != 2 {
            eprintln!("{} - rsync for two directories", "rs-ync".green());
            eprintln!("Usage: rs-ync `<SRC>` `<DST>`");
            eprintln!(
                "{} wrong number of args: expected 2 got {}. ",
                "Error:".bold().red(),
                args.len()
            );
            std::process::exit(1);
        }
        Args {
            src_dir: PathBuf::from(args[0].clone()),
            dst_dir: PathBuf::from(args[1].clone()),
        }
    }
}

impl Default for Args {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Args {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "src_dir: {:?} | dst_dir: {:?}",
            self.src_dir, self.dst_dir
        )
    }
}
