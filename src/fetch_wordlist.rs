use std::{
    time::Duration,
    fs::File,
    io::{Error, Write},
    path::PathBuf,
};

use crate::{DATA_LOCAL, WORDLIST_URL_GIT};

async fn fetch(path: PathBuf) -> Result<File, Error> {

    tokio::time::sleep(Duration::from_millis(250)).await;

    let wosh_base = DATA_LOCAL.join("wosh").to_owned();
    if !wosh_base.is_dir() {
        std::fs::create_dir(&wosh_base).unwrap();
    }
    let mut writer = File::create(&path)?;
    let body = reqwest::get(&*WORDLIST_URL_GIT)
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();

    writer.write_all(&body)?;
    let reader = File::open(path).unwrap();

    return Ok(reader);
}

fn confirm() -> std::io::Result<String> {
    loop {
        let mut input = String::new();

        let _ = std::io::stdin().read_line(&mut input);
        if !input.is_empty() {
            return Ok(input);
        }
    }
}

pub async fn get_wordlist(path: &PathBuf) -> Result<File, Error> {

    eprintln!(
        "[ERR]: Unable to find required wordlists (default wordlists expected in directory '{}').",
        DATA_LOCAL.join("wosh").to_owned().to_string_lossy().to_string()
    );
    eprintln!(
        "[ERR]: I can download this automatically from url\n       => '{}'\n[ERR]: Continue? [Y/n]:",
        &WORDLIST_URL_GIT.to_string()
    );
    match confirm() {
        Ok(res) => {
            let confirmed = Vec::from(["".to_string(), "y".to_string(), "yes".to_string()]);
            if !confirmed.contains(&res.trim().to_lowercase().to_owned()) {
                println!("[ERR]: Exiting.\n[NOTE]: Try passing in a wordlist filepath with '[-w | --wordlist < /PATH/TO/WORDLIST.csv > ]'.");
                std::process::exit(1);
            }
        }
        _ => {
            panic!("[ERR]: Unable to read input.");
        }
    };

    let mut dot_counter = 0;
    let future = tokio::task::spawn(fetch(path.to_path_buf()));
    println!(
        "   => Writing to file (path: '{}'):",
        &path.to_string_lossy().to_string()
    );
    print!("     ");
    while !future.is_finished() {
        tokio::time::sleep(Duration::from_millis(250)).await;
        for _ in 0..=(dot_counter * 2) + 2 {
            print!("\u{8}");
            std::io::stdout().flush().unwrap();
        }
        if dot_counter < 3 {
            dot_counter += 1;
            for _ in 0..=dot_counter * 2 {
                print!(" ");
                std::io::stdout().flush().unwrap();
            }
            print!(".");
            std::io::stdout().flush().unwrap();
        }
        if dot_counter == 3 {
            for _ in 0..dot_counter {
                tokio::time::sleep(Duration::from_millis(250)).await;
                print!("\u{8}");
                std::io::stdout().flush().unwrap();
            }

            dot_counter = 0;
        }
    }

    std::io::stdout().flush().unwrap();
    return future.await?;
}
