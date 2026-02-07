use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Clone)]
pub(super) struct RootFolders {
    pub(super) folders: HashMap<String, PathBuf>,
}

#[derive(Clone)]
pub(super) struct Roots {
    pub(super) local: Option<RootFolders>,
    pub(super) roaming: Option<RootFolders>,
    pub(super) local_low: Option<RootFolders>,
    pub(super) program_data: Option<RootFolders>,
}

pub(super) fn build_roots() -> Roots {
    let local = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .filter(|p| p.is_dir())
        .map(list_root_folders);
    let roaming = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .filter(|p| p.is_dir())
        .map(list_root_folders);
    let local_low = std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .map(|p| p.join(r"AppData\LocalLow"))
        .filter(|p| p.is_dir())
        .map(list_root_folders);
    let program_data = std::env::var_os("PROGRAMDATA")
        .map(PathBuf::from)
        .filter(|p| p.is_dir())
        .map(list_root_folders);

    Roots {
        local,
        roaming,
        local_low,
        program_data,
    }
}

fn list_root_folders(root: PathBuf) -> RootFolders {
    let mut folders = HashMap::new();
    if let Ok(rd) = std::fs::read_dir(&root) {
        for e in rd.flatten() {
            let p = e.path();
            if !p.is_dir() {
                continue;
            }
            let name = e.file_name().to_string_lossy().to_string();
            let key = name.to_lowercase();
            folders.insert(key, p);
        }
    }
    RootFolders { folders }
}
