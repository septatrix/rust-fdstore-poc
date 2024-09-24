# POC for using systemd's fdstore in Rust

The [file descriptor store (short: fdstore)](https://systemd.io/FILE_DESCRIPTOR_STORE/)
provided by systemd is a powerful concept to preserve state across service restarts and crashes.
It can be used to upload file descriptors to the service manager
which will hold on to a duplicate
and provide new instances of the service with those file descriptors.
Linux also has the concept of a [`memfd`](https://man7.org/linux/man-pages/man2/memfd_create.2.html),
an anonymous, memory-backed file
that can be used to store and retrieve arbitrary data.
As it provides a file descriptor it can be uploaded to the fdstore.
Finally, [`mmap`](https://www.man7.org/linux/man-pages/man2/mmap.2.html)
allows one to map file contents directly into process memory.

Together, this can be used to create variables in Rust backed by these features
which can survive crashes and persist the previous values.
While this requires several unsafe regions,
it is - in theory - safe, as no other process
has access to the underlying file
(except the service manager, which only holds on to the fd and does not touch the file contents).

You can run this yourself as follows:

```sh
systemd-run \
    --user \
    --unit fdstore \
    --pty \
    --same-dir \
    --wait \
    --collect \
    --service-type=notify \
    -p FileDescriptorStoreMax=5 \
    -p Restart=always \
    cargo run
```

Randomly, the process will simulate a crash by exiting with an error code.
This causes systemd to restart the process,
passing the previously uploaded fds down to the new process instance.
As the exits occur within a short time span the process
will eventually run into the burst error limit and no longer be restarted.
While the process is running you can inspect the uploaded fds using `systemd-analyze --user fdstore fdstore.service`.
Similarly, you can inspect the status using `systemctl --user status fdstore.service`.
