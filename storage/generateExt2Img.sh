#!/bin/bash
set -eux pipefail

# Create a 10 MB file
dd if=/dev/zero of=disk.img bs=1M count=10

# Create an ext2 filesystem on it
mkfs.ext2 disk.img

# Create a temporary mount point
mkdir -p /tmp/disk

# Check if running in Docker
if [ -f /.dockerenv ]; then
    sudo losetup -fP disk.img
    LOOPDEV=$(sudo losetup | sort -V | tail -n1 | cut -d' ' -f1)
    echo $LOOPDEV
    sudo mount $LOOPDEV /tmp/disk
else
    sudo mount -t ext2 disk.img /tmp/disk
fi

# Change ownership to current user
# sudo chown "$USER":"$USER" /tmp/disk

# Copy the file
sudo cp ../doom1.wad /tmp/disk/devdatadoom1.wad
sudo cp ../build/userspace/x86_64-unknown-none/debug/doom /tmp/disk/doom

# List files
ls -l -a /tmp/disk

# Unmount
sudo umount /tmp/disk

# Clean up
#rm -r /tmp/disk