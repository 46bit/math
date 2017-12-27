'''
cat day_4_input.txt | pypy3 day_4_2.py | grep nortn
'''

import sys, re
from collections import Counter

# Need to implement own most_common:
#   Counter.most_common "Elements with equal counts are ordered arbitrarily"
# This is O(n.m).
def most_common(counter, n):
    counter = counter.copy()
    arr = []
    for i in range(n):
        max_element = ["\0", 0]
        for element in counter.elements():
            element_count = counter[element]
            if element_count > max_element[1]:
                max_element[0] = element
                max_element[1] = element_count
            elif element_count == max_element[1] and ord(element) < ord(max_element[0]):
                max_element[0] = element
        del counter[max_element[0]]
        arr.append(max_element)
    return arr

valid_rooms = []
for line in sys.stdin.readlines():
    match = re.match("^([a-z0-9-]+)-([0-9]+)\[([a-z0-9-]+)\]$", line)
    encrypted_name = match.group(1)
    sector_id = int(match.group(2))
    checksum = match.group(3)

    counter = Counter()
    for character in list(re.sub("[0-9-]", "", encrypted_name)):
        counter[character] += 1
    commonest = most_common(counter, 5)
    expected_checksum = "".join([common[0] for common in commonest])

    if checksum == expected_checksum:
        valid_rooms.append((encrypted_name, sector_id, checksum))

'''
To decrypt a room name, rotate each letter forward through the alphabet a number of times equal
to the room's sector ID. A becomes B, B becomes C, Z becomes A, and so on. Dashes become spaces.
'''

def decrypt_room(room):
    print(room)
    encrypted_name, sector_id, checksum = room
    decrypted_name = ""
    for character in encrypted_name:
        ascii_code = ord(character)
        if ord("a") <= ascii_code <= ord("z"):
            alphabet_position = ord(character) - ord("a")
            new_alphabet_position = (alphabet_position + sector_id) % 26
            new_character = chr(new_alphabet_position + ord("a"))
        elif character == "-":
            new_character = " "
        decrypted_name += new_character
    return decrypted_name

for valid_room in valid_rooms:
    decrypted_name = decrypt_room(valid_room)
    print(decrypted_name, valid_room)
