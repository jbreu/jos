#!/bin/bash

# Create a 10 MB file
dd if=/dev/zero of=disk.img bs=1M count=10

# Create an ext2 filesystem on it
mkfs.ext2 disk.img

# Create a temporary mount point
mkdir -p /tmp/disk

# Check if running in Docker
if [ -f /.dockerenv ]; then
    sudo losetup -fP disk.img
    sudo losetup    # to find which /dev/loopX
    sudo mount /dev/loopX /tmp/disk
else
    sudo mount -t ext2 disk.img /tmp/disk
fi

# Change ownership to current user
sudo chown $USER:$USER /tmp/disk

# Copy the file
cp test.txt /tmp/disk/
cp ../doom1.wad /tmp/disk/devdatadoom1.wad

# List files
ls -l -a /tmp/disk

# Unmount
sudo umount /tmp/disk

# Clean up
rm -r /tmp/disk