#!/usr/bin/env python

# quick script to remove words that dont meet the 10-minute
# requirement. unsure what the largest words on stream words
# are so im just ignoring it.


def trim_list(words: list[str]) -> list[str]:
    trimmed_list: list[str] = []
    for word in words:
        word_length = len([c for c in word])
        if word != "" and word_length > 3:
            trimmed_list.append(word)

    return trimmed_list


def main():
    with open("words_alpha.txt", "r") as words, open(
        "./wordlist_processed", "w"
    ) as trim:
        w = trim_list(words.read().split("\n"))
        print(f"new wordlist length: {len(w)}")
        trim.writelines("\n".join(w))


if __name__ == "__main__":
    main()
