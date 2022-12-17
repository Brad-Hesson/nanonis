use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    num::ParseFloatError,
    path::Path,
};

use itertools::{izip, Itertools};

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct DatFile {
    pub attributes: BTreeMap<String, String>,
    pub signals: BTreeMap<String, Vec<f64>>,
}
impl DatFile {
    pub fn read_from_file(path: impl AsRef<Path>) -> Result<Self, ReadError> {
        let reader = BufReader::new(File::open(path).unwrap());
        Self::read_from(reader)
    }
    pub fn read_from(reader: impl BufRead) -> Result<Self, ReadError> {
        let mut self_ = Self {
            attributes: BTreeMap::new(),
            signals: BTreeMap::new(),
        };
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
            .filter(|h| !h.is_empty())
            .map(str::to_string)
            .collect_vec();
        let mut signals = vec![vec![]; headers.len()];
        for line in &mut lines {
            for (i, value) in line.split('\t').enumerate().take(headers.len()) {
                signals[i].push(value.trim().parse()?);
            }
        }
        self_.signals.extend(izip!(headers, signals));
        Ok(self_)
    }
    pub fn write_to_file(&self, path: impl AsRef<Path>) -> Result<(), std::io::Error> {
        let writer = BufWriter::new(File::create(path)?);
        Self::write_to(&self, writer)?;
        Ok(())
    }
    pub fn write_to(&self, mut writer: impl Write) -> Result<(), std::io::Error> {
        for (key, value) in self.attributes.iter() {
            writeln!(writer, "{key}\t{value}")?;
        }
        writeln!(writer)?;
        writeln!(writer, "[DATA]")?;
        for header in self.signals.keys() {
            write!(writer, "{header}\t")?;
        }
        writeln!(writer)?;
        let mut signals = self
            .signals
            .values()
            .map(|v| v.iter().peekable())
            .collect_vec();
        while signals[0].peek().is_some() {
            for it in signals.iter_mut() {
                write!(writer, "{}\t", it.next().unwrap())?;
            }
            writeln!(writer)?;
        }
        Ok(())
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

#[cfg(test)]
mod tests {
    use std::io::BufWriter;

    use super::*;

    #[test]
    fn round_trip() {
        let path = r#"C:\Users\Brad\Desktop\code\actuator-project\data\ln-stack\0011\aquisitions\trap_80s_400v_100p_0o_002.dat"#;
        let df = DatFile::read_from_file(path).unwrap();
        println!("{:#?}", df.attributes);
        println!("{:#?}", df.signals.keys());
        let path2 = r#"C:\Users\Brad\Desktop\test2.dat"#;
        df.write_to(BufWriter::new(File::create(path2).unwrap()))
            .unwrap();
        let df2 = DatFile::read_from_file(path2).unwrap();
        assert_eq!(df, df2);
    }
}
