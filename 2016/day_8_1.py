'''
cat day_8_input.txt | python3 day_8_1.py
'''

import sys, re, copy, numpy as np

width = int(sys.stdin.readline().strip()[6:])
height = int(sys.stdin.readline().strip()[7:])
# screen[y][x]
screen = np.zeros(shape=(height,width), dtype=int)
disp = {0: " ", 1: "#"}

for line in sys.stdin.readlines():
    line = line.strip()

    if line[0:4] == "rect":
        x, y = [int(v) for v in line[5:].split("x")]
        print("rect x=%d y=%d" % (x, y), repr(line))
        screen[:y,:x] = 1
    if line[0:10] == "rotate row":
        y, shift = [int(v) for v in line[13:].split(" by ")]
        print("rotate row y=%d shift=%d" % (y, shift), repr(line))
        screen[y,:] = np.roll(screen[y,:], shift)
    if line[0:13] == "rotate column":
        x, shift = [int(v) for v in line[16:].split(" by ")]
        print("rotate column x=%d shift=%d" % (x, shift), repr(line))
        screen[:,x] = np.roll(screen[:,x], shift)

    for y in range(height):
        print("".join([disp[c] for c in screen[y]]))

popcnt = 0
for y in range(height):
    for x in range(width):
        popcnt += screen[y][x]
print("popcnt=%d" % popcnt)
