#!/usr/bin/env python3

def square(i: int):
    return i**2

if __name__ =="__main__":
    import sys
    print(square(int(sys.argv[-1])))
