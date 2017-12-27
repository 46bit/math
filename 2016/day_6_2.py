'''
cat day_6_input.txt | pypy3 day_6_2.py
'''

import sys
from collections import Counter

messages = [message.strip() for message in sys.stdin.readlines()]
message_length = len(messages[0])
counters = [Counter() for i in range(message_length)]
for message in messages:
    for i in range(message_length):
        character = message[i]
        counters[i][character] -= 1
corrected_message = "".join([counter.most_common(1)[0][0] for counter in counters])
print(corrected_message)
