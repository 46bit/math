'''
cat day_3_input.txt | pypy3 day_3_1.py
'''

import sys, re

total = 0
valid = 0
invalid = 0
for line in sys.stdin.readlines():
    tokens = [int(match.group(0)) for match in re.finditer("([0-9]+)", line)]
    print(line, tokens)

    a = tokens[0] + tokens[1] - tokens[2]
    b = tokens[0] - tokens[1] + tokens[2]
    c = - tokens[0] + tokens[1] + tokens[2]

    total += 1
    if a <= 0 or b <= 0 or c <= 0:
        invalid += 1
    else:
        valid += 1

print("Of %d triangles, %d valid and %d invalid." % (total, valid, invalid))
