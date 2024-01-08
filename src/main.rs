mod file_info;
mod identify;
mod language;
mod line_kind;
mod table;
mod walker;

use std::{collections::HashMap, path::PathBuf};

use clap::Parser;
use futures::StreamExt;
use table::{Table, TableDescriptor, TableFormat};

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

    let mut rows: Vec<_> = loc_by_lang
        .into_iter()
        .map(|(x, y)| (x.to_string(), y))
        .collect();
    rows.sort_by(|x, y| x.0.cmp(&y.0));

    let rows_iter = rows
        .into_iter()
        .chain(std::iter::once(("Total".into(), loc)));

    println!("{}", TableWrapper::new(rows_iter));
    Ok(())
}

impl Table for (String, FileInfo) {
    fn describe() -> TableDescriptor<Self> {
        TableDescriptor::new()
            .column_with_format("String", TableFormat::Center, |x: &(String, FileInfo)| &x.0)
            .then(|x: &(String, FileInfo)| &x.1)
    }
}

impl Table for FileInfo {
    fn describe() -> TableDescriptor<Self> {
        TableDescriptor::new()
            .column("Code", |x: &FileInfo| &x.code)
            .column("Comments", |x: &FileInfo| &x.comments)
            .column("Empty", |x: &FileInfo| &x.comments)
            .column("Total", |x: &FileInfo| &x.code)
            .column_with_format("File count", TableFormat::Right, |x: &FileInfo| {
                &x.file_count
            })
    }
}
