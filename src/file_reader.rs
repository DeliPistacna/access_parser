use std::{
    fs::File,
    io::{self, BufRead, BufReader, Seek},
    path::PathBuf,
};
use rev_buf_reader::RevBufReader;

pub enum ReaderDirection {
    Normal,
    Reverse
}

#[derive(Debug)]
pub struct FileReader {
    file: File,
}

impl FileReader {
    pub fn new(filename: PathBuf) -> io::Result<Self> {
        let file = File::open(&filename)?;
        Ok(Self { file })
    }

    pub fn get_lines(&mut self, dir: ReaderDirection) -> Result<impl Iterator<Item = io::Result<String>>, io::Error> {

        self.file.rewind()?;

        let reader: Box<dyn BufRead> = match dir {
            ReaderDirection::Normal => Box::new(BufReader::new(&self.file)),
            ReaderDirection::Reverse => Box::new(RevBufReader::new(&self.file)),
        };
        
        Ok(reader.lines())
    }
}
