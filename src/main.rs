use clap::Parser;
use csv::{ReaderBuilder, Trim};
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::sync::Arc;
use tokio::task;

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

    #[arg(
        short,
        long,
        default_value = "/home/please/Documents/Repositories/plsuwu/wosh/wos-sorted.csv"
    )]
    wordlist: String,

    #[arg(short, long, default_value_t = 55)]
    spaces: usize,

    #[arg(short, long, default_value_t = 4)]
    min_length: usize,

    #[arg(short = 'n', long, default_value = "_")]
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

fn filtered(letters: &str, ignored: &str, board_size: usize, wordlist: Vec<Record>) -> Vec<Record> {
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
            if board_size < rec.spaces as usize {
                return false;
            }
            if rec.letters.contains(ignored) {
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

#[tokio::main]
async fn main() {
    let args = Arguments::parse();
    let wordlist = Arc::new(read_wordlist(&args.wordlist).unwrap());
    let chunk_size = 25;
    let chunks = wordlist.chunks(chunk_size);

    let futures = chunks.map(|chunk| {
        let wordlist_chunk = chunk.to_vec();
        let args_letters = args.letters.clone();
        let args_ignore = args.ignore.clone();
        let args_spaces = args.spaces;

        task::spawn_blocking(move || {
            filtered(&args_letters, &args_ignore, args_spaces, wordlist_chunk)
        })
    });

    let results = futures::future::join_all(futures).await;

    for res in results {
        let filtered_words = res.unwrap();
        for rec in filtered_words {
            let subs = rec
                .letters
                .chars()
                .filter(|c| !args.letters.contains(*c))
                .map(|c| c.to_string())
                .collect::<Vec<_>>();

            let fake = args
                .letters
                .chars()
                .filter(|c| !rec.letters.contains(*c) && *c != '.')
                .map(|c| c.to_string())
                .collect::<Vec<_>>();

            println!("");
            if !subs.is_empty() {
                println!("substituted '?' for '{}'", subs.join(", "));
            }

            if !fake.is_empty() {
                println!("fake(s) for potential board: '{}'", fake.join(", "));
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
        }
    }

    if !args.ignore.contains("_") {
        println!("\n[--ignore]: this run ignored '{}'.", args.ignore);
    }
}
