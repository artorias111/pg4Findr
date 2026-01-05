use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use flate2::read::MultiGzDecoder;
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    reads: String,
}

fn main() {
    let args = Args::parse();
    let path = &args.reads;
    eprintln!("Searching for G-quadruplex sequence motifs in your file");

    let forward_pattern = r"(?i)G{3,}[ATCGN]{1,7}G{3,}[ATCGN]{1,7}G{3,}[ATCGN]{1,7}G{3,}";
    let reverse_pattern = r"(?i)C{3,}[ATCGN]{1,7}C{3,}[ATCGN]{1,7}C{3,}[ATCGN]{1,7}C{3,}";
    let g_pat = Regex::new(forward_pattern).unwrap();
    let c_pat = Regex::new(reverse_pattern).unwrap();

    let _ = stream_fastq(path, &g_pat, &c_pat);
}

fn stream_fastq(filepath: &str, g_re: &Regex, c_re: &Regex) -> io::Result<()> {
    let is_gzipped:i32;
    let filepath = Path::new(filepath);

    if filepath.extension().unwrap() == "gz" {
        is_gzipped = 1;
    } else {
        is_gzipped = 0;
    }

    let _display = filepath.display();

    let file = match File::open(&filepath) {
        Err(why) => panic!("couldn't open the reads file! {why}"),
        Ok(file) => file,
    };

    let reader: Box<dyn BufRead> = if is_gzipped == 1 {
        Box::new(BufReader::new(MultiGzDecoder::new(file)))
    } else {
        Box::new(BufReader::new(file))
    };

    let mut lines = reader.lines();
    while let Some(Ok(h_line)) = lines.next() {
        let header = h_line; // Line 1: Header
        if let Some(Ok(seq_line)) = lines.next() {
            find_pg(&seq_line, &g_re, &header, "+");
            find_pg(&seq_line, &c_re, &header, "-");
        }
        let _ = lines.next();
        let _ = lines.next();
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
        let start = mat.start() as i32;
        let end = mat.end() as i32;
        let length = end - start;
        println!("{}\t{}\t{}\t{}\t{}\t{}", chrom, start, end, "G4", length, strand);
    }
}
