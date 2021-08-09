// https://flownet.com/ron/papers/lisp-java/instructions.html

use std::collections::HashMap;
use std::collections::VecDeque;
use std::env::args;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};

type Dictionary = HashMap<Vec<u8>, Vec<String>>;

// File to line iterator
fn read_lines(file: &str) -> impl Iterator<Item = String> {
    let f = File::open(file).unwrap_or_else(|_| panic!("Failed to open input file: {}", &file));
    BufReader::new(f).lines().map(Result::unwrap)
}

fn load_dictionary(path: &str) -> Dictionary {
    let mut result = HashMap::with_capacity(100000);

    for w in read_lines(path) {
        let key = word_key(&w);

        let entry = result.entry(key).or_insert_with(|| Vec::with_capacity(2));
        entry.push(w.to_string());
    }

    result
}

// Map our string of letters into a Vec of their corresponding numbers.
fn word_key(s: &str) -> Vec<u8> {
    s.chars()
        .map(|ch| match ch {
            'e' | 'E' => Some(0),
            'j' | 'n' | 'q' | 'J' | 'N' | 'Q' => Some(1),
            'r' | 'w' | 'x' | 'R' | 'W' | 'X' => Some(2),
            'd' | 's' | 'y' | 'D' | 'S' | 'Y' => Some(3),
            'f' | 't' | 'F' | 'T' => Some(4),
            'a' | 'm' | 'A' | 'M' => Some(5),
            'c' | 'i' | 'v' | 'C' | 'I' | 'V' => Some(6),
            'b' | 'k' | 'u' | 'B' | 'K' | 'U' => Some(7),
            'l' | 'o' | 'p' | 'L' | 'O' | 'P' => Some(8),
            'g' | 'h' | 'z' | 'G' | 'H' | 'Z' => Some(9),
            _ => None,
        })
        .flatten()
        .collect()
}

#[derive(Debug)]
struct Candidate {
    input_position: usize,
    word_end_positions: u64,
    digit_literal_positions: u64,
    last_was_literal: bool,
}

struct ExpansionNode<'a> {
    words: &'a[String],
    next_idx: usize,
    just_wrapped: bool,
}

impl<'a> ExpansionNode<'a> {
    pub fn value(&self) -> &str {
        &self.words[self.next_idx]
    }

    pub fn increment(&mut self) -> bool {
        self.next_idx += 1;

        // Did we wrap?
        if self.next_idx == self.words.len() {
            self.next_idx = 0;
            self.just_wrapped = true;
            true
        } else {
            self.just_wrapped = false;
            false
        }
    }
}

// Print expansions by generating every possible combination of words in each of
// our positions.  Works much like incrementing a number: start from the right
// and increment each digit.  If it overflows, keep moving left and incrementing
// until you find a number that doesn't.
fn print_expansions(writer: &mut dyn Write, number: &str, words: &[&[String]]) {
    let mut nodes: Vec<ExpansionNode> = words
        .into_iter()
        .map(|w| ExpansionNode {
            words: w,
            next_idx: 0,
            just_wrapped: false,
        })
        .collect();

    loop {
        if nodes[0].just_wrapped {
            break;
        }

        writer.write_all(number.as_bytes()).expect("IO error");
        writer.write_all(b":").expect("IO error");

        for n in &nodes {
            writer.write_all(b" ").expect("IO error");
            writer.write_all(n.value().as_bytes()).expect("IO error");
        }

        writer.write_all(b"\n").expect("IO error");

        for idx in (0..nodes.len()).rev() {
            let wrapped = nodes[idx].increment();

            if !wrapped {
                // Increment from right to left until something doesn't wrap
                break;
            }
        }
    }
}

struct MatchGenerator<'a> {
    number_digits: &'a [u8],
    dictionary: &'a Dictionary,
    candidates: VecDeque<Candidate>,
}

