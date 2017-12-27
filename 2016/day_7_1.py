'''
cat day_7_input.txt | pypy3 day_7_1.py
'''

import sys, re
from collections import Counter
abbad = 0
items = [item.strip() for item in sys.stdin.readlines()]
for item in items:
    #print(item.split("["))
    #print(re.match("^(.+)\[(.+)\](.+)$", item).groups())

    normal = []
    bracketed = []
    buildup = ""
    inside_brackets = False
    for i in range(len(item)):
        character = item[i]
        if inside_brackets and character == "]":
            inside_brackets = False
            bracketed.append(buildup)
            buildup = ""
        elif not inside_brackets and character == "[":
            inside_brackets = True
            normal.append(buildup)
            buildup = ""
        else:
            buildup += character
    if len(buildup) > 0:
        normal.append(buildup)

    has_abba = False
    for n in normal:
        print(n)
        for j in range(len(n) - 3):
            if n[j] == n[j+3] and n[j+1] == n[j+2] and n[j] != n[j+1]:
              has_abba = True
    for n in bracketed:
        print(n)
        for j in range(len(n) - 3):
            if n[j] == n[j+3] and n[j+1] == n[j+2] and n[j] != n[j+1]:
              has_abba = False
    if has_abba:
        abbad += 1
print(abbad)
