mod file_info;
mod identify;
mod language;
mod line_kind;
mod table;

use std::{collections::HashMap, fmt::Display, path::PathBuf};

use crate::{
    file_info::{file_info_from_path, FileInfo},
    language::Language,
    table::TableWrapper,
};
use anyhow::Context;
use clap::Parser;
use futures::StreamExt;
use table::{Table, TableDescriptor, TableDescriptorBuilder, TableFormat};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{
    fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt, Layer,
};

#[derive(clap::ValueEnum, Clone, Copy, Default, Debug)]
pub enum SortKey {
    #[default]
    Code,
    Total,
    Language,
    File,
}

impl Display for SortKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            SortKey::Code => "code",
            SortKey::Language => "language",
            SortKey::Total => "total",
            SortKey::File => "file",
        };

        write!(f, "{}", name)
    }
}

#[derive(clap::ValueEnum, Clone, Copy, Default, Debug)]
pub enum Mode {
    #[default]
    Language,
    File,
}
impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Mode::Language => "language",
            Mode::File => "file",
        };

        write!(f, "{}", name)
    }
}

#[derive(clap::Parser)]
struct Args {
    path: PathBuf,
    #[arg(short, long, default_value_t)]
    sort: SortKey,
    #[arg(short, long, default_value_t)]
    debug: bool,

    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[arg(short, long, default_value_t)]
    mode: Mode,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let reg = tracing_subscriber::registry();

    let filter = tracing_subscriber::filter::EnvFilter::builder()
        .with_default_directive(LevelFilter::WARN.into())
        .from_env_lossy();

    let filter = match args.verbose {
        0 => filter,
        1 => filter.add_directive(LevelFilter::INFO.into()),
        2 => filter.add_directive(LevelFilter::DEBUG.into()),
        _ => filter.add_directive(LevelFilter::TRACE.into()),
    };

    let reg = reg.with(
        tracing_subscriber::fmt::layer()
            .with_timer(tracing_subscriber::fmt::time::uptime())
            .with_span_events(FmtSpan::CLOSE)
            .with_filter(filter),
    );

    reg.init();

    tracing::debug!("Starting to walk the directory...");
    tracing::debug!("Using path: {}", args.path.display());

    let walker = ignore::WalkBuilder::new(args.path)
        .hidden(true)
        // .filter_entry(|x| !x.path().is_dir())
        .build();

    let mut file_infos = futures::stream::iter(walker)
        .then(|file| async {
            let f = file?;
            if f.path().is_dir() {
                return Ok(None);
            }
            let (i, p) = file_info_from_path(f.path(), args.debug)
                .await
                .with_context(|| format!("while getting file infos from {}", f.path().display()))?;
            anyhow::Ok(Some((f.path().to_path_buf(), i, p)))
        })
        .boxed();

