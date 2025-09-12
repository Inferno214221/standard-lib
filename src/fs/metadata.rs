use libc::stat;

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

impl Metadata {
    #[allow(clippy::unnecessary_cast)]
    pub(crate) const fn from_stat(raw: stat) -> Metadata {
        Metadata {
            size: raw.st_size,
            file_type: FileType::from_stat_mode(raw.st_mode),
            mode: raw.st_mode as u16,
            uid: raw.st_uid,
            gid: raw.st_gid,
            parent_device_id: raw.st_dev,
            self_device_id: raw.st_rdev,
            time_accessed: (raw.st_atime as i64, raw.st_atime_nsec),
            time_modified: (raw.st_mtime as i64, raw.st_mtime_nsec),
            time_changed: (raw.st_ctime as i64, raw.st_ctime_nsec),
            links: raw.st_nlink as u64,
            block_size: raw.st_blksize as i64,
            blocks: raw.st_blocks as i64,
            inode_num: raw.st_ino as u64,
        }
    }
}