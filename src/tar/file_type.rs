#[derive(Clone, Copy)]
pub enum FileType {
    Unspecified,
    Regular,
    Link,
    Symlink,
    Char,
    Block,
    Directory,
    Fifo,
    Raw(u8),
}
