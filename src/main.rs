use hash_files::{
    execute_file_movement_plan, get_struct_map, paths_to_blobs, plan_file_movements,
    struct_to_hashmap, FakeFileSystem, FileSystem,
};
use std::path::{Path, PathBuf};
use std::{env, fmt};
use text_colorizer::Colorize;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::new();

    let mut file_sys = FakeFileSystem::new();

    let values = vec![
        ("main__dir/file_1.txt", "hey"),
        ("main__dir/file_2.txt", "hello"),
        ("main__dir/file_3.txt", "world"),
        ("main__dir/some_file.rs", "let mut thing = Vec::new()"),
        ("other_dir/file4.rs", "let x = 4;"),
        ("other_dir/file_5.txt", "hello"),
        ("other_dir/file_7.txt", "hello"),
        ("other_dir/file_2.txt", "hello"),
    ];

    for (name, content) in values {
        file_sys.files.insert(name.into(), content.to_string());
    }

    let src_map = get_struct_map(&args.src_dir, &mut file_sys);
    let dst_map = get_struct_map(&args.dst_dir, &mut file_sys);

    let ops_plan = plan_file_movements(&args.dst_dir, &src_map, &dst_map);
    let _ = execute_file_movement_plan(&mut file_sys, ops_plan);

    dbg!(file_sys.files);
    dbg!(file_sys.operations);

    Ok(())
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

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>());
}
