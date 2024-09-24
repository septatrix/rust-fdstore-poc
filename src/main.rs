use libsystemd::activation::receive_descriptors_with_names;
use libsystemd::daemon::{notify, notify_with_fds, NotifyState};
use rustix::fs::{ftruncate, memfd_create, MemfdFlags};
use rustix::mm::{mmap, MapFlags, ProtFlags};
use std::fs::File;
use std::io::Result;
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd, OwnedFd};
use std::process::exit;
use std::slice;
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const MEMFD_SIZE: usize = 16;

fn main() -> Result<()> {
    let memfd = if let Some(memfd) = get_from_fdstore("memfd") {
        println!("Retrieved memfd from systemd FD store: {:?}", memfd);
        let file = File::from(memfd);
        assert_eq!(file.metadata()?.len(), MEMFD_SIZE as u64);
        OwnedFd::from(file)
    } else {
        // Create a new memfd
        let memfd = memfd_create("rust_memfd_example", MemfdFlags::CLOEXEC)?;

        // Resize the memfd to 512 bytes
        ftruncate(&memfd, MEMFD_SIZE as u64)?;

        // Store the FD in the systemd FD store
        println!("Storing memfd in systemd FD store: {:?}", memfd);
        notify_with_fds(
            false,
            &[
                NotifyState::Fdstore,
                NotifyState::Fdname("memfd".to_owned()),
            ],
            &[memfd.as_raw_fd()],
        )
        .unwrap();
        memfd
    };

    notify(
        true,
        &[
            NotifyState::Ready,
            NotifyState::Status("Running notify test".to_owned()),
        ],
    )
    .unwrap();

    // Map the memfd to memory
    let data = unsafe {
        let addr = mmap(
            std::ptr::null_mut(),
            MEMFD_SIZE,
            ProtFlags::READ | ProtFlags::WRITE, // allow read and write
            MapFlags::SHARED, // ensure that changes are reflected in the underlying file
            memfd,
            0,
        )?;
        // Convert the mapped memory to a mutable slice
        slice::from_raw_parts_mut(addr as *mut u8, MEMFD_SIZE)
    };
    let buffer: &mut [u8; MEMFD_SIZE] = data.try_into().expect("Slice has wrong length");

    loop {
        println!("{:?}", buffer);

        // Just a rudimentary RNG
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();
        let i = (nanos % (MEMFD_SIZE as u32)) as usize;

        // Simulate some changes being done to the buffer
        buffer[i] += 1;

        if nanos % 50 == 0 {
            eprintln!("Simulating crash!");
            exit(1);
        }

        sleep(Duration::from_millis(20));
    }
}

fn get_from_fdstore(name: &str) -> Option<OwnedFd> {
    // Check if we have an existing FD from systemd
    let listening_fds = receive_descriptors_with_names(false).ok()?;

    for fd in listening_fds.into_iter() {
        if fd.1 == name {
            // TODO also check is_socket() etc
            return Some(unsafe { OwnedFd::from_raw_fd(fd.0.into_raw_fd()) });
        }
    }
    None
}
