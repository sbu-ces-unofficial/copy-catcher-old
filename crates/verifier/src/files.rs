use walkdir::WalkDir;

pub fn count(path: &str) -> u32 {
    let mut num_files = 0;

    for entry in WalkDir::new(path)
        .contents_first(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_dir() {
            num_files += 1;
        }
    }
    num_files
}
