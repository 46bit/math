'''
cat day_1_input.txt | pypy3 day_1_1.py
'''

import sys

tokens = sys.stdin.read().split(", ")

start = [0, 0]
direction_vectors = (
  (0, -1),
  (1, 0),
  (0, 1),
  (-1, 0)
)

position = start
direction = 0

i = -1
for token in tokens:
    i += 1
    turning = token[0]
    moves = int(token[1:])
    print(i, token, turning, moves)
    if turning == "L":
        direction -= 1
        direction %= 4
    elif turning == "R":
        direction += 1
        direction %= 4
    else:
        print("Unknown turning for token", token)
        continue
    direction_vector = direction_vectors[direction]
    position[0] += direction_vector[0] * moves
    position[1] += direction_vector[1] * moves
    print(position)

print("Ended at", position)
print("Distance from start", abs(position[0]) + abs(position[1]))

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
