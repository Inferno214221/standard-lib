// TODO: Ensure that only Files are supported? Prefereably while keeping Metadata lazy - Metadata
// has a size of 120 bytes.
#[derive(Debug)]
pub enum FileType {
    BlockDevice,
    CharDevice,
    Directory,
    FIFO,
    Symlink,
    Regular,
    Socket,
    Unknown,
}