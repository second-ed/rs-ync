use hash_files::{
    execute_file_movement_plan, get_hashes_map, plan_file_movements, FakeFileSystem, FileSystem,
};
use std::path::{Path, PathBuf};
use std::{env, fmt};
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

    let file_plan = dbg!(plan_file_movements(&args.dst_dir, &src_hashes, &dst_hashes));
    let _ = execute_file_movement_plan(&mut file_sys, file_plan);

    dbg!(file_sys.files);
    dbg!(file_sys.operations);
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
