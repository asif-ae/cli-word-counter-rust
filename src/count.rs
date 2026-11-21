use std::io::{self, Read};

/// Standard buffer size for streaming I/O (64 KB).
/// This balances syscall frequency and CPU cache friendliness.
const BUFFER_SIZE: usize = 64 * 1024;

/// Holds the accumulated count statistics.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CountStats {
    pub lines: usize,
    pub words: usize,
    pub bytes: usize,
    pub chars: usize,
}

impl CountStats {
    /// Adds another set of stats to this one (used later for aggregating totals).
    #[allow(dead_code)]
    pub fn add(&mut self, other: &CountStats) {
        self.lines += other.lines;
        self.words += other.words;
        self.bytes += other.bytes;
        self.chars += other.chars;
    }
}

/// Counts lines, words, bytes, and characters from any readable stream in a single pass.
///
/// Accepts any type `R` implementing `std::io::Read` (files, standard input, or byte slices in tests).
pub fn count_stream<R: Read>(reader: R) -> io::Result<CountStats> {
    // 1. Wrap the input in a BufReader with our designated capacity.
    let mut reader = io::BufReader::with_capacity(BUFFER_SIZE, reader);
    let mut stats = CountStats::default();

    // 2. Pre-allocate a 64 KB buffer on the stack (reused for every chunk).
    let mut buffer = [0u8; BUFFER_SIZE];
    
    // Tracks word state across chunks (true if currently scanning non-whitespace).
    let mut in_word = false;
    
    // Holds incomplete trailing UTF-8 bytes split across chunk boundaries.
    let mut leftover = Vec::new();

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break; // EOF (End of File) reached
        }

        // --- BYTE COUNT ---
        stats.bytes += bytes_read;

        let chunk = &buffer[..bytes_read];

        // --- LINE & WORD COUNT ---
        for &b in chunk {
            // Count newlines (Unix \n)
            if b == b'\n' {
                stats.lines += 1;
            }

            // Word state machine
            if (b as char).is_whitespace() {
                in_word = false;
            } else if !in_word {
                in_word = true;
                stats.words += 1;
            }
        }

        // --- CHARACTER (UTF-8) COUNT ---
        // Prepend any leftover bytes from the previous chunk
        let slice = if leftover.is_empty() {
            chunk
        } else {
            leftover.extend_from_slice(chunk);
            &leftover[..]
        };

        match std::str::from_utf8(slice) {
            Ok(valid_str) => {
                stats.chars += valid_str.chars().count();
                leftover.clear();
            }
            Err(e) => {
                let valid_up_to = e.valid_up_to();
                if valid_up_to > 0 {
                    // Safety: from_utf8 guarantees bytes up to valid_up_to are valid UTF-8
                    let valid_str = unsafe { std::str::from_utf8_unchecked(&slice[..valid_up_to]) };
                    stats.chars += valid_str.chars().count();
                }

                if let Some(error_len) = e.error_len() {
                    // Corrupted or invalid UTF-8 byte encountered: count replacement char and continue
                    stats.chars += 1;
                    leftover = slice[valid_up_to + error_len..].to_vec();
                } else {
                    // Incomplete multi-byte sequence split across buffer boundary:
                    // store the unfinished tail to decode with the next chunk
                    leftover = slice[valid_up_to..].to_vec();
                }
            }
        }
    }

    // Flush any leftover bytes at EOF
    if !leftover.is_empty() {
        stats.chars += leftover.len();
    }

    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_empty_stream() {
        let input = b"";
        let stats = count_stream(Cursor::new(input)).unwrap();
        assert_eq!(stats, CountStats { lines: 0, words: 0, bytes: 0, chars: 0 });
    }

    #[test]
    fn test_single_line_no_trailing_newline() {
        let input = b"hello world";
        let stats = count_stream(Cursor::new(input)).unwrap();
        assert_eq!(stats, CountStats { lines: 0, words: 2, bytes: 11, chars: 11 });
    }

    #[test]
    fn test_multiple_lines_and_spaces() {
        let input = b"first line\n\nsecond  line with   spaces\n";
        let stats = count_stream(Cursor::new(input)).unwrap();
        assert_eq!(stats, CountStats { lines: 3, words: 6, bytes: 39, chars: 39 });
    }

    #[test]
    fn test_multibyte_utf8() {
        // "🦀" is 4 bytes, 1 char
        // "ñ" is 2 bytes, 1 char
        let input = "hello 🦀 and ñ\n".as_bytes();
        let stats = count_stream(Cursor::new(input)).unwrap();
        assert_eq!(stats.lines, 1);
        assert_eq!(stats.words, 4);
        assert_eq!(stats.chars, 14);
        assert_eq!(stats.bytes, 18); // 4 + 2 extra bytes for 🦀 and ñ
    }
}