'''
cat day_3_input.txt | pypy3 day_3_2.py
'''

import sys, re

rows = []
for line in sys.stdin.readlines():
    tokens = [int(match.group(0)) for match in re.finditer("([0-9]+)", line)]
    print(line, tokens)
    rows.append(tokens)

total = 0
valid = 0
invalid = 0

triangles_len = len(rows)
for t in range(3):
    for i in range(0, triangles_len, 3):
        tokens = [rows[i][t], rows[i+1][t], rows[i+2][t]]

        a = tokens[0] + tokens[1] - tokens[2]
        b = tokens[0] - tokens[1] + tokens[2]
        c = - tokens[0] + tokens[1] + tokens[2]

        total += 1
        if a <= 0 or b <= 0 or c <= 0:
            invalid += 1
        else:
            valid += 1

print("Of %d triangles, %d valid and %d invalid." % (total, valid, invalid))
