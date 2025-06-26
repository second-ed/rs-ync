use hash_files::{FakeFileSystem, FileSystem};
use polars::prelude::*;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::{env, fmt};
use text_colorizer::Colorize;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::new();

    let mut file_sys = FakeFileSystem::new();

    let values = vec![
        ("dir/file_1.txt", "hello"),
        ("dir/file_2.txt", "hello"),
        ("dir/file_3.txt", "world"),
        ("dir/some_file.rs", "let mut thing = Vec::new()"),
        ("other_dir/file4.rs", "let x = 4;"),
        ("other_dir/file_5.txt", "hello"),
        ("other_dir/file_7.txt", "hello"),
        ("other_dir/file_2.txt", "hello"),
    ];

    for (name, content) in values {
        file_sys.files.insert(name.into(), content.to_string());
    }

    let src_paths: Vec<PathBuf> = file_sys.list_files(Path::new(&args.src_dir)).collect();
    let dst_paths: Vec<PathBuf> = file_sys.list_files(Path::new(&args.dst_dir)).collect();

    dbg!(&src_paths);

    let rows: Result<Vec<DirObj>, _> = src_paths
        .iter()
        .map(|path| DirObj::new(path, &file_sys))
        .collect();

    dbg!(&rows);
    match rows {
        Ok(rows) => {
            let df = df![
                "path" => rows.iter().map(|r| r.path.clone()).collect::<Vec<_>>(),
                "basename" => rows.iter().map(|r| r.basename.clone()).collect::<Vec<_>>(),
                "dir" => rows.iter().map(|r| r.dir.clone()).collect::<Vec<_>>(),
                "hash" => rows.iter().map(|r| r.hash.clone()).collect::<Vec<_>>(),
                "size" => rows.iter().map(|r| r.size).collect::<Vec<_>>(),
            ];
            dbg!(df);
        }
        _ => {
            eprintln!("failed to parse df");
        }
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Serialize)]
struct DirObj {
    path: String,
    basename: String,
    dir: String,
    hash: String,
    size: u64,
}

impl DirObj {
    fn new(
        path: impl AsRef<Path>,
        file_sys: &impl FileSystem,
    ) -> Result<DirObj, Box<dyn std::error::Error>> {
        let path = path.as_ref();

        let full_path = path.to_string_lossy().into_owned();

        let basename = path
            .file_name()
            .ok_or("Missing file name in path")?
            .to_string_lossy()
            .into_owned();

        let dir = path
            .parent()
            .ok_or("Missing directory in path")?
            .to_string_lossy()
            .into_owned();

        let hash = file_sys.hash_file(&path)?;
        let size = file_sys.size(&path)?;

        Ok(DirObj {
            path: full_path,
            basename,
            dir,
            hash,
            size,
        })
    }
}

// cli stuff
#[derive(Debug)]
struct Args {
    src_dir: PathBuf,
    dst_dir: PathBuf,
}

impl Args {
    fn new() -> Args {
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

impl fmt::Display for Args {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "src_dir: {:?} | dst_dir: {:?}",
            self.src_dir, self.dst_dir
        )
    }
}
