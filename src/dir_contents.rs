use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

use systemstat::Duration;

pub struct DirContents {
    // HashSet of all files, no folders, relative to the base directory given at construction.
    files: HashSet<PathBuf>,
    // HashSet of all file names, e.g. the last section without any folders, as strings.
    file_names: HashSet<String>,
    // HashSet of all folders, relative to the base directory given at construction.
    folders: HashSet<PathBuf>,
    // HashSet of all extensions found, without dots, e.g. "js" instead of ".js".
    extensions: Trie,
}

impl DirContents {
    #[cfg(test)]
    pub fn from_path(base: &Path) -> Result<Self, std::io::Error> {
        Self::from_path_with_timeout(base, Duration::from_secs(30))
    }

    pub fn from_path_with_timeout(base: &Path, timeout: Duration) -> Result<Self, std::io::Error> {
        let start = Instant::now();

        let mut folders: HashSet<PathBuf> = HashSet::new();
        let mut files: HashSet<PathBuf> = HashSet::new();
        let mut file_names: HashSet<String> = HashSet::new();
        let mut extensions = Trie::new();

        fs::read_dir(base)?
            .enumerate()
            .take_while(|(n, _)| {
                cfg!(test) // ignore timeout during tests
                || n & 0xFF != 0 // only check timeout once every 2^8 entries
                || start.elapsed() < timeout
            })
            .filter_map(|(_, entry)| entry.ok())
            .for_each(|entry| {
                let path = PathBuf::from(entry.path().strip_prefix(base).unwrap());
                if entry.path().is_dir() {
                    folders.insert(path);
                } else {
                    if !path.to_string_lossy().starts_with('.') {
                        // Extract the file extensions (yes, that's plural) from a filename.
                        // Why plural? Consider the case of foo.tar.gz. It's a compressed
                        // tarball (tar.gz), and it's a gzipped file (gz). We should be able
                        // to match both.

                        // find the minimal extension on a file. ie, the gz in foo.tar.gz
                        // NB the .to_string_lossy().to_string() here looks weird but is
                        // required to convert it from a Cow.
                        path.extension()
                            .map(|ext| extensions.insert(&ext.to_string_lossy().to_string()));

                        // find the full extension on a file. ie, the tar.gz in foo.tar.gz
                        path.file_name().map(|file_name| {
                            file_name
                                .to_string_lossy()
                                .split_once('.')
                                .map(|(_, after)| extensions.insert(&after.to_string()))
                        });
                    }
                    if let Some(file_name) = path.file_name() {
                        // this .to_string_lossy().to_string() is also required
                        file_names.insert(file_name.to_string_lossy().to_string());
                    }
                    files.insert(path);
                }
            });

        log::trace!(
            "Building HashSets of directory files, folders and extensions took {:?}",
            start.elapsed()
        );

        Ok(Self {
            files,
            file_names,
            folders,
            extensions,
        })
    }

    pub fn files(&self) -> impl Iterator<Item = &PathBuf> {
        self.files.iter()
    }

    pub fn has_file(&self, path: &str) -> bool {
        self.files.contains(Path::new(path))
    }

    pub fn has_file_name(&self, name: &str) -> bool {
        self.file_names.contains(name)
    }

    pub fn has_folder(&self, path: &str) -> bool {
        self.folders.contains(Path::new(path))
    }

    pub fn has_extension(&self, ext: &str) -> bool {
        self.extensions.contains(ext)
    }

    pub fn has_any_positive_file_name(&self, names: &[&str]) -> bool {
        names
            .iter()
            .any(|name| !name.starts_with('!') && self.has_file_name(name))
    }

    pub fn has_any_positive_folder(&self, paths: &[&str]) -> bool {
        paths
            .iter()
            .any(|path| !path.starts_with('!') && self.has_folder(path))
    }

    pub fn has_any_positive_extension(&self, exts: &[&str]) -> bool {
        exts.iter()
            .any(|ext| !ext.starts_with('!') && self.has_extension(ext))
    }

    pub fn has_any_negative_file_name(&self, names: &[&str]) -> bool {
        names
            .iter()
            .any(|name| name.starts_with('!') && self.has_file_name(&name[1..]))
    }

    pub fn has_any_negative_folder(&self, paths: &[&str]) -> bool {
        paths
            .iter()
            .any(|path| path.starts_with('!') && self.has_folder(&path[1..]))
    }

    pub fn has_any_negative_extension(&self, exts: &[&str]) -> bool {
        exts.iter()
            .any(|ext| ext.starts_with('!') && self.has_extension(&ext[1..]))
    }
}

const NO_CHILD: usize = 0;

struct Trie {
    nodes: Vec<TrieNode>,
}

impl Trie {
    pub fn new() -> Self {
        Self {
            nodes: vec![TrieNode::new(false)],
        }
    }

    pub fn insert(&mut self, s: &str) {
        let mut node_index = 0;
        for c in s.chars() {
            let mut child_index = self.nodes[node_index].children[c as usize];
            if child_index == NO_CHILD {
                child_index = self.get_new_node(false);
                self.nodes[node_index].children[c as usize] = child_index;
            }
            node_index = child_index;
        }
        self.nodes[node_index].is_word = true;
    }
    pub fn contains(&self, s: &str) -> bool {
        let mut node_index = 0;
        for c in s.chars() {
            let child_index = self.nodes[node_index].children[c as usize];
            if child_index == NO_CHILD {
                return false;
            }
            node_index = child_index;
        }
        self.nodes[node_index].is_word
    }

    fn get_new_node(&mut self, is_word: bool) -> usize {
        self.nodes.push(TrieNode::new(is_word));
        self.nodes.len() - 1
    }
}

impl std::fmt::Debug for Trie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("TRIE (members: {})", self.nodes.len()))
    }
}

struct TrieNode {
    is_word: bool,
    children: [usize; 256],
}

impl TrieNode {
    pub fn new(is_word: bool) -> Self {
        Self {
            is_word,
            children: [NO_CHILD; 256],
        }
    }
}
