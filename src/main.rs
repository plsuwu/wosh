use clap::Parser;
use csv::{ReaderBuilder, Trim};
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::task::spawn_blocking;

#[derive(Debug, Deserialize, Clone)]
struct Record {
    // wordlen,longest,letters,spaces,wordlist
    wordlen: u32,
    longest: String,
    letters: String,
    spaces: u32,
    #[serde(rename(deserialize = "wordlist"))]
    words: Vec<String>,
}

#[derive(Parser, Debug)]
#[command(version, about)]
struct Arguments {
    #[arg(short, long)]
    letters: String,

    #[arg(short, long, default_value = "./wos-sorted.csv")]
    wordlist: String,

    #[arg(short = 'H', long, default_value_t = false)]
    contains_hidden: bool,

    #[arg(short, long, default_value_t = 55)]
    spaces: usize,

    #[arg(short, long, default_value_t = 4)]
    min_length: usize,

    #[arg(short, long, default_value = "_")]
    ignore: String,
}

fn read_wordlist(path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .trim(Trim::All)
        .flexible(true)
        .from_reader(file);
    let mut wordlists = vec![];

    for res in rdr.deserialize() {
        let wordlist: Record = res?;
        wordlists.push(wordlist);
    }

    return Ok(wordlists);
}

fn filtered(
    letters: &str,
    _ignored: &str,
    hidden: bool,
    board_size: usize,
    wordlist: Vec<Record>,
) -> Vec<Record> {
    let mut letter_counts = HashMap::new();
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

    let found: Vec<Record> = wordlist
        .into_iter()
        .filter(|rec| {
            if !hidden && letters.len() + subs <= rec.wordlen as usize {
                return false;
            }

            if board_size < rec.spaces as usize {
                return false;
            }

            let mut counts = letter_counts.clone();
            let mut remaining_subs = subs;

            rec.letters.chars().all(|ch| {
                let ch = ch.to_lowercase().next().unwrap();
                match counts.get_mut(&ch) {
                    Some(count) if *count > 0 => {
                        *count -= 1;
                        return true;
                    }
                    _ => {
                        if remaining_subs > 0 {
                            remaining_subs -= 1;
                            return true;
                        } else {
                            return false;
                        }
                    }
                }
            })
        })
        .collect();

    return found;
}

fn main() {
    let args = Arguments::parse();
    if !&args.contains_hidden {
        println!(
            "\n
            --------------
            [NOTE]: Hidden letters are UNSET (set with `-H`).
            Letters to be compared against wordlist as final known set.
            --------------
            \n"
        );
    }

    // im not actually sure this runs concurrently :(
    let wordlist = Arc::new(read_wordlist(&args.wordlist).unwrap());
    let runtime = Runtime::new().unwrap();

    runtime.block_on(async move {
        spawn_blocking(move || {
            let args_letters = args.letters.clone();
            let args_ignore = args.ignore.clone();
            let args_hidden = args.contains_hidden.clone();
            let args_spaces = args.spaces.clone();

            let filtered_wl = filtered(
                &args_letters.to_owned(),
                &args_ignore.to_owned(),
                args_hidden,
                args_spaces,
                wordlist.to_vec(),
            );

            let _: Vec<_> = filtered_wl
                .iter()
                .map(|rec| {
                    println!("");
                    let substituted = rec
                        .letters
                        .chars()
                        .filter(|c| !args_letters.contains(*c))
                        .map(|c| c.to_string())
                        .collect::<Vec<_>>()
                        .join(", ");
                    let fake_letter = args_letters
                        .chars()
                        .filter(|c| !rec.letters.contains(*c) && *c != '.')
                        .map(|c| c.to_string())
                        .collect::<Vec<_>>()
                        .join(",");

                    if !substituted.is_empty() {
                        println!("substituted '?' for '{}'", substituted);
                    }

                    if !fake_letter.is_empty() {
                        println!("fake(s) for this board: '{}'", fake_letter);
                    }
                    println!("board's longest word: '{}'", rec.longest);
                    let mut words: Vec<&str> = rec.words.iter().map(|s| &**s).collect();
                    let words = words
                        .iter_mut()
                        .map(|s| s.split_whitespace().collect::<Vec<_>>())
                        .collect::<Vec<_>>();

                    for (i, w) in words[0].iter().rev().enumerate() {
                        println!("{}: {}", i, w);
                    }
                })
                .collect();
        });
    });
}
