# Words on Stream "Helper"

> Because I am a litarate

## Installation

Installation is likely easiest with cargo (e.g using [rustup](https://rustup.rs/)) - assuming `git` and `cargo`/`rustc` are installed, it should
just be a matter of cloning this repo and compiling via `cargo install` to dump a binary into your `PATH`:

```bash
# platform agnostic

git clone https://github.com/plsuwu/wosh
cd wosh
cargo install --path .
```

## Usage

> Note: The current version's output intended to be read bottom-to-top.

The idea is that this should be quick and easy to run. Generally, you will only pass `-l`/`--letters` with the board's content:

```bash
# pass letters with '--letters'
# replace any '[?]' with '.'

[please@ruby]:[~] $ wosh -l hanla.ldz
[ RESULT 1 OF 2 ]:

=> [h]: 'e'         # [t] => hidden letter(s) on this board; indicates replacements for '[?]' letters
=> [x]: 'z'         # [x] => fake letter(s); the resolved board was otherwise valid without these letters
=> [^]: 'handle'    # [^] => longest word; the resolved board's longest word

[00]: handle
[01]: laden
[02]: haled
[03]: eland

# ...
```

I stole the 'main' csv wordlist from [a Reddit post](https://www.reddit.com/r/YouTubeGamers/comments/v0khwa/words_on_stream_dictionary/), and therefore isn't a complete dataset; 
some words are replaced with `?`s.

Any time a `?????` is found where there should be a word, the letters and word length are cross-referenced from a substitution list to suggest what this word might be:

```
[ RESULT 2 OF 2 ]:

=> [h]: 'b'
=> [x]: 'z'
=> [^]: 'handball'

[00]: handball
[01]: ballad
[02]: banal
[03]: ????? =>
     [
      [00 | aband ],
      [01 | abdal ],
      [02 | aland ],
      [03 | alban ],
# ...
```

A custom word or substitution list can be passed with `--wordlist` or `--sublist` respectively.

The subsitution list takes a simple line-separated text file, but the wordlist is parsed using the following `Record` struct, and as such, the wordlist needs to be a distinct 
comma-separated values file with the following column headers and data types:

```rust
// wordlen,longest,letters,spaces,wordlist
// number, string, string, number, string

struct Record {
    wordlen: u32,
    longest: String,
    letters: String,
    spaces: u32,
    #[serde(rename(deserialize = "wordlist"))]
    words: Vec<String>,
}
```

There are a handful of other arguments that I won't go into - `wosh` is built on `clap`, so pass `--help` to view a list of available commands

```bash
[please@ruby]:[~] $ wosh --help
Usage: wosh [OPTIONS] --letters <LETTERS>

Options:
  -l, --letters <LETTERS>
  -I, --interactive                         # not currently implemented
  -i, --ignore <IGNORE>      [default: _]
      --spaces <SPACES>      [default: 0]   # not fully implemented
  -w, --wordlist <WORDLIST>
  -s, --sublist <SUBLIST>
  -t, --threads <THREADS>    split the wordlist into specified number of chunks to be processed in their own threads. [default: 15]
  -h, --help                 Print help
  -V, --version              Print version
```

## Initial Run

If no wordlist or sublist filepath is explicitly given to `wosh` as an argument, it will try to find a list in your operating system's default local data directory (see the 
table below). If the lists also cannot be found in this default location, `wosh` will ask to download files from this repository. This will happen on-demand.

> e.g:

```bash
[please@ruby]:[~] $ wosh -l asdf...

[ERR]: Unable to find required wordlists (default wordlists expected in directory '/home/please/.local/share/wosh').
[ERR]: I can download this automatically from url
       => 'https://raw.githubusercontent.com/plsuwu/wosh/master/wordlist'
[ERR]: Continue? [Y/n]:
y
   => Writing to file (path: '/home/please/.local/share/wosh/wordlist'):
      .
   => Fetch ok, returning.
# ...
```

|**OS** |**'Local Data' directory** |
|--- |--- |
|Linux |`$XDG_DATA_HOME` _or_ `$HOME/.local/share/wosh/` |
|MacOS |`$HOME/Library/Application Support/wosh/` |
|Windows |`%LOCALAPPDATA%\wosh\` |

> [path stuff is handled by `dirs`](https://crates.io/crates/dirs)
