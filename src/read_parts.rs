//! Mutable read body (sequence + quality) for trimming.

use crate::fastq::OwnedFastqRead;
use crate::Error;
use crate::Result;

#[derive(Debug, Clone)]
pub struct MutableRead {
    pub name_line: String,
    pub plus_line: String,
    pub seq: Vec<u8>,
    pub qual: Vec<u8>,
}

impl MutableRead {
    pub fn from_owned(record: OwnedFastqRead) -> Result<Self> {
        let seq = trim_line_to_bytes(&record.sequence_line);
        let qual = trim_line_to_bytes(&record.quality_line);
        if seq.len() != qual.len() {
            return Err(Error::InvalidFastq {
                start_line: 0,
                reason: "sequence/quality length mismatch after parse".into(),
            });
        }
        Ok(Self {
            name_line: record.name_line,
            plus_line: record.plus_line,
            seq,
            qual,
        })
    }

    pub fn to_owned(&self) -> OwnedFastqRead {
        let mut sequence_line = String::with_capacity(self.seq.len() + 1);
        sequence_line.push_str(&String::from_utf8_lossy(&self.seq));
        sequence_line.push('\n');

        let mut quality_line = String::with_capacity(self.qual.len() + 1);
        quality_line.push_str(&String::from_utf8_lossy(&self.qual));
        quality_line.push('\n');

        OwnedFastqRead {
            name_line: self.name_line.clone(),
            sequence_line,
            plus_line: self.plus_line.clone(),
            quality_line,
        }
    }

    pub fn len(&self) -> usize {
        self.seq.len()
    }

    pub fn is_empty(&self) -> bool {
        self.seq.is_empty()
    }
}

fn trim_line_to_bytes(line: &str) -> Vec<u8> {
    let mut v: Vec<u8> = line.trim_end_matches(['\r', '\n']).as_bytes().to_vec();
    v.make_ascii_uppercase();
    v
}
