use std::fmt::Display;

const BOX_VERT: &str = "│";
const BOX_HORIZONTAL: &str = "─";
const BOX_CROSS_LEFT: &str = "├─";
const BOX_CROSS: &str = "─┼─";
const BOX_CROSS_RIGHT: &str = "─┤";

const BOX_CROSS_LEFT_DOWN: &str = "┌─";
const BOX_CROSS_DOWN: &str = "─┬─";
const BOX_CROSS_RIGHT_DOWN: &str = "─┐";

const BOX_CROSS_LEFT_UP: &str = "└─";
const BOX_CROSS_UP: &str = "─┴─";
const BOX_CROSS_RIGHT_UP: &str = "─┘";

// Display tables, the wanky way
pub struct TableWrapper<Tbl, T, It: Iterator<Item = T>> {
    data: std::cell::Cell<Option<It>>,
    phantom: std::marker::PhantomData<(Tbl, T)>,
}

impl<T, It: Iterator<Item = T>> TableWrapper<(), T, It> {
    pub fn new<Tbl>(data: It) -> TableWrapper<Tbl, T, It> {
        TableWrapper {
            data: std::cell::Cell::new(Some(data)),
            phantom: std::marker::PhantomData,
        }
    }
}

impl<Tbl, U, V, It: Iterator<Item = (U, V)>> Display for TableWrapper<Tbl, (U, V), It>
where
    Tbl: Table<Key = U, Value = V>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn line_sep(
            pads: &[usize],
            box_horizontal: &str,
            box_left: &str,
            box_mid: &str,
            box_right: &str,
        ) -> String {
            let total: usize = pads.iter().sum::<usize>() - pads.len() + 1;
            let mut buf: Vec<&str> = vec![box_horizontal; total];

            let mut sum = 0;
            buf[0] = box_left;
            for s in pads {
                sum += s - 1;
                buf[sum] = box_mid;
            }
            buf[sum] = box_right;

            buf.into_iter().collect::<String>()
        }

        fn write_col(
            f: &mut std::fmt::Formatter<'_>,
            format: TableFormat,
            col: &str,
            pad: usize,
        ) -> std::fmt::Result {
            write!(f, "{BOX_VERT}")?;
            match format {
                TableFormat::Center => write!(f, "{: ^1$}", col, pad),
                // When left padding and there is enough space, add an extra space to the left to
                // make it look nicer
                // spoiler: there is always enough space because of the way we calculate the padding
                TableFormat::Left => {
                    if pad > col.len() + 1 {
                        write!(f, " {: <1$}", col, pad - 1)
                    } else {
                        write!(f, "{: <1$}", col, pad)
                    }
                }
                TableFormat::Right => write!(f, "{: >1$}", col, pad),
            }
        }
        let table_descriptor = Tbl::describe();

        let min_pad = 5;
        let out = std::cell::Cell::new(None);
        self.data.swap(&out);
        let Some(it) = out.into_inner() else {
            return Ok(());
        };
        let mut pads = vec![];
        let mut rows = vec![];

        pads.push(2 + table_descriptor.key.name.len().max(min_pad));
        for entry in &table_descriptor.v {
            pads.push(2 + entry.name.len().max(min_pad));
        }
        for x in it {
            let mut row = vec![];

            row.push((
                table_descriptor.key.format,
                format!("{}", table_descriptor.key.disp.call(&x.0)),
            ));
            for entry in &table_descriptor.v {
                row.push((entry.format, format!("{}", entry.disp.call(&x.1))));
            }
            for ((_, cell), pad) in row.iter().zip(&mut pads) {
                *pad = (*pad).max(2 + cell.len());
            }
            rows.push(row);
        }

        writeln!(
            f,
            "{}",
            line_sep(
                &pads,
                BOX_HORIZONTAL,
                BOX_CROSS_LEFT_DOWN,
                BOX_CROSS_DOWN,
                BOX_CROSS_RIGHT_DOWN,
            )
        )?;

        write!(f, "{BOX_VERT}{: ^1$}", table_descriptor.key.name, pads[0])?;
        for (entry, pad) in table_descriptor.v.iter().zip(pads[1..].iter().copied()) {
            write!(f, "{BOX_VERT}{: ^1$}", entry.name, pad)?;
        }
        writeln!(f, "{BOX_VERT}")?;

        writeln!(
            f,
            "{}",
            line_sep(
                &pads,
                BOX_HORIZONTAL,
                BOX_CROSS_LEFT,
                BOX_CROSS,
                BOX_CROSS_RIGHT,
            )
        )?;

        for row in rows {
            for ((format, cell), pad) in row.into_iter().zip(&pads) {
                write_col(f, format, &cell, *pad)?;
            }
            writeln!(f, "{BOX_VERT}")?;
        }

        write!(
            f,
            "{}",
            line_sep(
                &pads,
                BOX_HORIZONTAL,
                BOX_CROSS_LEFT_UP,
                BOX_CROSS_UP,
                BOX_CROSS_RIGHT_UP,
            )
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TableFormat {
    Center,
    Left,
    Right,
}

pub struct TableEntry<T> {
    name: &'static str,
    format: TableFormat,
    disp: Lens<T>,
}

pub struct TableDescriptor<Val, Key> {
    v: Vec<TableEntry<Val>>,
    key: TableEntry<Key>,
}

pub struct TableDescriptorBuilder<T, Key> {
    v: Vec<TableEntry<T>>,
    key: TableEntry<Key>,
}

struct Lens<T>(Box<dyn for<'a> Fn(&'a T) -> &'a dyn Display>);

impl<T> Lens<T> {
    pub fn call<'s>(&self, t: &'s T) -> &'s dyn Display {
        (self.0)(t)
    }
}

impl<T, Key> TableDescriptorBuilder<T, Key> {
    pub fn column_with_format(
        mut self,
        name: &'static str,
        format: TableFormat,
        getter: impl (Fn(&T) -> &dyn Display) + 'static,
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
        getter: impl (Fn(&T) -> &dyn Display) + 'static,
    ) -> Self {
        self.column_with_format(name, TableFormat::Left, getter)
    }

    pub fn column_key(
        name: &'static str,
        getter: impl (Fn(&Key) -> &dyn Display) + 'static,
    ) -> Self {
        Self::column_key_with_format(name, TableFormat::Center, getter)
    }
    pub fn column_key_with_format(
        name: &'static str,
        format: TableFormat,
        getter: impl (Fn(&Key) -> &dyn Display) + 'static,
    ) -> Self {
        Self {
            v: Vec::new(),
            key: TableEntry {
                name,
                format,
                disp: Lens(Box::new(getter)),
            },
        }
    }

    pub fn build(self) -> TableDescriptor<T, Key> {
        TableDescriptor {
            v: self.v,
            key: self.key,
        }
    }
}

pub trait Table: Sized {
    type Key;
    type Value;
    fn describe() -> TableDescriptor<Self::Value, Self::Key>;
}
