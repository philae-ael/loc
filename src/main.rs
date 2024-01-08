mod file_info;
mod identify;
mod language;
mod line_kind;
mod walker;

use std::{collections::HashMap, fmt::Display, path::PathBuf};

use clap::Parser;
use futures::StreamExt;

use crate::{
    file_info::{file_info_from_path, FileInfo},
    language::Language,
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

    let rows_iter = loc_by_lang
        .into_iter()
        .map(|(x, y)| (x.to_string(), y))
        .chain(std::iter::once(("Total".into(), loc)));

    println!("{}", TableWrapper::new(rows_iter));
    Ok(())
}

// Display tables, the wanky way

pub struct TableWrapper<T, It: Iterator<Item = T>> {
    data: std::cell::Cell<Option<It>>,
}

impl<T, It: Iterator<Item = T>> TableWrapper<T, It> {
    pub fn new(data: It) -> Self {
        Self {
            data: std::cell::Cell::new(Some(data)),
        }
    }
}

impl<T: Table, It: Iterator<Item = T>> Display for TableWrapper<T, It> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = std::cell::Cell::new(None);
        self.data.swap(&out);
        let Some(mut it) = out.into_inner() else {return Ok(())};
        let mut arr = vec![];

        let it = if let Some(x) = it.next() {
            // print header
            for entry in x.describe().v {
                write!(f, "| {} ", entry.name)?;
                arr.push(entry.name.len());
            }
            writeln!(f, "|")?;
            std::iter::once(x).chain(it)
        } else {
            // No entry
            return Ok(());
        };

        // print seperator
        for entry in &arr {
            write!(f, "| {:-^1$} ", "", entry)?;
        }
        writeln!(f, "|")?;

        for x in it {
            // print the fucking rest
            for (entry, pad) in x.describe().v.iter().zip(&arr) {
                write!(f, "| {: ^1$} ", format!("{}", entry.disp), pad)?;
            }
            writeln!(f, "|")?;
        }
        Ok(())
    }
}

pub struct TableEntry<'a> {
    name: &'static str,
    disp: &'a dyn Display,
}

#[derive(Default)]
pub struct TableDescriptor<'a> {
    v: Vec<TableEntry<'a>>,
}

impl<'a> TableDescriptor<'a> {
    pub fn new() -> Self {
        Self { v: Vec::new() }
    }

    pub fn column(mut self, name: &'static str, d: &'a dyn Display) -> Self {
        self.v.push(TableEntry { name, disp: d });
        self
    }

    pub fn then<U: Table>(mut self, u: &'a U) -> Self {
        let u_descriptor = U::describe(u);
        for entry in u_descriptor.v {
            self.v.push(entry)

        }

        self
    }
}

impl<T: Table> Table for &T {
    fn describe(&self) -> TableDescriptor {
        TableDescriptor::new().then(*self)
    }
}

pub trait Table: Sized {
    fn describe(&self) -> TableDescriptor;
}

impl Table for (String, FileInfo) {
    fn describe(&self) -> TableDescriptor {
        TableDescriptor::new()
            .column("Language", &self.0)
            .then(&self.1)
    }
}

impl Table for FileInfo {
    fn describe(&self) -> TableDescriptor {
        TableDescriptor::new()
            .column("Code", &self.code)
            .column("Total", &self.total)
            .column("Comment", &self.comments)
            .column("Empty", &self.empty)
    }
}
