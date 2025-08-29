extern crate alloc;
use crate::hdd::*;
use crate::kprintln;
use crate::DEBUG;
use crate::ERROR;
use alloc::vec::Vec;
use core::fmt;
use include_bytes_aligned::include_bytes_aligned;
use lazy_static::lazy_static;

const DISK_IMG: &[u8] = include_bytes_aligned!(8, "../../storage/disk.img");

lazy_static! {
    pub static ref FILE_SYSTEM: Ext2FileSystem = Ext2FileSystem::new();
}

#[derive(Clone)]
pub struct FileHandle {
    inode: Inode,
    pub offset: usize,
}

impl fmt::Debug for FileHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FileHandle")
            .field("offset", &self.offset)
            .finish()
    }
}

impl FileHandle {
    pub fn new(filename: &str, _mode: u32) -> Option<FileHandle> {
        let _event = core::hint::black_box(crate::instrument!());
        match FILE_SYSTEM.read_inode_by_path(filename) {
            Some(inode) => return Some(Self { inode, offset: 0 }),
            None => {
                return None;
            }
        }
    }

    pub fn read(&mut self, buffer: *mut u8, size: usize) -> u64 {
        let _event = core::hint::black_box(crate::instrument!());
        let mut bytes_read = 0;
        let mut buffer_offset = 0;
        let total_size = self.inode.size as usize;
        let n: usize = FILE_SYSTEM.block_size as usize / core::mem::size_of::<u32>() as usize;

        // Don't read past the file's end
        let remaining = total_size - self.offset;
        let to_read = core::cmp::min(size, remaining);

        while bytes_read < to_read {
            let current_block_idx = self.offset / FILE_SYSTEM.block_size as usize;
            let offset_in_block = self.offset % FILE_SYSTEM.block_size as usize;

            // Get the block number based on index
            let block_num = if current_block_idx < 12 {
                // Direct blocks
                self.inode.direct_blocks[current_block_idx]
            } else if current_block_idx < 12 + n {
                // Single indirect (256 blocks)
                let indirect_block = FILE_SYSTEM.read_block(self.inode.indirect_block);
                let idx = current_block_idx - 12;
                unsafe { *(indirect_block.as_ptr().add(idx * 4) as *const u32) }
            } else if current_block_idx < 12 + n + n * n {
                // Double indirect (65,804 blocks total)
                let double_indirect_block = FILE_SYSTEM.read_block(self.inode.double_indirect);
                let primary_idx = (current_block_idx - (12 + n)) / n;
                let secondary_idx = (current_block_idx - (12 + n)) % n;

                if primary_idx < n {
                    let primary_ptr = unsafe {
                        *(double_indirect_block.as_ptr().add(primary_idx * 4) as *const u32)
                    };
                    let secondary_block = FILE_SYSTEM.read_block(primary_ptr);
                    unsafe { *(secondary_block.as_ptr().add(secondary_idx * 4) as *const u32) }
                } else {
                    break;
                }
            }
            // Triple indirect (16,843,020 blocks total)
            else if current_block_idx < 12 + n + n * n + n * n * n {
                let triple_indirect_block = FILE_SYSTEM.read_block(self.inode.triple_indirect);
                let offset = current_block_idx - (12 + n + n * n);
                let first_idx = offset / (n * n);
                let second_idx = (offset / n) % n;
                let third_idx = offset % n;

                let first_ptr =
                    unsafe { *(triple_indirect_block.as_ptr().add(first_idx * 4) as *const u32) };
                let second_block = FILE_SYSTEM.read_block(first_ptr);
                let second_ptr =
                    unsafe { *(second_block.as_ptr().add(second_idx * 4) as *const u32) };
                let third_block = FILE_SYSTEM.read_block(second_ptr);
                unsafe { *(third_block.as_ptr().add(third_idx * 4) as *const u32) }
            } else {
                break;
            };

            let block = FILE_SYSTEM.read_block(block_num);
            let block_remaining = FILE_SYSTEM.block_size as usize - offset_in_block;
            let can_read = core::cmp::min(to_read - bytes_read, block_remaining);

            unsafe {
                core::ptr::copy_nonoverlapping(
                    block.as_ptr().add(offset_in_block),
                    buffer.add(buffer_offset),
                    can_read,
                );
            }

            bytes_read += can_read;
            buffer_offset += can_read;
            self.offset += can_read;

            if bytes_read >= to_read {
                break;
            }
        }

        bytes_read as u64
    }

