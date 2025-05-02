# JOS - Jakob's OS

Experimental busy dad's learning activity. 

Rough list of features

- ✅ use Rust instead of C
- ✅ interrupts (time, keyboard)
- ✅ paging
- ✅ heap memory
- ✅ higher half kernel
- ✅ userspace (ring 3) programs
- ✅ multiprocessing with simple scheduler
- ✅ syscalls with userspace libc in Rust and C
- ✅ VGA mode
- ✅ "fake file operations" (read files embedded in executable)
- ✅ run Doom
- ✅ use High Precision Event Timer CPU feature to get nanosecond resolution timestamps
- ✅ colored logging
- ✅ pretty logo
- ✅ print logs to Serial
- ✅ Read Ext2 filesystem images
- enable userspace processes to communicate via an IPC
- network stack
- make running on RISC-V (ESP32-C3?) additionally to x86_64

Rough list of ecosystem 

- ✅ Github CI action to build
- ✅ Automatic testing via serial connection
- ✅ gdb debugging
- adopt Bazel instead of make

## Compile and Run

- `docker build buildenv_rust -t jos_buildenv`
- `docker run --rm --privileged -it -v "${pwd}:/root/env" jos_buildenv`
- `make build-x86_64`
- (other shell) `qemu-system-x86_64 -no-reboot -cdrom dist/x86_64/kernel.iso`

https://wiki.osdev.org/QEMU#Useful_QEMU_command-line_options

## Reverse Debugging via qemu

(couldn't get this working yet)

https://www.qemu.org/docs/master/system/replay.html

- `qemu-img create -f qcow2 empty.qcow2 1G`
- `qemu-system-x86_64 -s -S -d int,cpu_reset,guest_errors -action panic=pause -no-reboot  -monitor stdio -cdrom dist/x86_64/kernel.iso -icount shift=auto,rr=record,rrfile=record.bin,rrsnapshot=init -net none -drive file=empty.qcow2,if=none,id=rr`
- `savevm TAG` (multiple times if needed)
- `quit`
- `qemu-system-x86_64 -s -S -d int,cpu_reset,guest_errors -action panic=pause -no-reboot  -monitor stdio -cdrom dist/x86_64/kernel.iso -icount shift=auto,rr=replay,rrfile=record.bin,rrsnapshot=init -net none -drive file=empty.qcow2,if=none,id=rr`
- `loadvm TAG`

## Kudos

Bootstrapped by "Write Your Own 64-bit Operating System Kernel From Scratch" by https://github.com/davidcallanan/os-series