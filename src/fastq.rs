//! FASTQ record types and validation.

use crate::Error;
use crate::Result;

/// One FASTQ record, typically built from four `read_line` results (including trailing `\n` if present).
#[derive(Debug, Clone)]
pub struct OwnedFastqRead {
    pub name_line: String,
    pub sequence_line: String,
    pub plus_line: String,
    pub quality_line: String,
}

impl OwnedFastqRead {
    pub fn validate(&self, start_line: usize) -> Result<()> {
        validate_name_line(&self.name_line, start_line)?;
        validate_sequence_line(&self.sequence_line, start_line + 1)?;
        validate_plus_line(&self.plus_line, start_line + 2)?;
        validate_quality_line(&self.quality_line, &self.sequence_line, start_line + 3)?;
        Ok(())
    }

    pub fn write_to<W: std::io::Write>(&self, mut w: W) -> std::io::Result<()> {
        w.write_all(self.name_line.as_bytes())?;
        w.write_all(self.sequence_line.as_bytes())?;
        w.write_all(self.plus_line.as_bytes())?;
        w.write_all(self.quality_line.as_bytes())?;
        Ok(())
    }

    pub fn name_bytes_without_at(&self) -> &[u8] {
        let b = self.name_line.as_bytes();
        if b.first() == Some(&b'@') {
            &b[1..]
        } else {
            b
        }
    }
}

/// Borrowing view of a FASTQ record stored in a single buffer (four consecutive lines).
#[derive(Debug, Clone, Copy)]
pub struct FastqRead<'a> {
    pub name_line: &'a str,
    pub sequence_line: &'a str,
    pub plus_line: &'a str,
    pub quality_line: &'a str,
    pub start_line: usize,
}

impl<'a> FastqRead<'a> {
    pub fn from_block(block: &'a str, start_line: usize) -> Result<Self> {
        let mut lines = block.lines();
        let name_line = lines.next().ok_or(Error::UnexpectedEof { start_line })?;
        let sequence_line = lines.next().ok_or(Error::UnexpectedEof { start_line })?;
        let plus_line = lines.next().ok_or(Error::UnexpectedEof { start_line })?;
        let quality_line = lines.next().ok_or(Error::UnexpectedEof { start_line })?;
        if lines.next().is_some() {
            return Err(Error::InvalidFastq {
                start_line,
                reason: "expected exactly four lines per record".into(),
            });
        }

        let read = Self {
            name_line,
            sequence_line,
            plus_line,
            quality_line,
            start_line,
        };
        read.validate()?;
        Ok(read)
    }

    fn validate(&self) -> Result<()> {
        validate_name_line(self.name_line, self.start_line)?;
        validate_sequence_line(self.sequence_line, self.start_line + 1)?;
        validate_plus_line(self.plus_line, self.start_line + 2)?;
        validate_quality_line(self.quality_line, self.sequence_line, self.start_line + 3)?;
        Ok(())
    }

    pub fn write_to<W: std::io::Write>(&self, mut w: W) -> std::io::Result<()> {
        w.write_all(self.name_line.as_bytes())?;
        w.write_all(b"\n")?;
        w.write_all(self.sequence_line.as_bytes())?;
        w.write_all(b"\n")?;
        w.write_all(self.plus_line.as_bytes())?;
        w.write_all(b"\n")?;
        w.write_all(self.quality_line.as_bytes())?;
        w.write_all(b"\n")?;
        Ok(())
    }
}

/// Mutable builder for streaming four lines into an [`OwnedFastqRead`].
#[derive(Debug, Default)]
pub struct FastqReadMut {
    pub name_line: String,
    pub sequence_line: String,
    pub plus_line: String,
    pub quality_line: String,
}

impl FastqReadMut {
    pub fn clear(&mut self) {
        self.name_line.clear();
        self.sequence_line.clear();
        self.plus_line.clear();
        self.quality_line.clear();
    }

    pub fn read_four_lines<R: std::io::BufRead>(
        &mut self,
        reader: &mut R,
        global_line: &mut usize,
    ) -> std::io::Result<Option<usize>> {
        self.clear();
        let record_start = *global_line + 1;
        *global_line += 1;
        if reader.read_line(&mut self.name_line)? == 0 {
            return Ok(None);
        }
        *global_line += 1;
        if reader.read_line(&mut self.sequence_line)? == 0 {
            return Err(unexpected_eof());
        }
        *global_line += 1;
        if reader.read_line(&mut self.plus_line)? == 0 {
            return Err(unexpected_eof());
        }
        *global_line += 1;
        if reader.read_line(&mut self.quality_line)? == 0 {
            return Err(unexpected_eof());
        }
        Ok(Some(record_start))
    }

    pub fn as_owned(&self) -> OwnedFastqRead {
        OwnedFastqRead {
            name_line: self.name_line.clone(),
            sequence_line: self.sequence_line.clone(),
            plus_line: self.plus_line.clone(),
            quality_line: self.quality_line.clone(),
        }
    }

    pub fn validate(&self, record_start: usize) -> Result<()> {
        self.as_owned().validate(record_start)
    }
}

fn unexpected_eof() -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "truncated FASTQ record")
}

fn validate_name_line(name: &str, line: usize) -> Result<()> {
    let t = name.trim_end_matches(['\r', '\n']);
    if !t.starts_with('@') {
        return Err(Error::InvalidFastq {
            start_line: line,
            reason: "read name line must start with '@'".into(),
        });
    }
    if t.len() <= 1 {
        return Err(Error::InvalidFastq {
            start_line: line,
            reason: "read name is empty after '@'".into(),
        });
    }
    Ok(())
}

fn validate_sequence_line(seq: &str, line: usize) -> Result<()> {
    let t = seq.trim_end_matches(['\r', '\n']);
    if t.is_empty() {
        return Err(Error::InvalidFastq {
            start_line: line,
            reason: "sequence line is empty".into(),
        });
    }
    for (i, ch) in t.char_indices() {
        if !ch.is_ascii_alphabetic() {
            return Err(Error::InvalidFastq {
                start_line: line,
                reason: format!("invalid sequence character '{ch}' at column {}", i + 1),
            });
        }
    }
    Ok(())
}

fn validate_plus_line(plus: &str, line: usize) -> Result<()> {
    let t = plus.trim_end_matches(['\r', '\n']);
    if !t.starts_with('+') {
        return Err(Error::InvalidFastq {
            start_line: line,
            reason: "third FASTQ line must start with '+'".into(),
        });
    }
    Ok(())
}

fn validate_quality_line(qual: &str, seq: &str, line: usize) -> Result<()> {
    let qt = qual.trim_end_matches(['\r', '\n']);
    let st = seq.trim_end_matches(['\r', '\n']);
    if qt.len() != st.len() {
        return Err(Error::InvalidFastq {
            start_line: line,
            reason: format!(
                "quality length {} does not match sequence length {}",
                qt.len(),
                st.len()
            ),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_record_from_block() {
        let block = "@read1\nACGT\n+\nIIII";
        let r = FastqRead::from_block(block, 1).unwrap();
        assert_eq!(r.name_line, "@read1");
        assert_eq!(r.sequence_line, "ACGT");
    }

    #[test]
    fn rejects_mismatched_quality_length() {
        let block = "@read1\nACGT\n+\nII";
        let err = FastqRead::from_block(block, 1).unwrap_err();
        match err {
            Error::InvalidFastq { .. } => {}
            e => panic!("unexpected error: {e:?}"),
        }
    }
}