    pub fn fseek(&mut self, offset: usize, origin: u32) -> u64 {
        let _event = core::hint::black_box(crate::instrument!());
        match origin {
            0 => self.offset = offset,
            1 => self.offset += offset,
            2 => self.offset = self.inode.size as usize - offset,
            _ => {
                ERROR!("Invalid origin: {}", origin);
                return 1; // error case
            }
        }
        return 0;
    }
}

//https://slideplayer.com/slide/16554195/96/images/48/Linux+Example:+Ext2/3+Disk+Layout.jpg
//https://www.cs.unibo.it/~renzo/so/lecture_examples2324/20240411/ext2-walkthrough.pdf

#[derive(Debug)]
pub struct Ext2FileSystem {
    superblock: Superblock,
    block_groups: Vec<BlockGroupDescriptor>,
    block_size: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct Superblock {
    inode_count: u32,
    block_count: u32,
    reserved_blocks: u32,
    free_blocks: u32,
    free_inodes: u32,
    first_data_block: u32,
    block_size_shift: u32,
    fragment_size_shift: u32,
    blocks_per_group: u32,
    fragments_per_group: u32,
    inodes_per_group: u32,
    last_mount_time: u32,
    last_write_time: u32,
    mount_count: u16,
    max_mount_count: u16,
    magic: u16,
    state: u16,
    errors: u16,
    minor_rev_level: u16,
    lastcheck: u32,
    checkinterval: u32,
    creator_os: u32,
    rev_level: u32,
    def_resuid: u16,
    def_resgid: u16,
    first_nonreserved_inode: u32,
    inode_size: u16,
    block_group_nr: u16,
    compatible_features: u32,
    incompatible_features: u32,
    readonly_features: u32,
    fs_id: [u8; 16],
    volume_name: [u8; 16],
    last_mounted_path: [u8; 64],
    compression_algo: u32,
    preallocate_blocks_file: u8,
    preallocate_blocks_dir: u8,
    _unused1: u16,
    journal_id: [u8; 16],
    journal_inode: u32,
    journal_device: u32,
    orphan_inode_list: u32,
    _unused2: [u8; 788],
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct BlockGroupDescriptor {
    block_bitmap: u32,
    inode_bitmap: u32,
    inode_table: u32,
    free_blocks_count: u16,
    free_inodes_count: u16,
    used_dirs_count: u16,
    _pad: [u8; 14], // Combined pad and reserved into one field
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct Inode {
    mode: u16,
    uid: u16,
    size: u32,
    atime: u32,
    ctime: u32,
    mtime: u32,
    dtime: u32,
    gid: u16,
    links_count: u16,
    blocks: u32,
    flags: u32,
    _reserved1: u32,
    direct_blocks: [u32; 12],
    indirect_block: u32,
    double_indirect: u32,
    triple_indirect: u32,
    _remaining: [u8; 28], // Combined remaining fields
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct DirectoryEntry {
    inode: u32,
    rec_len: u16,
    name_len: u8,
    file_type: u8,
}

impl Ext2FileSystem {
    pub fn new() -> Self {
        let _event = core::hint::black_box(crate::instrument!());
        //kprintln!("Disk image location: {:p}", DISK_IMG.as_ptr());

        // Superblock always starts at offset 1024 and is 1024 bytes long
        let mut superblock_bytes: [u8; 1024] = [0; 1024];
        hdd_read(2, 2, superblock_bytes.as_mut());
        let superblock = unsafe { core::ptr::read(superblock_bytes.as_ptr() as *const Superblock) };

        let block_size = 1024 << superblock.block_size_shift;
        let blocks_per_group = superblock.blocks_per_group;
        let group_count = (superblock.block_count + blocks_per_group - 1) / blocks_per_group;

        // GDT starts at the block after superblock
        let gdt_block = if block_size == 1024 { 2 } else { 1 };
        let gdt_offset = gdt_block * block_size;

        let mut block_groups = Vec::new();
        for i in 0..group_count {
            let offset =
                gdt_offset as usize + i as usize * core::mem::size_of::<BlockGroupDescriptor>();
            let block_group =
                unsafe { *(DISK_IMG.as_ptr().add(offset) as *const BlockGroupDescriptor) };
            kprintln!(
                "Reading block group {} at offset {:#x}: {:?}",
                i,
                offset,
                block_group
            );
            block_groups.push(block_group);
        }

        Self {
            superblock,
            block_groups,
            block_size,
        }
    }

    fn read_block(&self, block_num: u32) -> &[u8] {
        let start = (block_num as usize) * (self.block_size as usize);
        &DISK_IMG[start..start + self.block_size as usize]
    }

    fn read_inode(&self, inode_num: u32) -> Inode {
        let group = (inode_num - 1) / self.superblock.inodes_per_group;
        let index = (inode_num - 1) % self.superblock.inodes_per_group;
        let block_group: &BlockGroupDescriptor = &self.block_groups[group as usize];

        // Calculate the byte offset of the inode within its block group
        // TODO: 1024 is the size of the superblock, but it is not in every block group
        let offset_within_block_group = (block_group.inode_table * self.block_size)
            + (index as u32 * self.superblock.inode_size as u32);

        let offset_from_partition_start =
            offset_within_block_group + group * self.superblock.blocks_per_group * self.block_size;

        DEBUG!(
            "Reading inode {} at offset {:#x}",
            inode_num,
            offset_from_partition_start
        );

        let inode: Inode = unsafe {
            *(DISK_IMG.as_ptr().add(offset_from_partition_start as usize) as *const Inode)
        };

        //DEBUG!("Inode read successfully: {:?}", inode);

        inode
    }

    pub fn debug_print_superblock(&self) {
        let inode_count = self.superblock.inode_count;
        let block_count = self.superblock.block_count;
        let blocks_per_group = self.superblock.blocks_per_group;
        let inodes_per_group = self.superblock.inodes_per_group;
        let magic = self.superblock.magic;

        kprintln!("Superblock info:");
        kprintln!("  Block size: {} bytes", self.block_size);
        kprintln!("  Inode count: {}", inode_count);
        kprintln!("  Block count: {}", block_count);
        kprintln!("  Blocks per group: {}", blocks_per_group);
        kprintln!("  Inodes per group: {}", inodes_per_group);
        kprintln!("  Magic: {:#x}", magic);
    }

    pub fn read_inode_by_path(&self, path: &str) -> Option<Inode> {
        let _event = core::hint::black_box(crate::instrument!());
        let mut current_inode = 2; // Root directory

        for component in path
            .trim_start_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
        {
            let dir_data = self.read_file(current_inode);
            let mut offset = 0;

            while offset < dir_data.len() {
                let entry: &DirectoryEntry =
                    unsafe { &*(dir_data[offset..].as_ptr() as *const DirectoryEntry) };
                let name = core::str::from_utf8(
                    &dir_data[offset + core::mem::size_of::<DirectoryEntry>()
                        ..offset
                            + core::mem::size_of::<DirectoryEntry>()
                            + entry.name_len as usize],
                )
                .ok()?;

                if name == component {
                    current_inode = entry.inode;

                    return Some(self.read_inode(current_inode));
                }
                offset += entry.rec_len as usize;
            }
        }

        None
    }

    fn read_file(&self, inode_num: u32) -> &[u8] {
        let inode = self.read_inode(inode_num);
        let block = self.read_block(inode.direct_blocks[0]);
        &block[..inode.size as usize]
    }
}

pub fn init_filesystem() {
    let _event = core::hint::black_box(crate::instrument!());
    let fs = Ext2FileSystem::new();
    fs.debug_print_superblock();

    /*match fs.read_files_first_block_by_path("/doom1.wad") {
        Some(block) => match core::str::from_utf8(block) {
            Ok(content) => DEBUG!("File content (1st block only): {}", content),
            Err(e) => {
                ERROR!("File content: Invalid UTF-8: {}", e);
                DEBUG!("File content as bytes: {:02x?}", block);
            }
        },
        None => {
            ERROR!("Failed to read file");
        }
    }*/
}
