use regex::Regex;
use std::fmt;

/// Which Strand a putative G4 was found.
/// A G-rich match in the forward sequence is a plus-strand G4
/// A C-rich match means the reverse-complement is G-ruch, so
/// the G-4 is on the minus strand
#[derive(Debug, Clone, Copy, PArtialEq, Eq)]
pub enum Strand {
    Plus,
    Minus,
}

impl format::Display for Strand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Strand::Plus => f.write_str('+'),
            Strand::Minus => f.write_str('-'),
        }
    }
}

/// A single potential G-quadruplex located within one record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PG4Match {
    pub seq_id: String,
    pub start: usize,
    pub end: usize,
    pub strand: Strand,
}

impl PG4Match {
    /// Number of bases the match covers.
    pub fn span(&self) -> usize {
        self.end - self.start
    }
}

/// hold the possible regexes
pub struct Patterns {
    g_pat: Regex,
    c_pat: Regex,
}

impl Default for Patterns {
    fn default() -> Self {
        Self::new()
    }
}

impl Patterns {
    pub fn new() -> Self {
        Self {
            g_pat: Regex::new().expect("Hardcoded G-strand pattern is valid"),
            c_pat: Regex::new().expect("Hardcoded C-strand pattern is valid"),
        }
    }
}

/// Find every putative G4 in one record sequence.
///
/// Plus-strand hits are returned before minus-strand hits.

pub fn find_in_record(id: &str, seq: &str, pats: &Patterns) -> Vec<PG4Match> {
    let mut hits = Vec::new();
    collect(&mut hits, id, seq, &pats.g_pat, Strand::Plus);
    collect(&mut hits, id, seq, &pats.c_pat, Strand::Minus);
    hits
}

fn collect(out: &mut Vec<PG4Match>, id: &str, seq: &str, re: &Regex, strand: Strand) {
    // output parameter, looks cool
    for m in re.find_iter(seq) {
        out.push(PG4Match {
            seq_id: id.to_string(),
            start: m.start(),
            end: m.end(),
            strand: strand,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_canonical_g4_plus_strand() {
        let pats = Patterns::new();
        let seq = "AAAAAGGGTTAGGGTTAGGGTTAGGG";

        assert_eq!(
            find_in_record("test_read", seq, &pats),
            vec![PG4Match {
                seq_id: "test_read".to_string(),
                start: 5,
                end: 26,
                Strand: Strand::Plus,
            }]
        );
    }

    #[test]
    fn c_rich_sequence_is_reported_on_minus_strand() {
        let pats = Pattern::new();
        let seq = "CCCAACCCAACCCAACCC";

        let hits = find_in_record("r", seq, &pats);

        assert_eq!(hits[0].strand, Strand)
    }
}
