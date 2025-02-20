use crate::tar::header::Header;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FilterResult {
    Keep = 0,
    Ignore = 1,
}

impl FilterResult {
    pub fn and(self, other: Self) -> Self {
        let s: u8 = self as u8;
        let o: u8 = other as u8;
        let a: u8 = s & o & 1;
        match a {
            0 => Self::Keep,
            1 => Self::Ignore,
            _ => Self::Keep,
        }
    }

    pub fn or(self, other: Self) -> Self {
        let s: u8 = self as u8;
        let o: u8 = other as u8;
        let a: u8 = (s | o) & 1;
        match a {
            0 => Self::Keep,
            1 => Self::Ignore,
            _ => Self::Keep,
        }
    }
}

pub fn nop_filter(_: &Header) -> FilterResult {
    FilterResult::Keep
}

pub fn filter_and<F, G>(f: F, g: G) -> impl Fn(&Header) -> FilterResult
where
    F: Fn(&Header) -> FilterResult,
    G: Fn(&Header) -> FilterResult,
{
    move |hdr: &Header| {
        let fr: FilterResult = f(hdr);
        let gr: FilterResult = g(hdr);
        fr.and(gr)
    }
}

pub fn filter_or<F, G>(f: F, g: G) -> impl Fn(&Header) -> FilterResult
where
    F: Fn(&Header) -> FilterResult,
    G: Fn(&Header) -> FilterResult,
{
    move |hdr: &Header| {
        let fr: FilterResult = f(hdr);
        let gr: FilterResult = g(hdr);
        fr.or(gr)
    }
}

pub fn path_filter_or<F, G>(f: F, g: G) -> impl Fn(&[u8]) -> FilterResult
where
    F: Fn(&[u8]) -> FilterResult,
    G: Fn(&[u8]) -> FilterResult,
{
    move |val: &[u8]| {
        let fr: FilterResult = f(val);
        let gr: FilterResult = g(val);
        fr.or(gr)
    }
}

pub fn path_bytes_filter_new<F>(pfilt: F) -> impl Fn(&Header) -> FilterResult
where
    F: Fn(&[u8]) -> FilterResult,
{
    move |hdr: &Header| {
        let path_bytes: &[u8] = &hdr.path_name;
        pfilt(path_bytes)
    }
}

/// Creates a filter using the suffix.
///
/// | suffix found | keep     | result | example                      |
/// |:------------:|:--------:|:------:|:----------------------------:|
/// | false        | exclude  | keep   | suffix: .jpg, name: helo.txt |
/// | false        | include  | ignore | suffix: .txt, name: helo.jpg |
/// | true         | exclude  | ignore | suffix: .jpg, name: helo.jpg |
/// | true         | include  | keep   | suffix: .txt, name: helo.txt |
pub fn suffix_filter_new(suffix: Vec<u8>, keep: bool) -> impl Fn(&[u8]) -> FilterResult {
    move |path_name: &[u8]| {
        let found: bool = path_name.ends_with(&suffix);

        let ignore: bool = found ^ keep;

        match ignore {
            true => FilterResult::Ignore,
            false => FilterResult::Keep,
        }
    }
}

pub fn prefix_filter_new(prefix: Vec<u8>, keep: bool) -> impl Fn(&[u8]) -> FilterResult {
    move |path_name: &[u8]| {
        let found: bool = path_name.starts_with(&prefix);

        let ignore: bool = found ^ keep;

        match ignore {
            true => FilterResult::Ignore,
            false => FilterResult::Keep,
        }
    }
}

pub enum SimpleFilter {
    KeepAll,
    Suffix(Vec<u8>, bool),
    Prefix(Vec<u8>, bool),
    SufPrefixEither(Vec<u8>, Vec<u8>, bool),
}

pub type PathFilter = dyn Fn(&[u8]) -> FilterResult;

impl SimpleFilter {
    pub fn into_path_filter(self) -> Box<PathFilter> {
        match self {
            Self::KeepAll => Box::new(|_: &[u8]| FilterResult::Keep),
            Self::Suffix(val, keep) => Box::new(suffix_filter_new(val, keep)),
            Self::Prefix(val, keep) => Box::new(prefix_filter_new(val, keep)),
            Self::SufPrefixEither(suffix, prefix, keep) => {
                let sf = suffix_filter_new(suffix, keep);
                let pf = prefix_filter_new(prefix, keep);
                Box::new(path_filter_or(sf, pf))
            }
        }
    }

    pub fn into_filter(self) -> impl Fn(&Header) -> FilterResult {
        let bpf: Box<PathFilter> = self.into_path_filter();
        path_bytes_filter_new(bpf)
    }
}

#[cfg(test)]
mod test_filter {
    mod suffix_filter_new {
        use crate::tar::filter;
        use crate::tar::filter::FilterResult;

        #[test]
        fn ignore_jpg_files() {
            let suffix = b".jpg";
            let keep: bool = false;
            let filter = filter::suffix_filter_new(suffix.into(), keep);

            assert_eq!(FilterResult::Keep, filter(b"helo.txt"));
            assert_eq!(FilterResult::Keep, filter(b"wrld.txt"));
            assert_eq!(FilterResult::Keep, filter(b"wrld.png"));
            assert_eq!(FilterResult::Ignore, filter(b"helo.jpg"));
            assert_eq!(FilterResult::Ignore, filter(b"wrld.jpg"));
        }

        #[test]
        fn process_txt_files() {
            let suffix = b".txt";
            let keep: bool = true;
            let filter = filter::suffix_filter_new(suffix.into(), keep);

            assert_eq!(FilterResult::Keep, filter(b"helo.txt"));
            assert_eq!(FilterResult::Keep, filter(b"wrld.txt"));
            assert_eq!(FilterResult::Ignore, filter(b"helo.png"));
            assert_eq!(FilterResult::Ignore, filter(b"helo.jpg"));
            assert_eq!(FilterResult::Ignore, filter(b"wrld.jpg"));
        }
    }
}
