// TODO: Ensure that only Files are supported? Preferably while keeping Metadata lazy - Metadata
// has a size of 120 bytes.
#[derive(Debug)]
pub enum FileType {
    BlockDevice,
    CharDevice,
    Directory,
    Fifo,
    Symlink,
    Regular,
    Socket,
    Unknown,
}