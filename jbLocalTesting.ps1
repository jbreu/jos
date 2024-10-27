# https://stackoverflow.com/a/28482050
$qemu = Get-Process qemu-system-x86_64 -ErrorAction SilentlyContinue
if ($qemu) {
  if (!$qemu.HasExited) {
    $qemu | Stop-Process -Force
  }
}
Remove-Variable qemu

docker run --rm -it -v "${pwd}:/root/env" jos_buildenv make -B build-x86_64
qemu-system-x86_64 -cdrom dist/x86_64/kernel.iso


##################

docker run -it --rm --env=BOOT_MODE=legacy --env=CPU_CORES=4 --env=RAM_SIZE=4G --env=DISK_SIZE=16G --volume=/mnt/c/Users/Jakob/Documents/workspace/os-series/dist/x86_64/kernel.iso:/boot.iso -p 22 -p 5900:5900 -p 8006:8006 -p 5700:5700 --restart=no --runtime=runc --cap-add NET_ADMIN --env=DISPLAY=web --env=VGA=std --env=DEBUG=Y --device=/dev/kvm --device=/dev/dri -d qemux/qemu-docker:latest

docker run --env=BOOT_MODE=legacy --env=CPU_CORES=4 --env=RAM_SIZE=4G --env=DISK_SIZE=16G  --volume=C:\Users\Jakob\Documents\workspace\os-series\dist\x86_64\kernel.iso:/boot.iso --network=bridge -p 22 -p 5900 -p 8006:8006 --restart=no --runtime=runc --cap-add NET_ADMIN --env=DISPLAY=web --device=/dev/kvm -d qemux/qemu-docker:latest