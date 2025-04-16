use std::{collections::HashMap, fs::File, io::{BufRead, BufReader, Seek, Write}, path::PathBuf};
// use crate::file_reader::FileReader;


pub struct Cache{
    path:PathBuf,
    cache: HashMap<String,String>
}

impl Cache{

    pub fn new() -> Result<Self, std::io::Error> {
        let mut c = Cache {
            path: PathBuf::from("acccess_parser.cache"),
            cache: HashMap::new()
        };
        c.read()?;
        Ok(c)
    }

    pub fn set(&mut self, key: &str, value: &str) { 
        self.cache.insert(key.to_string(), value.to_string());
    }

    pub fn get(&self, key: &str) -> Result<Option<String>, std::io::Error> {
        match self.cache.get(key) {
            Some(value) => {Ok(Some(value.to_string()))},
            _ => {Ok(None)}
        }
    }


    pub fn read(&mut self) -> Result<(), std::io::Error> {
        let path=  self.path.to_path_buf();
        if !path.exists() {
            return Ok(())
        }

        let file = File::open(self.path.clone())?;
        let mut reader = BufReader::new(file);
        reader.rewind()?;

        for line in reader.lines() {
            if let Some((key,value)) = line?.split_once('=') {
                self.set(key, value);
            }
        };

        Ok(())
    }

    pub fn persist(&self) -> Result<(), std::io::Error> { 
        let mut file = File::create(&self.path)?;
        let mut buffer = String::new();
        for (k,v) in self.cache.iter(){
            buffer.push_str(k);
            buffer.push('=');
            buffer.push_str(v);
            buffer.push('\n');
        }
        file.write_all(buffer.as_bytes())?;
        Ok(())
    }
}
