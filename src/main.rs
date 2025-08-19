use rs_ync::{Args, RealFileSystem, execute_rsync};
use std::io;

fn main() -> Result<(), io::Error> {
    let args = Args::new();

    let mut file_sys = RealFileSystem;
    execute_rsync(args, &mut file_sys)
}

#[allow(dead_code)]
fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>());
}
