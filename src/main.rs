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
        ("dir/some_file.rs", "let mut thing = Vec::new()"),
        ("other_dir/file4.rs", "let x = 4;"),
        ("other_dir/file_5.txt", "hello"),
        ("other_dir/file_7.txt", "hello"),
        ("other_dir/file_2.txt", "hello"),
    ];

    for (name, content) in values {
        fake_fs.files.insert(name.into(), content.to_string());
    }

    let src_paths: Vec<PathBuf> = fake_fs.list_files(Path::new(&args.src_dir)).collect();
    let src_hashes = dbg!(get_hashes_map(&fake_fs, src_paths));

    let dst_paths: Vec<PathBuf> = fake_fs.list_files(Path::new(&args.dst_dir)).collect();
    let dst_hashes = dbg!(get_hashes_map(&fake_fs, dst_paths));

    let file_plan = dbg!(plan_file_movements(&args, &src_hashes, &dst_hashes));

    for op in file_plan {
        match op {
            FileOp::MoveFile {
                src_path: src_path,
                dst_path: dst_path,
            } => fake_fs.move_file(&src_path, &dst_path),
            FileOp::CopyFile {
                src_path: src_path,
                dst_path: dst_path,
            } => fake_fs.copy_file(&src_path, &dst_path),
            FileOp::DeleteFile { path: path } => fake_fs.delete_file(&path),
        }
        .expect("{op} operation failed");
    }
    dbg!(fake_fs.files);
    dbg!(fake_fs.operations);
}

fn get_hashes_map(fs: &impl FileSystem, paths: Vec<PathBuf>) -> HashMap<String, Vec<PathBuf>> {
    let mut hashes: HashMap<String, Vec<PathBuf>> = HashMap::new();

    for file in paths {
        hashes
            .entry(fs.hash_file(&file).unwrap())
            .or_default()
            .push(file);
    }
    hashes
}

fn plan_file_movements(
    args: &Args,
    src_hashes: &HashMap<String, Vec<PathBuf>>,
    dst_hashes: &HashMap<String, Vec<PathBuf>>,
) -> Vec<FileOp> {
    let mut file_ops = Vec::new();

    for (hash, src_paths) in src_hashes {
        let src_path = &src_paths[0];
        match dst_hashes.get(hash) {
            Some(dst_paths) => {
                let dst_path = &dst_paths[0];

                // delete first to avoid renaming a file and then deleting it
                for dst_extra in dst_paths.iter().skip(1) {
                    file_ops.push(FileOp::DeleteFile {
                        path: dst_extra.clone(),
                    });
                }

                file_ops.push(FileOp::MoveFile {
                    src_path: dst_path.clone(),
                    dst_path: dst_path.with_file_name(src_path.file_name().unwrap()),
                });
            }
            None => {
                let dst_file = PathBuf::from(&args.dst_dir).join(src_path.file_name().unwrap());
                file_ops.push(FileOp::CopyFile {
                    src_path: src_path.clone(),
                    dst_path: dst_file,
                });
            }
        }
    }

    for (hash, dst_paths) in dst_hashes {
        match src_hashes.get(hash) {
            Some(_) => {}
            None => {
                for dst_path in dst_paths {
                    file_ops.push(FileOp::DeleteFile {
                        path: dst_path.clone(),
                    });
                }
            }
        }
    }
    file_ops
}

fn replace_directory(src_path: &PathBuf, dst_dir: &PathBuf) -> PathBuf {
    match src_path.file_name() {
        Some(file_name) => dst_dir.join(file_name),
        None => dst_dir.to_path_buf(), // fallback: no file name
    }
}

#[derive(Debug)]
enum FileOp {
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
