use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone, PartialEq, Eq)]
#[command(
    name = "rwc",
    author,
    version,
    about = "Modern, colored word counter reading files or standard input",
    long_about = "A fast, modern word counter (wc) clone written in Rust.\n\
                  Reads files or standard input and counts lines, words, characters, and byte size."
)]
pub struct Cli {
    /// Print the newline counts
    #[arg(short = 'l', long = "lines")]
    pub lines: bool,

    /// Print the word counts
    #[arg(short = 'w', long = "words")]
    pub words: bool,

    /// Print the byte counts
    #[arg(short = 'c', long = "bytes")]
    pub bytes: bool,

    /// Print the character counts (UTF-8 aware)
    #[arg(short = 'm', long = "chars")]
    pub chars: bool,

    /// Files to process (reads from standard input if empty or "-")
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}

impl Cli {
    /// Resolves flags: if no specific count flags are set, default to lines, words, and bytes.
    pub fn resolve_flags(&mut self) {
        if !self.lines && !self.words && !self.bytes && !self.chars {
            self.lines = true;
            self.words = true;
            self.bytes = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_flags_resolution() {
        let mut cli = Cli {
            lines: false,
            words: false,
            bytes: false,
            chars: false,
            files: vec![],
        };
        cli.resolve_flags();
        assert!(cli.lines, "lines should be true by default");
        assert!(cli.words, "words should be true by default");
        assert!(cli.bytes, "bytes should be true by default");
        assert!(!cli.chars, "chars should remain false by default");
    }

    #[test]
    fn test_custom_flags_preservation() {
        let mut cli = Cli {
            lines: true,
            words: false,
            bytes: false,
            chars: false,
            files: vec![],
        };
        cli.resolve_flags();
        assert!(cli.lines);
        assert!(!cli.words);
        assert!(!cli.bytes);
        assert!(!cli.chars);

        let mut cli_chars = Cli {
            lines: false,
            words: false,
            bytes: false,
            chars: true,
            files: vec![],
        };
        cli_chars.resolve_flags();
        assert!(!cli_chars.lines);
        assert!(!cli_chars.words);
        assert!(!cli_chars.bytes);
        assert!(cli_chars.chars);
    }
}
