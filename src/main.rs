use std::process::ExitCode;

use std::io;

use std::io::BufWriter;
use std::io::Write;

use rs_tar2tar::tar::filter::FilterResult;
use rs_tar2tar::tar::filter::SimpleFilter;
use rs_tar2tar::tar::header::Header;

use rs_tar2tar::tar::alex::tar2items::tar2items2filtered2tar;

fn env_val_by_key(key: &'static str) -> Result<Vec<u8>, io::Error> {
    std::env::var_os(key)
        .map(|o| o.into_encoded_bytes())
        .ok_or_else(|| io::Error::other(format!("the env var {key} missing")))
}

fn simple_filter_suffix() -> Option<Vec<u8>> {
    env_val_by_key("ENV_SIMPLE_FILTER_SUFFIX").ok()
}

fn simple_filter_prefix() -> Option<Vec<u8>> {
    env_val_by_key("ENV_SIMPLE_FILTER_PREFIX").ok()
}

fn simple_filter(keep: bool) -> SimpleFilter {
    let so: Option<Vec<u8>> = simple_filter_suffix();
    let po: Option<Vec<u8>> = simple_filter_prefix();

    match so {
        None => match po {
            None => SimpleFilter::KeepAll,
            Some(prefix) => SimpleFilter::Prefix(prefix, keep),
        },
        Some(suffix) => match po {
            None => SimpleFilter::Suffix(suffix, keep),
            Some(prefix) => SimpleFilter::SufPrefixEither(suffix, prefix, keep),
        },
    }
}

fn filter(keep: bool) -> impl Fn(&Header) -> FilterResult {
    simple_filter(keep).into_filter()
}

fn keep() -> bool {
    env_val_by_key("ENV_KEEP")
        .ok()
        .and_then(|b| {
            std::str::from_utf8(&b)
                .ok()
                .and_then(|s| str::parse(s).ok())
        })
        .unwrap_or(true)
}

fn verbose() -> bool {
    env_val_by_key("ENV_VERBOSE")
        .ok()
        .and_then(|b| {
            std::str::from_utf8(&b)
                .ok()
                .and_then(|s| str::parse(s).ok())
        })
        .unwrap_or(true)
}

fn stdin2tar2filtered2stdout() -> Result<(), io::Error> {
    let i = io::stdin();
    let il = i.lock();

    let o = io::stdout();
    let mut ol = o.lock();

    tar2items2filtered2tar(il, filter(keep()), BufWriter::new(&mut ol), verbose())?;

    ol.flush()
}

fn sub() -> Result<(), io::Error> {
    stdin2tar2filtered2stdout()
}

fn main() -> ExitCode {
    sub().map(|_| ExitCode::SUCCESS).unwrap_or_else(|e| {
        eprintln!("{e}");
        ExitCode::FAILURE
    })
}
