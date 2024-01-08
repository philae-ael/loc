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

impl<T: Table + 'static, It: Iterator<Item = T>> Display for TableWrapper<T, It> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let min_pad = 10;
        let out = std::cell::Cell::new(None);
        self.data.swap(&out);
        let Some(it) = out.into_inner() else {return Ok(())};
        let mut pads = vec![];

        let descriptor = T::describe();

        for entry in &descriptor.v {
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
            for (entry, pad) in descriptor.v.iter().zip(&pads) {
                let d = format!("{}", entry.disp.call(&x));
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
    disp: Lens<T, dyn Display>,
}

#[derive(Default)]
pub struct TableDescriptor<T> {
    v: Vec<TableEntry<T>>,
}

struct Lens<T, S: ?Sized>(Box<dyn Fn(&T) -> &S>);

impl<T: 'static, S: 'static + ?Sized> Lens<T, S> {
    pub fn new(f: impl (Fn(&T) -> &S) + 'static) -> Self {
        Self(Box::new(f))
    }

    pub fn compose<A: 'static>(self, f: impl (for<'a> Fn(&'a A) -> &'a T) + 'static) -> Lens<A, S> {
        Lens::new(move |x| self.call(f(x)))
    }

    pub fn call<'a>(&self, t: &'a T) -> &'a S {
        (self.0)(t)
    }
}

impl<T: 'static> TableDescriptor<T> {
    pub fn new() -> Self {
        Self { v: Vec::new() }
    }

    pub fn column_with_format(
        mut self,
        name: &'static str,
        format: TableFormat,
        getter: impl (Fn(&T) -> &(dyn Display + 'static)) + 'static,
    ) -> Self {
        self.v.push(TableEntry {
            name,
            format,
            disp: Lens(Box::new(getter)),
        });
        self
    }

    pub fn column(
        self,
        name: &'static str,
        getter: impl (Fn(&T) -> &(dyn Display + 'static)) + 'static,
    ) -> Self {
        self.column_with_format(name, TableFormat::Left, getter)
    }

    pub fn then<U: Table + 'static>(mut self, f: impl (Fn(&T) -> &U) + Copy + 'static) -> Self {
        for entry in U::describe().v {
            self.v.push(TableEntry {
                name: entry.name,
                format: entry.format,
                disp: entry.disp.compose(f),
            });
        }

        self
    }
}

pub trait Table: Sized {
    fn describe() -> TableDescriptor<Self>;
}