impl<'a> MatchGenerator<'a> {
    fn new(number_digits: &'a [u8], dictionary: &'a Dictionary) -> MatchGenerator<'a> {
        let mut result = MatchGenerator {
            number_digits,
            dictionary,
            candidates: VecDeque::new(),
        };

        // Each candidate represents a portion of the input digits that we haven't
        // finished exploring.
        result.candidates.push_back(Candidate {
            input_position: 0,
            word_end_positions: 0,
            digit_literal_positions: 0,
            last_was_literal: false,
        });

        result
    }
}

impl<'a> Iterator for MatchGenerator<'a> {
    type Item = Candidate;

    fn next(&mut self) -> Option<Candidate> {
        while let Some(candidate) = self.candidates.pop_back() {
            let start_idx = candidate.input_position;

            let mut found_word = false;

            // Scan the rest of the input for this candidate.  As we find words in our
            // dictionary, record their end positions and add new Candidates to our search
            // list.
            for idx in (candidate.input_position + 1)..=self.number_digits.len() {
                let candidate_key = &self.number_digits[start_idx..idx];

                if let Some(_words) = self.dictionary.get(candidate_key) {
                    // matched a word
                    found_word = true;

                    let positions = candidate.word_end_positions | (1 << idx);

                    let next_candidate = Candidate {
                        input_position: idx,
                        word_end_positions: positions,
                        last_was_literal: false,
                        ..candidate
                    };

                    if idx == self.number_digits.len() {
                        // A complete match!
                        return Some(next_candidate);
                    } else {
                        // Partial match... keep looking from here
                        self.candidates.push_back(next_candidate);
                    }
                }
            }

            // If we didn't find a word at `input_position`, we can add a digit here if we
            // didn't do that for the last position.
            if !found_word && !candidate.last_was_literal {
                // We have the option of inserting a literal digit
                let digit_literal_positions = candidate.digit_literal_positions | (1 << (candidate.input_position + 1));

                let next_candidate = Candidate {
                    input_position: candidate.input_position + 1,
                    digit_literal_positions: digit_literal_positions,
                    last_was_literal: true,
                    ..candidate
                };

                if (candidate.input_position + 1) == self.number_digits.len() {
                    // A complete match!
                    return Some(next_candidate);
                } else {
                    // Partial match... keep looking from here
                    self.candidates.push_back(next_candidate);
                }
            }
        }

        None
    }
}


fn main() {
    let mut args: Vec<_> = args().skip(1).collect();
    let words_file: String = if !args.is_empty() {
        args.remove(0)
    } else {
        panic!("need a words file")
    };
    let input_file: String = if !args.is_empty() {
        args.remove(0)
    } else {
        panic!("need a numbers file")
    };

    let dictionary = load_dictionary(&words_file);
    let mut words: Vec<&[String]> = Vec::new();

    let digit_strings: Vec<Vec<String>> = (0..9).map(|digit| vec!(digit.to_string())).collect();

    for number in read_lines(&input_file) {
        let mut number_digits: Vec<u8> = Vec::with_capacity(32);
        number_digits.extend(
            number
                .chars()
                .filter(char::is_ascii_digit)
                .map(|ch| ch.to_digit(10).unwrap() as u8),
        );

        if number_digits.is_empty() {
            continue;
        }

        let stdout = io::stdout();
        let mut writer = BufWriter::new(stdout.lock());

        for m in MatchGenerator::new(&number_digits, &dictionary) {
            words.clear();

            let mut last_idx = 0;
            for idx in 0..=number_digits.len() {
                if (m.word_end_positions & (1 << idx)) != 0 {
                    let key = number_digits[last_idx..idx].to_vec();
                    words.push(dictionary.get(&key).unwrap());
                    last_idx = idx;
                } else if (m.digit_literal_positions & (1 << idx)) != 0 {
                    words.push(&digit_strings[number_digits[idx as usize] as usize]);
                    last_idx += 1;
                }
            }

            print_expansions(&mut writer, &number, &words);
        }
    }
}
