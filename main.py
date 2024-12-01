#!/usr/bin/env python3

import os
import mmap
import random
import sys
import time


from systemd.daemon import listen_fds_with_names, notify

MEMFD_SIZE = 16


def main():
    if memfd := get_from_fdstore("memfd"):
        print(f"Retrieved memfd from systemd FD store: {memfd}")
    else:
        memfd = os.memfd_create("python_memfd_example", flags=os.MFD_CLOEXEC)
        os.truncate(memfd, MEMFD_SIZE)
        print(f"Storing memfd in systemd FD store: {memfd}")
        notify("FDSTORE=1\nFDNAME=memfd", fds=[memfd])

    print(f"Working on fd {memfd}")

    notify("READY=1\nSTATUS=Running notify test")

    data = mmap.mmap(
        memfd, MEMFD_SIZE, flags=mmap.MAP_SHARED, prot=mmap.PROT_READ | mmap.PROT_WRITE
    )

    while True:
        print(list(data[:]))
        data[random.randrange(0, MEMFD_SIZE)] += 1
        if random.randrange(0, 50) == 0:
            print("Simulating crash!", file=sys.stderr)
            sys.exit(1)

        time.sleep(0.02)


def get_from_fdstore(name: str) -> int | None:
    for fd, fd_name in listen_fds_with_names().items():
        if fd_name == name:
            return fd
    return None


if __name__ == "__main__":
    main()
