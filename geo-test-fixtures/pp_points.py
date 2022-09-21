#!/usr/bin/env python3
''' Script to parse coordinates through debug log, and replace them with easier
names.

'''

import sys
import re

def main():
    pts = {}

    float_regex = r"[-+]?(?:\d*\.\d+|\d+)"
    coord_regex = f"({float_regex}),? ({float_regex})"
    regex = re.compile(coord_regex)

    line = sys.stdin.readline()
    while line:
        if line.startswith('input:'):
            print(line, end='')
        m = regex.search(line)
        while m:
            st = m.start()
            print(line[:st], end='')
            sig = m.expand(r"\1#\2")
            if sig not in pts:
                pts[sig] = len(pts)
            idx = pts[sig]
            print(f'⚝{idx}', end='')

            en = m.end()
            line = line[en:]
            m = regex.search(line)
        print(line, end='')
        line = sys.stdin.readline()

    print("end of input")
    print("points:")
    for sig in pts:
        x,y = sig.split('#')
        idx = pts[sig]
        print(f"\t{idx}: Pt({x} {y})")
if __name__ == "__main__":
    main()
