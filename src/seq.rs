// seq.rs
use flate2::read::MultiGzDecoder;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::mem;
use std::path::PathBuf;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFormat {
    Fastq,
    Fasta,
}

impl fmt::Display for FileFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileFormat::Fasta => f.write_str("fasta"),
            FileFormat::Fastq => f.write_str("fastq"),
        }
    }
}

#[allow(dead_code)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compression {
    Gzipped,
    #[default]
    Uncompressed,
}

impl fmt::Display for Compression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Compression::Gzipped => f.write_str("gzipped"),
            Compression::Uncompressed => f.write_str("uncompressed"),
        }
    }
}

#[allow(dead_code)] // unused, it'd be nice if I can use it, else I'll clean it up
pub struct Filestate {
    file: PathBuf,
    format: FileFormat,
    compression: Compression,
}

impl fmt::Display for Filestate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.compression, self.format)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    pub id: String,
    pub seq: String,
}

pub struct Records<R: BufRead> {
    lines: io::Lines<R>,
    header: String,
    seq: String,
    is_fastq: bool,
    fq_step: usize,
}

impl<R: BufRead> Records<R> {
    pub fn new(reader: R) -> Self {
        Self {
            lines: reader.lines(),
            header: String::new(),
            seq: String::new(),
            is_fastq: false,
            fq_step: 0,
        }
    }
}

impl<R: BufRead> Iterator for Records<R> {
    type Item = io::Result<Record>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let line = match self.lines.next() {
                Some(Ok(l)) => l,
                Some(Err(e)) => return Some(Err(e)),
                None => {
                    if !self.seq.is_empty() {
                        return Some(Ok(Record {
                            id: self
                                .header
                                .trim_start_matches(['@', '>'])
                                .split_whitespace()
                                .next()?
                                .to_string(),
                            seq: mem::take(&mut self.seq),
                        }));
                    } else {
                        return None;
                    }
                }
            };
            if line.is_empty() {
                continue;
            }

            if line.starts_with(">") && self.fq_step == 0 {
                self.is_fastq = false;

                let prev_header = mem::replace(&mut self.header, line);

                if !self.seq.is_empty() {
                    let sequence = mem::take(&mut self.seq);
                    return Some(Ok(Record {
                        id: prev_header
                            .trim_start_matches('>')
                            .split_whitespace()
                            .next()?
                            .to_string(),
                        seq: sequence,
                    }));
                }
            } else if line.starts_with("@") && self.fq_step == 0 {
                self.header = line;
                self.is_fastq = true;
                self.fq_step = 1;
            } else if self.is_fastq {
                match self.fq_step {
                    1 => {
                        self.fq_step = 2;
                        return Some(Ok(Record {
                            id: self
                                .header
                                .trim_start_matches('@')
                                .split_whitespace()
                                .next()?
                                .to_string(),
                            seq: line,
                        }));
                    }
                    2 => self.fq_step = 3,
                    3 => self.fq_step = 0,
                    _ => {}
                }
            } else {
                self.seq.push_str(line.trim());
            }
        }
    }
}

// file I/O

pub fn from_path(path: &str) -> io::Result<Records<Box<dyn BufRead>>> {
    let mut gzipped = false;
    if path.ends_with(".gz") {
        gzipped = true;
    };
    let file = File::open(path)?;

    let reader: Box<dyn BufRead> = match gzipped {
        true => Box::new(BufReader::new(MultiGzDecoder::new(file))),
        false => Box::new(BufReader::new(file)),
    };
    Ok(Records::new(reader))
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;
    // for testing
    use std::io::Cursor;

    fn parse(s: &str) -> Vec<Record> {
        Records::new(Cursor::new(s))
            .collect::<io::Result<Vec<_>>>()
            .unwrap()
    }

    #[test]
    fn fasta_yields_both_records() {
        let recs = parse(">a\nGGG\n>b\nCCC\n");
        assert_eq!(recs.len(), 2);
        assert_eq!(recs[0].id, "a");
        assert_eq!(recs[1].id, "b");
        assert_eq!(recs[0].seq, "GGG");
        assert_eq!(recs[1].seq, "CCC");
    }

    #[test]
    fn fastq_yields_both_records() {
        let recs = parse("@r1\nGGG\n+\nIII\n@r2\nCCC\n+\nIII\n");
        assert_eq!(recs.len(), 2);
        assert_eq!(recs[0].id, "r1");
        assert_eq!(recs[0].seq, "GGG");
        assert_eq!(recs[1].id, "r2");
        assert_eq!(recs[1].seq, "CCC");
    }

    #[test]
    fn fasta_multiline_sequence_is_concatenated() {
        let recs = parse(">a\nGGG\nTTT\n>b\nCCC\n");
        assert_eq!(recs.len(), 2);
        assert_eq!(recs[0].seq, "GGGTTT");
    }

    #[test]
    fn last_record_without_trailing_newline() {
        let recs = parse(">a\nGGG");
        assert_eq!(recs.len(), 1);
        assert_eq!(recs[0].seq, "GGG");
    }

    #[test]
    fn fastq_quality_line_starting_with_gt_is_not_a_header() {
        let recs = parse("@r1\nGGG\n+\n>>>\n@r2\nCCC\n+\nIII\n");
        assert_eq!(recs.len(), 2);
        assert_eq!(recs[0].id, "r1");
        assert_eq!(recs[0].seq, "GGG");
        assert_eq!(recs[1].id, "r2");
        assert_eq!(recs[1].seq, "CCC");
    }

    #[test]
    fn fastq_quality_line_starting_with_at_is_not_a_header() {
        let recs = parse("@r1\nGGG\n+\n@@@\n@r2\nCCC\n+\nIII\n");
        assert_eq!(recs.len(), 2);
        assert_eq!(recs[1].id, "r2");
    }
}
