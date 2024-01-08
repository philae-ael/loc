mod file_info;
mod identify;
mod language;
mod line_kind;
mod table;
mod walker;

use std::{collections::HashMap, path::PathBuf, fmt::Display};

use clap::Parser;
use futures::StreamExt;
use table::{Table, TableDescriptor, TableDescriptorBuilder, TableFormat};

use crate::{
    file_info::{file_info_from_path, FileInfo},
    language::Language,
    table::TableWrapper,
    walker::Walker,
};

#[derive(clap::Parser)]
struct Args {
    path: PathBuf,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let mut loc_by_lang = HashMap::<Language, FileInfo>::new();
    let mut loc = FileInfo::default();
    let walker = Walker::new(args.path)?;

    let mut file_infos = futures::stream::iter(walker)
        .then(|file| async { (file.clone(), file_info_from_path(file).await) })
        .boxed(); // Dont understand pinning

    while let Some((file, file_info)) = file_infos.next().await {
        match file_info {
            Ok((file_info, language)) => {
                loc_by_lang
                    .entry(language)
                    .or_default()
                    .merge_with(&file_info);
                loc.merge_with(&file_info);
            }
            Err(err) => {
                println!("ERROR for file {}: {err}", file.display());
            }
        }
    }

    let rows: Vec<_> = loc_by_lang
        .into_iter()
        .map(|(x, y)| (TableKey::Language(x), y))
        .collect();

    let rows_iter = rows
        .into_iter()
        .chain(std::iter::once((TableKey::Total, loc)));

    println!("{}", TableWrapper::new(rows_iter));
    Ok(())
}

#[derive(Debug)]
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
    fn describe(x: Option<Self::Key>) -> TableDescriptor<Self> {
        TableDescriptorBuilder::new(x)
            .column_key("Language", |x: &TableKey| format!("{x}"))
            .column("Code", |x: &FileInfo| x.code)
            .column("Comments", |x: &FileInfo| x.comments)
            .column("Empty", |x: &FileInfo| x.comments)
            .column("Total", |x: &FileInfo| x.code)
            .column_with_format("File count", TableFormat::Right, |x: &FileInfo| {
                x.file_count
            })
            .build()
    }
}
