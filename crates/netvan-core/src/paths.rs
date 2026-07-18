use directories::ProjectDirs;
use std::path::PathBuf;

pub fn project_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("com", "Netvan", "Netvan")
}

pub fn data_dir() -> PathBuf {
    project_dirs()
        .map(|p| p.data_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from(".").join("netvan-data"))
}

pub fn db_path() -> PathBuf {
    data_dir().join("netvan.db")
}

pub fn ensure_data_dir() -> std::io::Result<PathBuf> {
    let dir = data_dir();
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub const PIPE_NAME: &str = r"\\.\pipe\netvan-service";
