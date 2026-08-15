# pg4findr
![CI](https://github.com/artorias111/pg4findr/actions/workflows/ci.yml/badge.svg)


Find G-quadruplex motifs in sequencing reads/genome assemblies. Input is expected to be a fastq (optionally can be gzipped) or a fasta (if you're working with a genome assembly). Output is a bed file with the columns sequence_id, start, end, G4, length, strand. Default output is to stdout. See https://github.com/samtools/hts-specs/blob/master/BEDv1.pdf for more information on the BED file format.


The sequences are found via a regular expression explained in  https://doi.org/10.1093/nar/gki609 with Rust Regex's `find_iter()` (https://docs.rs/regex/latest/regex/struct.Regex.html#method.find_iter) to avoid overlaps and repeating counts. 

### Usage
```shell
# The default output is to stdout, you can redirect it to a file. the output is in a standard bed file format.

# quick run with cargo
cargo run -- --reads /path/to/reads.fastq(.gz) > g4_motifs.bed

# Works with multiple read files, and pipe the output to gzip/pigz before saving
cargo run -- --reads ../*.fastq.gz  | pigz > g4_motifs.bed.gz 

# Run on Polar2020
/data2/work/local/pg4Findr/pg4Findr --reads /path/to/read/or/assembly.fa | pigz > g4_motifs.bed.gz
```
