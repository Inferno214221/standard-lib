use super::FileType;

pub struct Metadata {
    pub size: i64,             // st_size
    pub file_type: FileType,   // st_mode
    pub mode: u16,             // st_mode
    pub uid: u32,              // st_uid
    pub gid: u32,              // st_gid
    pub parent_device_id: u64, // st_dev
    pub self_device_id: u64,   // st_rdev
    // x86_64:
    pub time_accessed: (i64, i64), // st_atime, st_atime_nsec
    pub time_modified: (i64, i64), // st_mtime, st_mtime_nsec
    pub time_changed: (i64, i64),  // st_ctime, st_ctime_nsec
    pub links: u64,                // st_nlink
    pub block_size: i64,           // st_blksize
    // 64-bit:
    pub blocks: i64,    // st_blocks
    pub inode_num: u64, // st_ino
}