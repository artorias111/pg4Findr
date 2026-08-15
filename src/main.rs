// main.rs
mod cli;
mod g4;
mod seq;

use clap::Parser;

use cli::Args;
use g4::{Patterns, find_in_record};

fn main() {
    let args = Args::parse();
    let pats = Patterns::new();

    for file_path in &args.reads {
        let records = match seq::from_path(file_path) {
            Ok(records) => records,
            Err(e) => {
                eprintln!("Error processing the file {}: {}", file_path, e);
                continue;
            }
        };

        for item in records {
            let record = match item {
                Ok(record) => record,
                Err(e) => {
                    eprintln!("Error processing {}: {}", file_path, e);
                    break;
                }
            };

            for m in find_in_record(&record.id, &record.seq, &pats) {
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
    }
}
