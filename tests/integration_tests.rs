use rs_ync::{Args, execute_rsync};
use std::collections::HashMap;
use std::path::PathBuf;

mod common;

#[test]
fn test_edge_to_edge() {
    let mut file_sys = common::setup_fake_fs();

    let args = Args {
        src_dir: PathBuf::from("dir"),
        dst_dir: PathBuf::from("other_dir"),
    };
    let _ = execute_rsync(args, &mut file_sys);

    let expected_result = HashMap::from([
        (PathBuf::from("other_dir/file_1.txt"), String::from("hey")),
        (PathBuf::from("dir/file_3.txt"), String::from("world")),
        (
            PathBuf::from("dir/some_file.rs"),
            String::from("let mut thing = Vec::new()"),
        ),
        (PathBuf::from("other_dir/file_3.txt"), String::from("world")),
        (PathBuf::from("dir/file_2.txt"), String::from("hello")),
        (
            PathBuf::from("other_dir/some_file.rs"),
            String::from("let mut thing = Vec::new()"),
        ),
        (PathBuf::from("dir/file_1.txt"), String::from("hey")),
        (PathBuf::from("other_dir/file_2.txt"), String::from("hello")),
    ]);

    assert_eq!(file_sys.files, expected_result);

    let expected_ops = vec![
        "list: `dir`",
        "list: `other_dir`",
        "copy: `dir/file_1.txt` -> `other_dir/file_1.txt`",
        "copy: `dir/file_3.txt` -> `other_dir/file_3.txt`",
        "copy: `dir/some_file.rs` -> `other_dir/some_file.rs`",
        "delete: `other_dir/file4.rs`",
        "delete: `other_dir/file_5.txt`",
        "delete: `other_dir/file_7.txt`",
    ];

    assert_eq!(file_sys.operations, expected_ops);
}
