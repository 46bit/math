'''
cat day_2_input.txt | pypy3 day_2_1.py
'''

import sys

# (y, x)
keypad_values = (
    (1, 2, 3),
    (4, 5, 6),
    (7, 8, 9)
)
start = (1, 1)

keys = []

l = -1
for line in sys.stdin.readlines():
    l += 1
    moves = list(line)
    position = list(start)

    i = -1
    for move in moves:
        i += 1
        prev_position = list(position)
        print(l, i, move)
        if move == "L":
            position[1] -= 1
        elif move == "U":
            position[0] -= 1
        elif move == "R":
            position[1] += 1
        elif move == "D":
            position[0] += 1
        else:
            print("Unknown move", move)
        if not (0 <= position[0] < len(keypad_values) and 0 <= position[1] < len(keypad_values[position[0]])):
            print("Outside of keypad, reverting from", position, "to", prev_position)
            position = prev_position
        print(l, i, position)

    end = tuple(position)
    key = keypad_values[position[0]][position[1]]
    print("Ended at", end, "which is key", key)
    keys.append(key)
print(keys)

'''
1 0   0 =  0
0 1  -1   -1

0 1   0 = -1
0 1  -1   -1

1 1   0 = -1
0 1  -1   -1

left
 0  -1   0   1    0
-1   0   1   0   -1
'''
