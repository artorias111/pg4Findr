// cli.rs

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version)]
pub struct Args {
    #[arg(short, long, num_args = 1..)]
    pub reads: Vec<String>,
}
