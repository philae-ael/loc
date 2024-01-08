use std::{fs::ReadDir, path::PathBuf};

pub struct Walker {
    cur_read_dir: ReadDir,
    dirs: Vec<PathBuf>,
}

impl Walker {
    pub fn new(path: PathBuf) -> std::io::Result<Self> {
        let read_dir = path.read_dir()?;
        Ok(Self {
            cur_read_dir: read_dir,
            dirs: vec![],
        })
    }
}

impl Iterator for Walker {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        for path in self.cur_read_dir.by_ref() {
            let dir_entry = path.unwrap();
            let file_type = dir_entry.file_type().unwrap();

            if file_type.is_dir() {
                self.dirs.push(dir_entry.path());
                continue;
            }
            return Some(dir_entry.path());
        }

        while let Some(path) = self.dirs.pop() {
            match path.read_dir() {
                Ok(read_dir) => {
                    self.cur_read_dir = read_dir;
                    return self.next();
                }
                Err(_) => continue,
            }
        }
        None
    }
}
