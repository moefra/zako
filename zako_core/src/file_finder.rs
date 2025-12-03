use std::ffi::OsString;
use std::path::PathBuf;
use std::env;
use tracing::{trace, warn};

pub struct FileFinder {
    pub paths: Vec<PathBuf>,
    pub prefixes: Vec<String>,
    pub suffixes: Vec<String>,
}

impl Default for FileFinder {
    fn default() -> Self {
        Self {
            paths: Vec::default(),
            suffixes: Vec::default(),
            prefixes: Vec::default(),
        }
    }
}

impl FileFinder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_path_env() -> Self {
        let prefixes = Vec::<String>::default();

        let paths: Vec<PathBuf> = env::split_paths(&env::var_os("PATH").unwrap_or_else(|| {
            warn!("failed to read variable PATH for file finder,skip");
            OsString::new()
        }))
        .collect();

        let suffixes = if cfg!(windows) {
            env::var_os("PATHEXT")
                .unwrap_or_else(|| {
                    warn!("failed to read variable PATHEXT for file finder,skip");
                    OsString::new()
                })
                .to_string_lossy()
                .split(';')
                .flat_map(|x| {
                    // In case of case-sensitive file system(like NTFS with case-sensitive), we need to consider both cases
                    let s = x.to_string();
                    let lower = s.to_ascii_lowercase();
                    if s == lower { vec![s] } else { vec![s, lower] }
                })
                .collect()
        } else {
            Vec::default()
        };

        FileFinder {
            paths,
            prefixes,
            suffixes,
        }
    }

    pub fn search<'a>(&'a self, target: &'a str) -> impl Iterator<Item = PathBuf> + 'a {
        self.paths.iter().flat_map(move |path| {
            std::iter::once("")
                .chain(self.prefixes.iter().map(|x| x.as_str()))
                .flat_map(move |prefix| {
                    std::iter::once("")
                        .chain(self.suffixes.iter().map(|x| x.as_str()))
                        .filter_map(move |suffix| {
                            let filename = format!("{}{}{}", prefix, target, suffix);
                            let full_path = path.join(&filename);

                            if full_path.is_file() {
                                trace!("Search {:?} - found", full_path);
                                Some(full_path)
                            } else {
                                trace!("Search {:?} - not found", full_path);
                                None
                            }
                        })
                })
        })
    }

    pub fn search_first(&self, target: &str) -> Option<PathBuf> {
        self.search(target).next()
    }
}
