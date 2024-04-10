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

#[derive(clap::ValueEnum, Clone, Copy, Default, Debug)]
pub enum SortBy {
    #[default]
    Code,
    Total,
    Language,
}

impl Display for SortBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            SortBy::Code => "code",
            SortBy::Language => "language",
            SortBy::Total => "total",
        };

        write!(f, "{}", name)
    }
}

#[derive(clap::Parser)]
struct Args {
    path: PathBuf,
    #[arg(short, long, default_value_t)]
    sort_by: SortBy,
    #[arg(short, long, default_value_t)]
    debug: bool,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

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
            let info = file_info_from_path(f.path(), args.debug)
                .await
                .with_context(|| format!("while getting file infos from {}", f.path().display()))?;
            anyhow::Ok(Some(info))
        })
        .boxed();

    let mut loc = FileInfo::default();
    let mut loc_by_lang = HashMap::<Language, FileInfo>::new();
    while let Some(next_file_info) = file_infos.next().await {
        match next_file_info {
            Ok(Some((file_info, language))) => {
                loc_by_lang
                    .entry(language)
                    .or_default()
                    .merge_with(&file_info);
                loc.merge_with(&file_info);
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

    match args.sort_by {
        SortBy::Language => rows.sort_by_key(|(key, _)| key.to_string()),
        SortBy::Code => {
            rows.sort_by(|(_, fileinfo1), (_, fileinfo2)| fileinfo2.code.cmp(&fileinfo1.code))
        }
        SortBy::Total => {
            rows.sort_by(|(_, fileinfo1), (_, fileinfo2)| fileinfo2.total.cmp(&fileinfo1.total))
        }
    };

    let rows_iter = rows
        .into_iter()
        .chain(std::iter::once((TableKey::Total, loc)));

    print!("{}", TableWrapper::new(rows_iter));
    Ok(())
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

impl Table for FileInfo {
    type Key = TableKey;
    fn describe() -> TableDescriptor<Self, Self::Key> {
        TableDescriptorBuilder::column_key("Language", |x: &TableKey| x)
            .column(
                "Code",
                |x: &FileInfo| {
                    if x.textual {
                        &x.code
                    } else {
                        &"-"
                    }
                },
            )
            .column(
                "Comments",
                |x: &FileInfo| {
                    if x.textual {
                        &x.code
                    } else {
                        &"-"
                    }
                },
            )
            .column(
                "Empty",
                |x: &FileInfo| {
                    if x.textual {
                        &x.code
                    } else {
                        &"-"
                    }
                },
            )
            .column(
                "Total",
                |x: &FileInfo| {
                    if x.textual {
                        &x.code
                    } else {
                        &"-"
                    }
                },
            )
            .column_with_format("File count", TableFormat::Right, |x: &FileInfo| {
                &x.file_count
            })
            .build()
    }
}
