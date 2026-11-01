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
- ✅ Read data via ATA driver from hard disk
- ✅ 4 KB page size
- enable userspace processes to communicate via an IPC
- network stack
- make running on RISC-V (ESP32-C3?) additionally to x86_64

Rough list of ecosystem 

- ✅ Github CI action to build
- ✅ Automatic testing via serial connection
- ✅ gdb debugging
- ✅ performance profiling scripts like flamegraphs, tracing (see fol)
- adopt Bazel instead of make

## Compile and Run

- `docker build buildenv_rust -t jos_buildenv`
- `docker run --rm --privileged -it -v "${pwd}:/root/env" jos_buildenv`
- `make build-x86_64`
- (other shell) `qemu-system-x86_64 -no-reboot -cdrom dist/x86_64/kernel.iso -hda storage/disk.img`

https://wiki.osdev.org/QEMU#Useful_QEMU_command-line_options

## Reverse Debugging via qemu

https://www.qemu.org/docs/master/system/replay.html

- Terminal: `qemu-img create -f qcow2 -F raw -b "$(Resolve-Path storage/disk.img)" storage/recording.qcow2`
- Terminal: `qemu-system-x86_64 `
  -action panic=pause -no-reboot -no-shutdown `
  -monitor stdio -s -S -net none `
  -icount shift=auto,rr=record,rrfile=record.bin,rrsnapshot=init `
  -drive file=storage/recording.qcow2,format=qcow2,if=none,id=disk `
  -drive driver=blkreplay,if=none,image=disk,id=disk-replay `
  -device ide-hd,drive=disk-replay,bus=ide.0,unit=0 `
  -drive file=dist/x86_64/kernel.iso,format=raw,if=none,id=cd,read-only=on `
  -drive driver=blkreplay,if=none,image=cd,id=cd-replay,read-only=on `
  -device ide-cd,drive=cd-replay,bus=ide.1,unit=0`
- Qemu monitor: `savevm TAG` (multiple times if needed)
- Qemu monitor: `quit`
- Terminal: `qemu-system-x86_64 `
  -action panic=pause -no-reboot -no-shutdown `
  -monitor stdio -s -S -net none `
  -icount shift=auto,rr=replay,rrfile=record.bin,rrsnapshot=TAG `
  -drive file=storage/recording.qcow2,format=qcow2,if=none,id=disk `
  -drive driver=blkreplay,if=none,image=disk,id=disk-replay `
  -device ide-hd,drive=disk-replay,bus=ide.0,unit=0 `
  -drive file=dist/x86_64/kernel.iso,format=raw,if=none,id=cd,read-only=on `
  -drive driver=blkreplay,if=none,image=cd,id=cd-replay,read-only=on `
  -device ide-cd,drive=cd-replay,bus=ide.1,unit=0`
- Debug Console (gdb): `rsi`

## Kudos

Bootstrapped by "Write Your Own 64-bit Operating System Kernel From Scratch" by https://github.com/davidcallanan/jos