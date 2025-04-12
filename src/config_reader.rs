use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn get_config(config_file: &Path) -> Result<HashMap<String, String>, std::io::Error> {
    let mut config_map: HashMap<String, String> = HashMap::new();

    let mut config_file = File::open(config_file)?;
    let mut config_buffer = String::new();
    config_file.read_to_string(&mut config_buffer)?;
    for line in config_buffer.split('\n') {
        if let Some((k, v)) = line.split_once('=') {
            config_map.insert(k.to_string(), v.to_string());
        }
    }

    Ok(config_map)
}
