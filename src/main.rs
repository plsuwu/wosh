use clap::Parser;
use csv::{ReaderBuilder, Trim};
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Clone)]
struct Record {
    letters: String,
    longest: String,
    wordlen: u32,
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

fn filtered(letters: &str, _ignored: &str, hidden: bool, wordlist: Vec<Record>) -> Vec<Record> {
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
            if !hidden && letters.len() != rec.wordlen as usize {
                return false;
            }

            if letters.len() < rec.wordlen as usize {
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
        println!("\n\n--------------\n[NOTE]: Hidden letters are UNSET (set with `-H`).\nLetters to be compared against wordlist as final known set.\n--------------\n");
    }

    let wordlist = read_wordlist(&args.wordlist).unwrap();
    let filtered_wl = filtered(&args.letters, &args.ignore, args.contains_hidden, wordlist);

    let _: Vec<_> = filtered_wl
        .iter()
        .clone()
        .map(|rec| {
            println!("\nFOR LONGEST WORD: {}\n", rec.longest);
            let mut words: Vec<&str> = rec.words.iter().map(|s| &**s).collect();
            let words = words
                .iter_mut()
                .map(|s| s.split_whitespace().collect::<Vec<_>>())
                .collect::<Vec<_>>();
            for (i, w) in words[0].iter().enumerate() {
                println!("{}: {}", i, w);
            }
        })
        .collect();

    // println!("{:?}, {:?}, {:#?}", args.letters, longest_word, wordset);
}
