use std::fs::read;
use std::io::BufRead;
use std::num::ParseFloatError;
use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use itertools::{izip, Itertools};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct DatFile {
    pub attributes: BTreeMap<String, String>,
    pub signals: BTreeMap<String, Vec<f64>>,
}
impl DatFile {
    pub fn read_from_file(path: impl AsRef<Path>) -> Result<Self, ReadError> {
        let mut self_ = Self {
            attributes: BTreeMap::new(),
            signals: BTreeMap::new(),
        };
        let reader = BufReader::new(File::open(path)?);
        let mut lines = reader
            .lines()
            .filter_map(|v| v.ok())
            .filter(|line| !line.is_empty());
        for line in (&mut lines).take_while(|line| line.trim() != "[DATA]") {
            let (key, value) = line
                .split_once('\t')
                .ok_or_else(|| ReadError::EmptyAttr(line.clone()))?;
            self_
                .attributes
                .insert(key.trim().into(), value.trim().into());
        }
        let headers = lines
            .next()
            .unwrap()
            .split('\t')
            .map(str::trim)
            .map(str::to_string)
            .collect_vec();
        let mut signals = vec![vec![]; headers.len()];
        for line in &mut lines {
            for (i, value) in line.split('\t').enumerate() {
                signals[i].push(value.trim().parse()?);
            }
        }
        self_.signals.extend(izip!(headers, signals));
        Ok(self_)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ReadError {
    #[error("{0}")]
    IOError(#[from] std::io::Error),
    #[error("{0}")]
    ParseFloatError(#[from] ParseFloatError),
    #[error("Empty attribute: {0:?}")]
    EmptyAttr(String),
}

#[test]
fn feature() {
    let path = r#"C:\Users\Brad\Desktop\test.dat"#;
    let res = DatFile::read_from_file(path);
    match res {
        Ok(df) => {
            println!("{:#?}", df.attributes);
            println!("{:#?}", df.signals.keys());
        }
        Err(e) => println!("{e}"),
    }
}
