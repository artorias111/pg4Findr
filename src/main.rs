use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use flate2::read::MultiGzDecoder;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[arg(short, long, num_args = 1..)]
    reads: Vec<String>,
}

fn main() {
    let args = Args::parse();

    let g_pat = Regex::new(r"(?i)G{3,}[ATCGN]{1,7}G{3,}[ATCGN]{1,7}G{3,}[ATCGN]{1,7}G{3,}").unwrap();
    let c_pat = Regex::new(r"(?i)C{3,}[ATCGN]{1,7}C{3,}[ATCGN]{1,7}C{3,}[ATCGN]{1,7}C{3,}").unwrap();

    for file_path in &args.reads {
        if let Err(e) = stream_records(file_path, &g_pat, &c_pat) {
            eprintln!("Error processing {}: {}", file_path, e);
        }
    }
}

fn stream_records(filepath: &str, g_re: &Regex, c_re: &Regex) -> io::Result<()> {
    let path = Path::new(filepath);
    let is_gzipped = path.extension().map_or(false, |ext| ext.eq_ignore_ascii_case("gz"));
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
        if line.is_empty() { continue; }

        if line.starts_with('>') {
            if !seq_acc.is_empty() {
                find_pg(&seq_acc, g_re, &header, "+");
                find_pg(&seq_acc, c_re, &header, "-");
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
                    find_pg(&line, g_re, &header, "+");
                    find_pg(&line, c_re, &header, "-");
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
        find_pg(&seq_acc, g_re, &header, "+");
        find_pg(&seq_acc, c_re, &header, "-");
    }

    Ok(())
}

fn find_pg(seq: &str, re: &Regex, header: &str, strand: &str) {
    let chrom = header
        .trim_start_matches(|c| c == '@' || c == '>')
        .split_whitespace()
        .next()
        .unwrap_or("unknown");

    for mat in re.find_iter(seq) {
        let start = mat.start();
        let end = mat.end();
        let length = end - start;
        println!("{}\t{}\t{}\t{}\t{}\t{}", chrom, start, end, "G4", length, strand);
    }
}
