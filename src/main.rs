mod file_info;
mod identify;
mod language;
mod walker;
mod line_kind;

use std::{collections::HashMap, path::PathBuf};

use clap::Parser;
use futures::StreamExt;

use crate::{file_info::{FileInfo, file_info_from_path}, language::Language, walker::Walker};

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
        .then(file_info_from_path)
        .boxed(); // Dont understand pinning

    while let Some(file_info) = file_infos.next().await {
        match file_info {
            Ok((file_info, language)) => {
                loc_by_lang
                    .entry(language)
                    .or_default()
                    .merge_with(&file_info);
                loc.merge_with(&file_info);
            }
            Err(err) => {
                println!("ERROR {err}");
            }
        }
    }

    println!("{loc:?}");
    println!("{loc_by_lang:?}");
    Ok(())
}
