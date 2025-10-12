#[derive(Debug, Clone, PartialEq)]
pub enum FileType {
    BlockDevice,
    CharDevice,
    Directory,
    Fifo,
    Symlink,
    Regular,
    Socket,
    Other,
}

use FileType::*;

impl FileType {
    #[inline(always)]
    pub(crate) const fn from_stat_mode(st_mode: u32) -> FileType {
        match st_mode & libc::S_IFMT {
            libc::S_IFBLK => BlockDevice,
            libc::S_IFCHR => CharDevice,
            libc::S_IFDIR => Directory,
            libc::S_IFIFO => Fifo,
            libc::S_IFLNK => Symlink,
            libc::S_IFREG => Regular,
            libc::S_IFSOCK => Socket,
            _ => Other,
        }
    }

    pub(crate) fn from_dirent_type(d_type: u8) -> Option<FileType> {
        Some(match d_type {
            libc::DT_BLK => BlockDevice,
            libc::DT_CHR => CharDevice,
            libc::DT_DIR => Directory,
            libc::DT_FIFO => Fifo,
            libc::DT_LNK => Symlink,
            libc::DT_REG => Regular,
            libc::DT_SOCK => Socket,
            libc::DT_UNKNOWN => None?,
            _ => Other,
        })
    }
}