use clap::Parser;
use csv::{ReaderBuilder, Trim};
use fetch::get_list;
use lazy_static::lazy_static;
use serde::Deserialize;
use tokio::sync::Mutex;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{stdin, BufRead, BufReader, ErrorKind, Read};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::task;

mod fetch;

lazy_static!(
    #[derive(Debug)]
    pub static ref DATA_LOCAL: PathBuf = Path::new(&dirs::data_local_dir().unwrap()).to_path_buf();
    pub static ref WORDLIST_DEFAULT: PathBuf = Path::new(&DATA_LOCAL.join("wosh/wordlist")).to_path_buf();
    pub static ref SUBLIST_DEFAULT: PathBuf = Path::new(&DATA_LOCAL.join("wosh/sublist")).to_path_buf();
    pub static ref WORDLIST_URL_GIT: String = "https://raw.githubusercontent.com/plsuwu/wosh/master/wordlist".to_owned();
    pub static ref SUBLIST_URL_GIT: String = "https://raw.githubusercontent.com/plsuwu/wosh/master/sublist".to_owned();
);

#[derive(Debug, Deserialize, Clone)]
struct Record {
    // wordlen,longest,letters,spaces,wordlist
    #[allow(dead_code)]
    wordlen: u32, // this isn't used but i dont want to remove it from the csv
    longest: String,
    letters: String,
    spaces: u32,
    #[serde(rename(deserialize = "wordlist"))]
    words: Vec<String>,
}

#[derive(Parser, Debug)]
#[command(version, author, about)]
struct Arguments {
    #[arg(short, long)]
    letters: String,

    #[arg(short = 'I', long)]
    interactive: bool,

    #[arg(short = 'i', long, default_value = "_")]
    ignore: String,

    #[arg(long, default_value_t = 0)]
    spaces: usize,

    #[arg(short, long)]
    wordlist: Option<String>,

    #[arg(short, long)]
    sublist: Option<String>,

    #[arg(
        short = 't',
        long,
        default_value_t = 15,
        help = "split the wordlist into specified number of chunks to be processed in their own threads."
    )]
    threads: usize,
}

async fn read_wordlist(path: &PathBuf) -> Result<Vec<Record>, Box<dyn Error>> {
    let file_res = File::open(path);
    let file = match file_res {
        Ok(file) => file,
        Err(err) => match err.kind() {
            ErrorKind::NotFound => match get_list(&path, "word").await {
                Ok(fc) => {
                    println!("\n   => Fetch ok, returning.");
                    fc
                }
                Err(e) => panic!("[ERR]: Unable to initialize a wordlist (no wordlist supplied, default wordlist unavailable): \n{:#?}", e),
            },
            unhandled => {
                panic!(
                    "[ERR]: Unhandled error while opening the wordlist: {:#?}",
                    unhandled
                );
            }
        },
    };

    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .trim(Trim::All)
        .flexible(true)
        .from_reader(file);
    let mut wordlists = vec![];

    for res in reader.deserialize() {
        let wordlist: Record = res?;
        wordlists.push(wordlist);
    }

    return Ok(wordlists);
}

async fn read_sublist(path: &PathBuf) -> Result<Vec<String>, Box<dyn Error>> {
    let file_res = File::open(path);
    let file = match file_res {
        Ok(file) => file,
        Err(err) => match err.kind() {
            ErrorKind::NotFound => match get_list(&path, "sub").await {
                Ok(fc) => {
                    println!("\n    => Fetch ok, returning.");
                    fc
                }
                Err(e) => panic!("[ERR]: Unable to initialize a sublist (no sublist supplied, default sublist unavailable): {:#?}", e),
            },
            unhandled => {
                panic!(
                    "[ERR]: Unhandled error while opening the sublist: {:#?}",
                    unhandled
                    );
            }
        },
    };

    let reader = BufReader::new(file);
    let mut sublist = Vec::new();

    for line in reader.lines() {
        sublist.push(line.unwrap());
    }

    return Ok(sublist);
}

