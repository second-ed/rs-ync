use hash_files::{FakeFileSystem, FileSystem};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{env, fmt, io};
use text_colorizer::Colorize;

fn main() {
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
    let src_hashes = dbg!(get_hashes_map(&file_sys, src_paths));

    let dst_paths: Vec<PathBuf> = file_sys.list_files(Path::new(&args.dst_dir)).collect();
    let dst_hashes = dbg!(get_hashes_map(&file_sys, dst_paths));

    let file_plan = dbg!(plan_file_movements(&args, &src_hashes, &dst_hashes));
    let _ = execute_file_movement_plan(&mut file_sys, file_plan);

    dbg!(file_sys.files);
    dbg!(file_sys.operations);
}

fn get_hashes_map(
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

fn plan_file_movements(
    args: &Args,
    src_hashes: &HashMap<String, Vec<PathBuf>>,
    dst_hashes: &HashMap<String, Vec<PathBuf>>,
) -> Vec<FileOp> {
    let mut file_ops = Vec::new();

    for (hash, src_paths) in src_hashes {
        let src_path = &src_paths[0];

        for src_extra in src_paths.iter().skip(1) {
            file_ops.push(FileOp::DeleteFile {
                path: src_extra.clone(),
            });
        }

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

fn execute_file_movement_plan(
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
