'''
cat day_2_input.txt | pypy3 day_2_2.py
'''

import sys

# (y, x)
keypad_values = {
    (0, 2): "1",
    (1, 1): "2",
    (1, 2): "3",
    (1, 3): "4",
    (2, 0): "5",
    (2, 1): "6",
    (2, 2): "7",
    (2, 3): "8",
    (2, 4): "9",
    (3, 1): "A",
    (3, 2): "B",
    (3, 3): "C",
    (4, 2): "D"
}
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
        if tuple(position) not in keypad_values:
            print("Outside of keypad, reverting from", position, "to", prev_position)
            position = prev_position
        print(l, i, position)

    end = tuple(position)
    key = keypad_values[end]
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
