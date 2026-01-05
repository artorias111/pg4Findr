# pg4Findr

Find G-quadruplex motifs in sequencing reads. Input is expected to be a fastq (optionally can be gzipped). Output is a bed file with the columns sequence_id, start, end, G4, length, strand. Default output is to stdout. See https://genome.ucsc.edu/FAQ/FAQformat.html#format1 for more information on the BED file format.


The sequences are found via a regular expression explained in  https://doi.org/10.1093/nar/gki609

### Usage
```shell
# The default output is to stdout, you can redirect it to a file. the output is in a standard bed file format.

# quick run with cargo
cargo run -- --reads /path/to/reads.fastq(.gz) > g4_motifs.bed

# Works with multiple read files, and pipe the output to gzip/pigz before saving
cargo run -- --reads ../*.fastq.gz  | pigz > g4_motifs.bed.gz 
```
