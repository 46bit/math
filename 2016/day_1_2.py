'''
cat day_1_input.txt | pypy3 day_1_2.py | grep Revisiting
'''

import sys

tokens = sys.stdin.read().split(", ")

start = (0, 0)
direction_vectors = (
  (0, -1),
  (1, 0),
  (0, 1),
  (-1, 0)
)

position = start
direction = 0
visited = set()

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
    for move in range(moves):
        position = (
            position[0] + direction_vector[0],
            position[1] + direction_vector[1]
        )
        if position in visited:
            print("Revisiting", position, "which is a distance", abs(position[0]) + abs(position[1]), "from start.")
        else:
            visited.add(position)
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
