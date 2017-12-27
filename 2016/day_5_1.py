'''
cat day_5_input.txt | pypy3 day_5_1.py
'''

import sys, re, hashlib

door_id = sys.stdin.readline().strip()
print("door_id=%s" % door_id)

password = ""
i = 0
while len(password) < 8:
    hash_input = bytes(door_id + str(i), encoding="utf-8")
    md5 = hashlib.md5()
    md5.update(hash_input)
    hash_output = md5.hexdigest()

    match = re.match("^00000([0-9a-z])", hash_output)
    if match:
        print(hash_output, match.group(1), i)
        password += match.group(1)
    i += 1
print("password=%s" % password)
