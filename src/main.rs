use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use flate2::read::MultiGzDecoder;

fn main() {
    println!("Hello, world!");

    let pattern = r"(?i)G{3,}[ATCGN]{1,7}G{3,}[ATCGN]{1,7}G{3,}[ATCGN]{1,7}G{3,}";
    let re = Regex::new(pattern).unwrap();
}


// read a fastq file
//

fn stream_fastq(filepath: &str, threshold: f32, re: &Regex) -> io::Result<()> {
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


    let mut is_header_assigned = 0;
    let mut header = String::new();

    for line in reader.lines() {
        let fastq_line = match line {
            Err(_) => panic!("Empty fastq file, please try again"),
            Ok(file) => file,
        };
        if fastq_line.starts_with('@') {
            header = fastq_line;
            is_header_assigned = 1;
            continue;
        }

        if is_header_assigned == 1 {
            find_pg(&fastq_line, &re, &header);
            is_header_assigned = 0;
        }

    }

    Ok(())
}





fn find_pg(seq: &str, re: &Regex, header: &str) {
    let chrom = header.trim_start_matches('@').split_whitespace().next().unwrap_or("unknown");
    
    for mat in re.find_iter(seq) {
        let start = mat.start() as i32;
        let end = mat.end() as i32;
        let length = end - start;
        println!("{}\t{}\t{}\t{}\t{}", chrom, start, end, "G4", length);
    }
}
