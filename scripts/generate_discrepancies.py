#!/usr/bin/env python3

import os
import random
import string
import sys


def _get_random_string(length=10):
    return ''.join(random.choice(string.ascii_letters) for i in range(length))


def generate_discrepancies(directory):
    #   .1% of files would have file size mismatch
    #   .4% of files would have be missing
    # 99.5% of files would be OK

    num_mismatch = 0
    num_missing = 0
    num_ok = 0

    for root, _dirs, files in os.walk(directory):
        for filename in files:
            filename = os.path.join(root, filename)
            n = random.random()
            if n <= 0.001:
                with open(filename, 'a') as file_handler:
                    file_handler.write(_get_random_string())
                num_mismatch += 1
            elif n <= 0.005:
                os.remove(filename)
                num_missing += 1
            else:
                num_ok += 1
    print(f'After generating discrepancies, there should be {num_ok} OK, {num_missing} missing, {num_mismatch} with size mismatch')


if __name__ == '__main__':
    if len(sys.argv) != 2:
        print('Please provide the directory path to generate discrepancies in')
        sys.exit(1)
    generate_discrepancies(sys.argv[1])