    match args.mode {
        Mode::Language => {
            let mut loc_total = FileInfo::default();
            let mut loc_by_lang = HashMap::<Language, FileInfo>::new();
            while let Some(next_file_info) = file_infos.next().await {
                match next_file_info {
                    Ok(Some((_, file_info, language))) => {
                        loc_by_lang
                            .entry(language)
                            .or_default()
                            .merge_with(&file_info);
                        loc_total.merge_with(&file_info);
                    }
                    Ok(None) => (),
                    Err(err) => {
                        println!("ERROR! {err:#}");
                    }
                }
            }

            let mut rows: Vec<_> = loc_by_lang
                .into_iter()
                .map(|(x, y)| (TableKey::Language(x), y))
                .collect();

            match args.sort {
                SortKey::Language => rows.sort_by_key(|(key, _)| key.to_string()),
                SortKey::Code => rows.sort_by_key(|fileinfo| -(fileinfo.1.code as isize)),
                SortKey::Total => rows.sort_by(|(_, fileinfo1), (_, fileinfo2)| {
                    fileinfo2.total.cmp(&fileinfo1.total)
                }),
                SortKey::File => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Cannot sort by file when mode is language",
                    ))
                }
            };

            let rows_iter = rows
                .into_iter()
                .chain(std::iter::once((TableKey::Total, loc_total)));
            println!("{}", TableWrapper::new::<TableByLanguage>(rows_iter));
        }
        Mode::File => {
            let mut loc_total = FileInfo::default();
            let mut loc_per_file = HashMap::<String, (FileInfo, Language)>::new();
            while let Some(next_file_info) = file_infos.next().await {
                match next_file_info {
                    Ok(Some((path, file_info, language))) => {
                        loc_total.merge_with(&file_info);
                        loc_per_file.insert(path.display().to_string(), (file_info, language));
                    }
                    Ok(None) => (),
                    Err(err) => {
                        println!("ERROR! {err:#}");
                    }
                }
            }
            let mut rows: Vec<_> = loc_per_file
                .into_iter()
                .map(|(x, y)| (TableFileKey::Path(x), y))
                .collect();
            match args.sort {
                SortKey::Language => rows.sort_by_key(|(_, (_, lang))| lang.to_string()),
                SortKey::Code => rows.sort_by_key(|(_, (fileinfo, _))| -(fileinfo.code as isize)),
                SortKey::Total => rows.sort_by(|(_, (fileinfo1, _)), (_, (fileinfo2, _))| {
                    fileinfo2.total.cmp(&fileinfo1.total)
                }),
                SortKey::File => rows.sort_by_key(|(key, _)| key.to_string()),
            };

            println!("{}", TableWrapper::new::<TableFile>(rows.into_iter()));
        }
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub enum TableFileKey {
    Path(String),
    Total,
}
impl Display for TableFileKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TableFileKey::Path(p) => write!(f, "{}", p),
            TableFileKey::Total => write!(f, "Total"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TableKey {
    Language(Language),
    Total,
}

impl Display for TableKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TableKey::Language(l) => l.fmt(f),
            TableKey::Total => write!(f, "Total"),
        }
    }
}
fn get_or_default<'a>(fi: &'a FileInfo, val: &'a usize) -> &'a dyn Display {
    if fi.textual {
        val
    } else {
        &"-"
    }
}
struct TableFile;
impl Table for TableFile {
    type Key = TableFileKey;
    type Value = (FileInfo, Language);
    fn describe() -> TableDescriptor<Self::Value, Self::Key> {
        TableDescriptorBuilder::column_key_with_format(
            "File",
            TableFormat::Left,
            |x: &TableFileKey| x,
        )
        .column("Code", |(x, _): &(FileInfo, Language)| {
            get_or_default(x, &x.code)
        })
        .column("Comments", |(x, _): &(FileInfo, Language)| {
            get_or_default(x, &x.comments)
        })
        .column("Empty", |(x, _): &(FileInfo, Language)| {
            get_or_default(x, &x.empty)
        })
        .column("Total", |(x, _): &(FileInfo, Language)| {
            get_or_default(x, &x.total)
        })
        .column_with_format(
            "Language",
            TableFormat::Left,
            |(_, l): &(FileInfo, Language)| l,
        )
        .build()
    }
}

struct TableByLanguage;
impl Table for TableByLanguage {
    type Key = TableKey;
    type Value = FileInfo;
    fn describe() -> TableDescriptor<Self::Value, Self::Key> {
        TableDescriptorBuilder::column_key("Language", |x: &TableKey| x)
            .column("Code", |x: &FileInfo| get_or_default(x, &x.code))
            .column("Comments", |x: &FileInfo| get_or_default(x, &x.comments))
            .column("Empty", |x: &FileInfo| get_or_default(x, &x.empty))
            .column("Total", |x: &FileInfo| get_or_default(x, &x.total))
            .column_with_format("File count", TableFormat::Right, |x: &FileInfo| {
                &x.file_count
            })
            .build()
    }
}
