//! Stream openers for plain and gzip-compressed FASTQ, plus `-` for stdin / stdout.

use crate::Result;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;

/// `-` as path → stdin (read) or stdout (write).
pub fn path_is_stdio_dash(path: &Path) -> bool {
    path.as_os_str() == std::ffi::OsStr::new("-")
}

/// Heuristic: path ends with `.gz` (case-insensitive).
pub fn is_likely_gzip_path(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("gz"))
        .unwrap_or(false)
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ReaderOptions {
    pub stdin_gzip: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct WriterOptions {
    pub gzip_level: u32,
    pub stdout_gzip: bool,
}

impl Default for WriterOptions {
    fn default() -> Self {
        Self {
            gzip_level: 4,
            stdout_gzip: false,
        }
    }
}

/// Buffered FASTQ input (plain or gzip file, or stdin as `-`).
pub enum FastqReader {
    Plain(BufReader<File>),
    Gzip(Box<BufReader<GzDecoder<BufReader<File>>>>),
    StdinPlain(BufReader<std::io::Stdin>),
    StdinGzip(Box<BufReader<GzDecoder<BufReader<std::io::Stdin>>>>),
}

impl std::io::Read for FastqReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            FastqReader::Plain(r) => r.read(buf),
            FastqReader::Gzip(r) => r.read(buf),
            FastqReader::StdinPlain(r) => r.read(buf),
            FastqReader::StdinGzip(r) => r.read(buf),
        }
    }
}

impl std::io::BufRead for FastqReader {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        match self {
            FastqReader::Plain(r) => r.fill_buf(),
            FastqReader::Gzip(r) => r.fill_buf(),
            FastqReader::StdinPlain(r) => r.fill_buf(),
            FastqReader::StdinGzip(r) => r.fill_buf(),
        }
    }

    fn consume(&mut self, amt: usize) {
        match self {
            FastqReader::Plain(r) => r.consume(amt),
            FastqReader::Gzip(r) => r.consume(amt),
            FastqReader::StdinPlain(r) => r.consume(amt),
            FastqReader::StdinGzip(r) => r.consume(amt),
        }
    }
}

/// Open a FASTQ reader: `-` → stdin (plain or gzip per `opts`); `.gz` files are decompressed.
pub fn open_fastq_reader(path: &Path, opts: ReaderOptions) -> Result<FastqReader> {
    if path_is_stdio_dash(path) {
        if is_likely_gzip_path(path) {
            return Err(crate::Error::Config(
                "stdin path '-' does not support .gz suffix; use --stdin-gzip or pipe plain FASTQ".into(),
            ));
        }
        if opts.stdin_gzip {
            let stdin = std::io::stdin();
            let dec = GzDecoder::new(BufReader::new(stdin));
            return Ok(FastqReader::StdinGzip(Box::new(BufReader::new(dec))));
        }
        return Ok(FastqReader::StdinPlain(BufReader::new(std::io::stdin())));
    }
    let file = File::open(path)?;
    let buf = BufReader::new(file);
    let inner = if is_likely_gzip_path(path) {
        FastqReader::Gzip(Box::new(BufReader::new(GzDecoder::new(buf))))
    } else {
        FastqReader::Plain(buf)
    };
    Ok(inner)
}

/// Buffered FASTQ output (plain or gzip file, or stdout as `-`).
pub enum FastqWriter {
    Plain(BufWriter<File>),
    Gzip(BufWriter<GzEncoder<BufWriter<File>>>),
    StdoutPlain(BufWriter<std::io::Stdout>),
    StdoutGzip(BufWriter<GzEncoder<BufWriter<std::io::Stdout>>>),
}

impl Write for FastqWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            FastqWriter::Plain(w) => w.write(buf),
            FastqWriter::Gzip(w) => w.write(buf),
            FastqWriter::StdoutPlain(w) => w.write(buf),
            FastqWriter::StdoutGzip(w) => w.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            FastqWriter::Plain(w) => w.flush(),
            FastqWriter::Gzip(w) => w.flush(),
            FastqWriter::StdoutPlain(w) => w.flush(),
            FastqWriter::StdoutGzip(w) => w.flush(),
        }
    }
}

/// Open a FASTQ writer: `-` → stdout (plain or gzip per `opts`); `.gz` paths are gzip-compressed.
pub fn open_fastq_writer(path: &Path, opts: WriterOptions) -> Result<FastqWriter> {
    let level = opts.gzip_level.clamp(1, 9);
    let compression = Compression::new(level);
    if path_is_stdio_dash(path) {
        if is_likely_gzip_path(path) {
            return Err(crate::Error::Config(
                "stdout path '-' does not support .gz suffix; use --stdout-gzip".into(),
            ));
        }
        if opts.stdout_gzip {
            let stdout = std::io::stdout();
            let enc = GzEncoder::new(BufWriter::new(stdout), compression);
            return Ok(FastqWriter::StdoutGzip(BufWriter::new(enc)));
        }
        return Ok(FastqWriter::StdoutPlain(BufWriter::new(std::io::stdout())));
    }
    let file = File::create(path)?;
    let buf = BufWriter::new(file);
    let inner = if is_likely_gzip_path(path) {
        FastqWriter::Gzip(BufWriter::new(GzEncoder::new(buf, compression)))
    } else {
        FastqWriter::Plain(buf)
    };
    Ok(inner)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn gzip_roundtrip_smoke() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("t.fq.gz");
        {
            let mut w = open_fastq_writer(
                &path,
                WriterOptions {
                    gzip_level: 6,
                    ..Default::default()
                },
            )
            .unwrap();
            w.write_all(b"@x\nA\n+\n!\n").unwrap();
            w.flush().unwrap();
        }
        let mut r = open_fastq_reader(&path, ReaderOptions::default()).unwrap();
        let mut s = String::new();
        r.read_to_string(&mut s).unwrap();
        assert_eq!(s, "@x\nA\n+\n!\n");
    }
}
