use core::time::Duration;

use std::borrow::Cow;

use crate::tar::file_type::FileType;

pub struct Header<'a> {
    pub file_type: FileType,

    pub path_name: Cow<'a, [u8]>,
    pub link_name: Option<Cow<'a, [u8]>>,

    pub size: u64,
    pub entry_size: u64,

    pub mode: u64,

    pub uid: u64,
    pub gid: u64,

    pub uname: Option<&'a [u8]>,
    pub gname: Option<&'a [u8]>,

    pub modified: Duration,

    pub dev_major: Option<u64>,
    pub dev_minor: Option<u64>,
}
