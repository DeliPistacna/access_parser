use std::{
    fs::File,
    io::{self, BufRead, BufReader, Seek},
    path::PathBuf,
};

#[derive(Debug)]
pub struct FileReader {
    file: File,
}

impl FileReader {
    pub fn new(filename: PathBuf) -> io::Result<Self> {
        let file = File::open(&filename)?;
        Ok(Self { file })
    }

    pub fn get_lines(&self) -> Result<impl Iterator<Item = io::Result<String>>, io::Error> {
        let mut reader = BufReader::new(&self.file);
        reader.rewind()?;
        Ok(reader.lines())
    }
}
