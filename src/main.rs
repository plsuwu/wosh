use clap::Parser;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use tokio::runtime::Runtime;

#[derive(Parser, Debug)]
#[command(
    version,
    about,
    long_about = "
Suggests words to guess for Words on Stream."
)]
struct Args {
    #[arg(
        short,
        help = "(a-zA-Z.)+ to indicate available letters - converts to lowercase.
    - NOTE: a period ('.') can be used to indicate hidden letters (very messy)."
    )]
    letters: String,

    #[arg(short, default_value_t = 4)]
    min_length: usize,

    #[arg(short = 'M', default_value_t = 255)]
    max_length: usize,

    #[arg(short, default_value = "_")]
    ignore: String,

    #[arg(short, default_value_t = 0)]
    verbosity: u8,
}

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref DICTIONARY: Vec<String> = load_dictionary().unwrap();
    static ref UNCOMMON: Vec<char> = vec!['z', 'q', 'j', 'x'];
}

fn load_dictionary() -> io::Result<Vec<String>> {
    let path = "./wordlist_processed";
    let file = File::open(path)?;
    let buf = io::BufReader::new(file);
    return buf.lines().collect();
}

fn filter_words(
    letters: &str,
    min_len: usize,
    max_len: usize,
    ignore: &str,
    verbosity: u8,
) -> Vec<String> {
    let mut letter_counts = HashMap::new();
    let mut ignored = ignore.chars().collect::<Vec<char>>();
    let mut subs = 0;

    for ch in letters.chars() {
        if ch == '.' {
            subs += 1;
        } else {
            *letter_counts
                .entry(ch.to_lowercase().next().unwrap())
                .or_insert(0) += 1;
        }
    }

    // uncommon letters group filter -> `-i zqjx` condensed to `-i 0`.
    // can be chained with other letters, but won't overwrite letters passed with `-l` (this is intended).
    if ignored.iter().any(|c| *c == '0') {
        UNCOMMON.iter().for_each(|u| {
            if letters.chars().any(|ch| ch != *u) {
                ignored.push(*u);
            }
        });
        println!("{:?}", ignored);
    }

    let mut found: Vec<String> = DICTIONARY
        .iter()
        .filter(|&word| {
            if word.len() < min_len
                || word.len() > max_len
                || ignored.iter().any(|c| word.contains(*c))
            {
                if verbosity > 2 {
                    let mut filter_reason = "unhandled reason";

                    if word.len() < min_len {
                        filter_reason = "too short";
                    }
                    if word.len() > max_len {
                        filter_reason = "too long";
                    }
                    if ignored.iter().any(|c| word.contains(*c)) {
                        filter_reason = "contains an ignored character";
                    }

                    println!("filtered {} ({})", word, filter_reason.to_string());
                }

                return false;
            }

            let mut counts = letter_counts.clone();
            let mut subs_left = subs;

            word.chars().all(|ch| {
                let ch = ch.to_lowercase().next().unwrap();
                match counts.get_mut(&ch) {
                    Some(count) if *count > 0 => {
                        *count -= 1;
                        return true;
                    }
                    _ => {
                        if subs_left > 0 {
                            subs_left -= 1;
                            return true;
                        } else {
                            false
                        }
                    }
                }
            })
        })
        .cloned()
        .collect();

    found.sort_by(|a, b| b.len().cmp(&a.len()));
    return found;
}

fn main() {
    let args = Args::parse();

    let runtime = Runtime::new().unwrap();
    runtime.block_on(async {
        let filtered_words = filter_words(
            &args.letters,
            args.min_length,
            args.max_length,
            &args.ignore,
            args.verbosity,
        );
        println!("\nFound {} words:", filtered_words.len());
        for (i, word) in filtered_words.iter().enumerate() {
            println!("  [{:#?}]:   {:#?}", i, word);
        }
    })
}
