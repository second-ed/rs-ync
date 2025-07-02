use hash_files::FakeFileSystem;

pub fn setup_fake_fs() -> FakeFileSystem {
    let mut file_sys = FakeFileSystem::new();

    let values = vec![
        ("dir/file_1.txt", "hey"),
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
    file_sys
}
