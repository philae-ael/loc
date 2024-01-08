use std::fmt::Display;

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

impl<U, V, It: Iterator<Item = (U, V)>> Display for TableWrapper<(U, V), It>
where
    V: Table<Key = U> + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let min_pad = 10;
        let out = std::cell::Cell::new(None);
        self.data.swap(&out);
        let Some(it) = out.into_inner() else {return Ok(())};
        let mut pads = vec![];

        for entry in &V::describe(None).v {
            write!(f, "| {: ^1$} ", entry.name, min_pad)?;
            pads.push(entry.name.len().max(min_pad));
        }
        writeln!(f, "|")?;

        // print seperator
        for entry in &pads {
            write!(f, "| {:-^1$} ", "", entry)?;
        }
        writeln!(f, "|")?;

        for x in it {
            // print the fucking rest
            for (entry, pad) in V::describe(Some(x.0)).v.into_iter().zip(&pads) {
                let d = format!("{}", entry.disp.call(&x.1));
                write!(f, "| ")?;
                match entry.format {
                    TableFormat::Center => write!(f, "{: ^1$}", d, pad)?,
                    TableFormat::Left => write!(f, "{: <1$}", d, pad)?,
                    TableFormat::Right => write!(f, "{: >1$}", d, pad)?,
                }
                write!(f, " ")?;
            }
            writeln!(f, "|")?;
        }
        Ok(())
    }
}

#[allow(dead_code)]
pub enum TableFormat {
    Center,
    Left,
    Right,
}

pub struct TableEntry<T> {
    name: &'static str,
    format: TableFormat,
    disp: Lens<T, Box<dyn Display>>,
}

pub struct TableDescriptor<T> {
    v: Vec<TableEntry<T>>,
}

pub struct TableDescriptorBuilder<T, Key> {
    key: Option<Key>,
    v: Vec<TableEntry<T>>,
}

struct Lens<T, S: ?Sized>(Box<dyn FnOnce(&T) -> S>);

impl<T: 'static, S> Lens<T, S> {
    pub fn call(self, t: &T) -> S {
        (self.0)(t)
    }
}

impl<T: 'static, Key: 'static> TableDescriptorBuilder<T, Key> {
    pub fn new(key: Option<Key>) -> Self {
        Self { key, v: Vec::new() }
    }

    pub fn column_with_format<F: Display + 'static>(
        mut self,
        name: &'static str,
        format: TableFormat,
        getter: impl (FnOnce(&T) -> F) + 'static,
    ) -> Self {
        self.v.push(TableEntry {
            name,
            format,
            disp: Lens(Box::new(|x| Box::new(getter(x)))),
        });
        self
    }

    pub fn column<F: Display + 'static>(
        self,
        name: &'static str,
        getter: impl (FnOnce(&T) -> F) + 'static,
    ) -> Self {
        self.column_with_format(name, TableFormat::Left, getter)
    }

    pub fn column_key<F: Display>(
        self,
        name: &'static str,
        getter: impl (FnOnce(&Key) -> F) + 'static,
    ) -> Self {
        match &self.key {
            Some(x) => {
                let out = format!("{}", getter(x));
                self.column(name, move |_| out)
            }
            None => self.column(name, move |_| name),
        }
    }

    // pub fn then<U: Table + 'static>(mut self, f: impl (Fn(&T) -> &U) + Copy + 'static) -> Self {
    //     for entry in U::describe().v {
    //         self.v.push(TableEntry {
    //             name: entry.name,
    //             format: entry.format,
    //             disp: entry.disp.compose(f),
    //         });
    //     }
    //
    //     self
    // }
    pub fn build(self) -> TableDescriptor<T> {
        TableDescriptor { v: self.v }
    }
}

pub trait Table: Sized {
    type Key;
    fn describe(x: Option<Self::Key>) -> TableDescriptor<Self>;
}
