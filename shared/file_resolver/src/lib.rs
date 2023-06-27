use std::{
    fs,
    path::{Path, PathBuf},
};


/// `FileResolver` is represents an API to work with source files in a directory
struct FileResolver {
    /// `paths` 
    paths: Vec<PathBuf>,
    contents: Vec<String>,
}

impl FileResolver {
    fn new<P: AsRef<Path>>(dir_path: P) -> std::io::Result<Self> {
        // Get path names of the files
        let paths: Vec<PathBuf> = dir_path
            .as_ref()
            .read_dir()?
            .into_iter()
            .map(|file| file.expect("Unable to get path").path())
            .collect();

        dbg!(&paths);

        // Read the contents of those files
        let contents: Vec<String> = paths
            .iter()
            .map(|file| fs::read_to_string(file).expect("Unable to read `{file}`"))
            .collect();

        dbg!(&contents);

        Ok(Self { paths, contents })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let resolver = FileResolver::new("src/");
    }
}
