// main.rs
mod cli;
mod g4;
mod seq;

use clap::Parser;
use flate2::read::MultiGzDecoder;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
// use std::str::pattern::Pattern;

use cli::Args;
use g4::{Patterns, find_in_record};

fn main() {
    let args = Args::parse();

    let pats = Patterns::new();

    for file_path in &args.reads {
        if let Err(e) = stream_records(file_path, &pats) {
            eprintln!("Error processing {}: {}", file_path, e);
        }
    }
}

fn stream_records(filepath: &str, pats: &Patterns) -> io::Result<()> {
    let path = Path::new(filepath);

    let is_gzipped = path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("gz"));

    let file = File::open(path)?;

    let reader: Box<dyn BufRead> = if is_gzipped {
        Box::new(BufReader::new(MultiGzDecoder::new(file)))
    } else {
        Box::new(BufReader::new(file))
    };

    let mut header = String::new();
    let mut seq_acc = String::new();
    let mut is_fastq = false;
    let mut fq_step = 0;

    for line in reader.lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }

        if line.starts_with('>') {
            if !seq_acc.is_empty() {
                emit(&seq_acc, &header, pats);
                seq_acc.clear();
            }
            header = line;
            is_fastq = false;
        } else if line.starts_with('@') && fq_step == 0 {
            header = line;
            is_fastq = true;
            fq_step = 1;
        } else if is_fastq {
            match fq_step {
                1 => {
                    emit(&line, &header, pats);
                    fq_step = 2;
                }
                2 => fq_step = 3,
                3 => fq_step = 0,
                _ => {}
            }
        } else {
            seq_acc.push_str(line.trim());
        }
    }

    if !seq_acc.is_empty() {
        emit(&seq_acc, &header, pats)
    }

    Ok(())
}

fn emit(seq: &str, header: &str, pats: &Patterns) {
    let chrom = header
        .trim_start_matches(['@', '>'])
        .split_whitespace()
        .next()
        .unwrap_or("Unknown sequence header");

    // print the BED file to stdout
    for m in find_in_record(chrom, seq, pats) {
        println!(
            "{}\t{}\t{}\tG4\t{}\t{}",
            m.seq_id,
            m.start,
            m.end,
            m.span(),
            m.strand
        );
    }
}
