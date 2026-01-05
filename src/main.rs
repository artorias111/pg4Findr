use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use flate2::read::MultiGzDecoder;
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    /// List of input FASTQ files (can be .gz or plain text)
    #[arg(short, long, num_args = 1.., value_delimiter = ' ')]
    reads: Vec<String>,
}

fn main() {
    let args = Args::parse();

    let forward_pattern = r"(?i)G{3,}[ATCGN]{1,7}G{3,}[ATCGN]{1,7}G{3,}[ATCGN]{1,7}G{3,}";
    let reverse_pattern = r"(?i)C{3,}[ATCGN]{1,7}C{3,}[ATCGN]{1,7}C{3,}[ATCGN]{1,7}C{3,}";
    
    let g_pat = Regex::new(forward_pattern).unwrap();
    let c_pat = Regex::new(reverse_pattern).unwrap();

    for file_path in &args.reads {
        eprintln!("Processing: {}", file_path);
        if let Err(e) = stream_fastq(file_path, &g_pat, &c_pat) {
            eprintln!("Error processing {}: {}", file_path, e);
        }
    }
}

fn stream_fastq(filepath: &str, g_re: &Regex, c_re: &Regex) -> io::Result<()> {
    let path = Path::new(filepath);

    let is_gzipped = path.extension()
        .map_or(false, |ext| ext.eq_ignore_ascii_case("gz"));

    let file = File::open(path)?;

    let reader: Box<dyn BufRead> = if is_gzipped {
        Box::new(BufReader::new(MultiGzDecoder::new(file)))
    } else {
        Box::new(BufReader::new(file))
    };

    let mut lines = reader.lines();
    while let Some(Ok(header)) = lines.next() {
        if let Some(Ok(seq)) = lines.next() {
            find_pg(&seq, g_re, &header, "+");
            find_pg(&seq, c_re, &header, "-");
        }
        let _ = lines.next(); // Skip +
        let _ = lines.next(); // Skip Quality
    }

    Ok(())
}

fn find_pg(seq: &str, re: &Regex, header: &str, strand: &str) {
    let chrom = header
        .trim_start_matches('@')
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
