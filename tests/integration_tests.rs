use rs_ync::{execute_file_movement_plan, get_struct_map, plan_file_movements};
use std::collections::HashMap;
use std::path::PathBuf;

mod common;

#[test]
fn test_edge_to_edge() {
    let src_dir: PathBuf = PathBuf::from("dir");
    let dst_dir: PathBuf = PathBuf::from("other_dir");

    let mut file_sys = common::setup_fake_fs();

    let src_map = get_struct_map(&src_dir, &mut file_sys);
    let dst_map = get_struct_map(&dst_dir, &mut file_sys);

    let ops_plan = plan_file_movements(&dst_dir, &src_map, &dst_map);
    let _ = execute_file_movement_plan(&mut file_sys, ops_plan);

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
