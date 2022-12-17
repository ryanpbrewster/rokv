use std::{path::PathBuf, fs::File};

use rokv::sync_read::Writer;
use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command()]
struct Args {
    #[arg(long)]
    output: PathBuf,

    #[arg(long, default_value_t = 1_024)]
    count: usize,

    #[arg(long, default_value_t = 1_024)]
    value_size: usize,
}

fn main() -> anyhow::Result<()> {
    let args = Args::try_parse()?;

    let mut fout = File::create(args.output)?;
    let mut w = Writer::new(&mut fout)?;

    let mut buf = Vec::with_capacity(args.value_size);
    for i in 0 .. args.count {
        let key = format!("key-{:06}", i);
        let value = format!("value-{:06}", i);

        buf.clear();
        buf.extend_from_slice(value.as_bytes());
        buf.extend(std::iter::repeat(b'_').take(args.value_size - value.len()));
        w.append(key.as_bytes(), &buf)?;
    }
    w.finish()?;

    Ok(())
}