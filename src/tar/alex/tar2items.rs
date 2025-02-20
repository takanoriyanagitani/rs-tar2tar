use core::time::Duration;

use std::io;

use std::io::Read;

use std::io::Write;

use tar::Archive;
use tar::Builder;
use tar::Entry;
use tar::EntryType;

use crate::tar::file_type::FileType;
use crate::tar::filter::FilterResult;
use crate::tar::header::Header;

pub trait FileTypeLike {
    fn to_file_type(&self) -> FileType;
}

impl FileTypeLike for EntryType {
    fn to_file_type(&self) -> FileType {
        match self {
            EntryType::Regular => FileType::Regular,
            EntryType::Link => FileType::Link,
            EntryType::Symlink => FileType::Symlink,
            EntryType::Char => FileType::Char,
            EntryType::Block => FileType::Block,
            EntryType::Directory => FileType::Directory,
            EntryType::Fifo => FileType::Fifo,
            _ => FileType::Raw(self.as_byte()),
        }
    }
}

pub trait HeaderLike {
    fn to_header(&self, verbose: bool) -> Result<Header, io::Error>;
}

impl<R> HeaderLike for Entry<'_, R>
where
    R: Read,
{
    fn to_header(&self, verbose: bool) -> Result<Header, io::Error> {
        let hdr = Header {
            file_type: self.header().entry_type().to_file_type(),
            path_name: self.path_bytes(),

            link_name: self.link_name_bytes(),

            size: self.size(),
            entry_size: self.header().entry_size()?,

            mode: self.header().mode()?.into(),

            uid: self.header().uid()?,
            gid: self.header().gid()?,

            uname: self.header().username_bytes(),
            gname: self.header().groupname_bytes(),

            modified: Duration::from_secs(self.header().mtime()?),

            dev_major: self
                .header()
                .device_major()
                .unwrap_or_else(|e| {
                    if verbose {
                        eprintln!("{e}")
                    }
                    None
                })
                .map(|u| u.into()),
            dev_minor: self
                .header()
                .device_minor()
                .unwrap_or_else(|e| {
                    if verbose {
                        eprintln!("{e}");
                    }
                    None
                })
                .map(|u| u.into()),
        };
        Ok(hdr)
    }
}

pub fn tar2items2filtered2tar<R, F, W>(
    input: R,
    header_filter: F,
    output: W,
    verbose: bool,
) -> Result<(), io::Error>
where
    R: Read,
    W: Write,
    F: Fn(&Header) -> FilterResult,
{
    let mut wtr: Builder<W> = Builder::new(output);

    let mut rdr = Archive::new(input);
    let items = rdr.entries()?;

    let mut hbuf: [u8; 512] = [0; 512];

    for ritem in items {
        let item: Entry<_> = ritem?;
        let hdr: Header = item.to_header(verbose)?;
        let fres: FilterResult = header_filter(&hdr);
        if FilterResult::Keep == fres {
            let thdr: &tar::Header = item.header();
            let thdr_raw: &[u8; 512] = thdr.as_bytes();
            hbuf.copy_from_slice(thdr_raw);
            let thdr: &tar::Header = tar::Header::from_byte_slice(&hbuf);
            wtr.append(thdr, item)?;
        }
    }

    let mut output: W = wtr.into_inner()?;
    output.flush()
}