fn filtered(
    wordlist: Arc<Vec<Record>>,
    letters: &str,
    ignored: &str,
    board_size: usize,
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

    let found = wordlist
        .iter()
        .filter(|rec| {
            if board_size > 0 && board_size < rec.spaces as usize || rec.letters.contains(ignored) {
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
        }).cloned()
        .collect::<Vec<_>>();

    return found;
}

async fn suggest_unknown(unknown: &str, letters: String, sub_path: &PathBuf) -> Vec<String> {
    let sublist = read_sublist(sub_path).await.unwrap();

    let mut letter_counts = HashMap::new();

    for ch in letters.chars() {
        *letter_counts
            .entry(ch.to_lowercase().next().unwrap())
            .or_insert(0) += 1;
    }

    let found = sublist
        .iter()
        .filter(|word| {
            if word.len() != unknown.len() {
                return false;
            }

            let mut counts = letter_counts.clone();
            word.chars().all(|ch| {
                let ch = ch.to_lowercase().next().unwrap();
                match counts.get_mut(&ch) {
                    Some(count) if *count > 0 => {
                        *count -= 1;
                        return true;
                    }
                    _ => {
                        return false;
                    }
                }
            })
        })
        .cloned()
        .collect::<Vec<_>>();

    return found;
}

async fn process(list_path: &PathBuf, sub_path: &PathBuf, chunk_size: usize, letters: &str, ignore: &str, spaces: usize) {
    println!("\n[-----------------------------]\n"); // result separator
                                                     //
    let wordlist = Mutex::new(Arc::new(read_wordlist(list_path).await.unwrap()));
    let chunks = wordlist.lock().await
        .chunks(chunk_size)
        .map(|chunk| Arc::new(chunk.to_vec()))
        .collect::<Vec<_>>();

    let futures = chunks.into_iter().map(|chunk| {
        let args_letters = letters.to_owned();
        let args_ignore = ignore.to_owned();
        let args_spaces = spaces.clone();

        task::spawn_blocking(move || {
            filtered(chunk, &args_letters, &args_ignore, args_spaces) // grr
        })
    });

    let joined = futures::future::join_all(futures).await;

    let results = joined
        .iter()
        .map(|r| r.as_ref().unwrap().to_owned())
        .filter(|r| !r.is_empty())
        .flatten()
        .collect::<Vec<_>>();

    for (i, rec) in results.iter().enumerate() {
        let subs = rec
            .letters
            .chars()
            .filter(|c| !letters.contains(*c))
            .map(|c| c.to_string())
            .collect::<Vec<_>>();

        let fake = letters
            .chars()
            .filter(|c| !rec.letters.contains(*c) && *c != '.')
            .map(|c| c.to_string())
            .collect::<Vec<_>>();

        let mut words: Vec<&str> = rec.words.iter().map(|s| &**s).collect();
        let words = words
            .iter_mut()
            .map(|s| {
                s.split_whitespace().collect::<Vec<_>>()
            }).rev()
            .collect::<Vec<_>>();

        let mut run_suggest = HashMap::new();
        for (j, w) in words[0].iter().enumerate() {
            print!("[{:02}]: {} ", words[0].len() - j, w);
            if w.contains('?') {
                if !run_suggest.contains_key(&w.len()) {
                    let suggested = suggest_unknown(w, rec.letters.clone(), sub_path).await;
                    println!("=> [",);
                    for (k, word) in suggested.iter().rev().enumerate() {
                        println!("    [{:02}| {} ],", suggested.len() - k, word);
                    }
                    run_suggest.insert(w.len(), words[0].len() - j);
                } else {
                    let (_, val) = run_suggest.get_key_value(&w.len()).unwrap();
                    print!("(as per [^{}])", val);
                }
            }
            println!("");
        }

        println!("\n=> [^]: '{}'", rec.longest);
        if !subs.is_empty() {
            println!("=> [h]: '{}'", subs.join(", "));
        }

        if !fake.is_empty() {
            println!("=> [x]: '{}'", fake.join(", "));
        }

        println!("\n[-----------------------------]");
        println!(
            "[  ^^^ [RESULT {:03}/{:03}] ^^^   ]",
            &results.len() - i,
            &results.len()
        );
        println!("[-----------------------------]\n");
    }

    if !ignore.contains("_") {
        let char_list = ignore.chars().map(|c| c.to_string()).collect::<Vec<_>>();
        println!(
            "\n[--ignore]: this run ignored '{}'.",
            &char_list.join(", ")
        );
    }
}

#[tokio::main]
async fn main() {
    let args = Arguments::parse();

    // this could be an iterator or use more pattern matching
    // but i dont want to refactor it as of now
    let sublist_path = match args.sublist {
        Some(ref _path) => {
            let path_from_args: PathBuf = args.sublist.unwrap().into();
            match path_from_args.try_exists() {
                Ok(_) => path_from_args,
                Err(e) => {
                    panic!("[ERR]: Could not find a file from provided path: {:#?}", e);
                }
            }
        }
        None => {
            let default_initialized: PathBuf = SUBLIST_DEFAULT.to_owned();
            match default_initialized.try_exists() {
                Ok(_) => SUBLIST_DEFAULT.to_path_buf(),
                Err(e) => {
                    panic!("[ERR]: Unable to create a default sublist: {:#?}", e);
                }
            }
        }
    };

    let wordlist_path = match args.wordlist {
        Some(ref _path) => {
            let path_from_args: PathBuf = args.wordlist.unwrap().into();
            match path_from_args.try_exists() {
                Ok(_) => path_from_args,
                Err(e) => {
                    panic!("[ERR]: Could not find a file from provided path: {:#?}", e);
                }
            }
        }
        None => {
            let default_initialized: PathBuf = WORDLIST_DEFAULT.to_owned();
            match default_initialized.try_exists() {
                Ok(_) => WORDLIST_DEFAULT.to_path_buf(),
                Err(e) => {
                    panic!("[ERR]: Unable to create a default wordlist: {:#?}", e);
                }
            }
        }
    };

    println!("");

    // not working + cant really be bothered
    if args.interactive {
        println!("[--interactive]: running interactively ('\\q' to exit):");
        loop {
            let mut input = String::new();
            stdin().read_to_string(&mut input).unwrap();

            if input == "\\q" {
                break;
            }

            println!("{:?}", input);
        }
    } else {
        process(
            &wordlist_path,
            &sublist_path,
            args.threads,
            &args.letters,
            &args.ignore,
            args.spaces,
        )
        .await;
    }
}
