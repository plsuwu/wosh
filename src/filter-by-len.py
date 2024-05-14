#!/usr/bin/env python

with open("wordlist_processed", "r") as wl:
    words = wl.readlines()
    sorted_words = sorted(words, key=len)

    long_idx = -1

    for i, w in enumerate(sorted_words):
        if len(w) > 10:
            long_idx = i
            break

    print('cropping list @ index: ', long_idx)

    cropped_words = sorted_words[:long_idx]

    with open("cropped_wordlist", "w") as wr:
        wr.writelines(cropped_words)


    # print(long_words)




    # print(sorted_words)




# with open('wos-boardlist-no-wl.csv', 'r') as wor:
#     with open('wos-boardlist-only-wl.csv', 'r') as let:
#         with open('wos-combined.csv', 'w') as combined:
#             letlines = let.readlines()
#             worlines = wor.readlines()
#
#             for i in range(len(letlines)):
#                 line = worlines[i].strip() + letlines[i]
#                 combined.write(line)
#
# with open('wos-combined.csv', 'r') as r:
#     with open('wos-sorted.csv', 'w') as w:
#         first_word = r.readlines()
#
#         all = []
#
#         for line in first_word:
#             letters = line.strip().split(",")[0]
#             row = line.strip().rsplit(",")
#             row.remove("".join(letters))
#             list_letters = ([*letters])
#             list_letters.sort()
#             reletter = "".join(list_letters)
#
#             row.insert(0, reletter)
#             rerow = ",".join(row)
#             # print(rerow)
#             all.append(rerow)
#
#
#         all.sort()
#         # print(all)
#         #
#         for line in all:
#             writeme = line + '\n'
#             w.writelines(writeme)
#             # print(line)
#


