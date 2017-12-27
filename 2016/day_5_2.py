'''
cat day_5_input.txt | pypy3 day_5_2.py
'''

import sys, re, hashlib

door_id = sys.stdin.readline().strip()
print("door_id=%s" % door_id)

password = {}
i = 0
while len(password) < 8:
    hash_input = bytes(door_id + str(i), encoding="utf-8")
    md5 = hashlib.md5()
    md5.update(hash_input)
    hash_output = md5.hexdigest()

    match = re.match("^00000([0-7])([0-9a-z])", hash_output)
    if match:
        position = int(match.group(1))
        character = match.group(2)
        print(hash_output, position, character, i)
        if position not in password:
            password[position] = character
    i += 1
print("password=%s" % password)
print("password string=%s" % str([password[i] for i in range(8)]))
