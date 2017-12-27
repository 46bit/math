'''
cat day_4_input.txt | pypy3 day_4_1.py
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

total = 0
valid = 0
invalid = 0
valid_sector_ids_sum = 0
for line in sys.stdin.readlines():
    match = re.match("^([a-z0-9-]+)-([0-9]+)\[([a-z0-9-]+)\]$", line)
    encrypted_name = match.group(1)
    sector_id = int(match.group(2))
    checksum = match.group(3)
    print(repr(encrypted_name), repr(sector_id), repr(checksum))

    counter = Counter()
    for character in list(re.sub("[0-9-]", "", encrypted_name)):
        counter[character] += 1
    commonest = most_common(counter, 5)
    expected_checksum = "".join([common[0] for common in commonest])
    print("  ", counter, repr(expected_checksum))

    total += 1
    if checksum == expected_checksum:
        valid += 1
        valid_sector_ids_sum += sector_id
        print("  valid")
    else:
        invalid += 1
        print("  invalid")

print("Of %d rooms, %d valid and %d invalid." % (total, valid, invalid))
print("Sum of sector IDs of valid rooms is %d." % valid_sector_ids_sum)
