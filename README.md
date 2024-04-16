# words on stream 'helper'

> i cosplay as someone intelligent

hardcoded to work with my system.

**usage**

let rust find the wordlist `.csv` (either with `-w` or change in `main.rs`). install to `PATH` with cargo

```bash
$ cargo install --path .
```

pretty simplistic compared to some previous iterations. 

- `--letters`/`-l` to pass a set of letters to match against the wordlist,
- pass a period - `-l .` - to indicate a hidden letter,
- use `-H` to indicate the letters passed in `--letters`/`-l` contains fake letter(s) (i think this is also required when passing a hidden letter).
- if there are too many results, use `-i` to filter out unwanted substitution letters, or `-s` to indicate the total number of words a board has space for

```bash
$ wosh [-H] -l abc.... [ -s | -w </path/to/wordlist.csv> | -i zxyj ]
```

- built on `clap` so pass `--help` to view a list of commands (a handful of these aren't implemented lmao;l)

```bash
$ wosh --help
Usage: wosh [OPTIONS] --letters <LETTERS>

Options:
  -l, --letters <LETTERS>
  -w, --wordlist <WORDLIST>      [default: /home/please/Documents/Repositories/plsuwu/wosh/wos-sorted.csv]
  -H, --contains-hidden
  -s, --spaces <SPACES>          [default: 55]
  -m, --min-length <MIN_LENGTH>  [default: 4]     # no longer implemented, does nothing.
  -i, --ignore <IGNORE>          [default: _]
  -h, --help                     Print help
  -V, --version                  Print version
```

