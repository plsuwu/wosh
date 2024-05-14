# Words on Stream "Helper"

> because i am a litarate

## installation

building from source is easiest with [rustup](https://rustup.rs/); it should just be a matter of cloning the repo
and running `cargo install` to dump a binary into your `PATH`:

```bash
# on linux (though should be cross-platform) (i haven't tested on windows)
git clone https://github.com/plsuwu/wosh
cd wosh
cargo install --path .
```

## usage

> note: output intended to be read bottom-to-top.

the idea is that this is as quick and easy as possible to run; generally, you will only want to use the `-l`/`--letters` arg:

```bash
# pass letters with `-l`/`--letters` - any `[?]` on the board are replaced with `.`:
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

[ RESULT 2 OF 2 ]:

=> [h]: 'b'
=> [x]: 'z'
=> [^]: 'handball'

# the main wordlist isn't a complete dataset; any time a `???...` is found instead of a word,
# the letters and word length are cross-referenced from a substitution list to suggest possible
# words:

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

- built on `clap` so `--help` can be passed to view a list of available commands

```bash
[please@ruby]:[~] $ wosh --help
Usage: wosh [OPTIONS] --letters <LETTERS>

Options:
  -l, --letters <LETTERS>
  -I, --interactive                         # not currently implemented
  -i, --ignore <IGNORE>      [default: _]
      --spaces <SPACES>      [default: 0]   # not fully implemented
  -w, --wordlist <WORDLIST>                 # defaults to '~/.local/share/wosh/wordlist'; asks to download if not present.
  -s, --sublist <SUBLIST>                   # defaults to '~/.local/share/wosh/sublist'; asks to download if not present.
  -t, --threads <THREADS>    split the wordlist into specified number of chunks to be processed in their own threads. [default: 15]
  -h, --help                 Print help
  -V, --version              Print version
```

### initial run

if no wordlist/sublist path is passed as an argument and cannot be found in the default location, `wosh` will prompt to download
when the files are not found but required by the program:

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
[ RESULT 1 OF 15 ]:

=> [h]: 'c, e, r'
=> [x]: 'd, f'
=> [^]: 'acres'

[00]: scare
# ...
```

> this also goes for the sublist:

```bash
# ...
[ RESULT 2 OF 15 ]:

=> [h]: 'b, e, r'
=> [x]: 's, f'
=> [^]: 'bread'

[ERR]: Unable to find required sublists (default wordlists expected in directory '/home/please/.local/share/wosh').
[ERR]: I can download this automatically from url
       => 'https://raw.githubusercontent.com/plsuwu/wosh/master/sublist'
[ERR]: Continue? [Y/n]:
y
[00]: ?????    => Writing to file (path: '/home/please/.local/share/wosh/sublist'):
       .
    => Fetch ok, returning.
=>
    [
      [00 | ardeb ],
# ...
```

