use walkdir::DirEntry;

pub fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

pub fn is_file(dir_entry: &Result<DirEntry, walkdir::Error>) -> bool {
    let Ok(dir_entry) = dir_entry else {
        return false;
    };
    let Ok(ent) = dir_entry.metadata() else {
        return false;
    };
    ent.is_file()
}
