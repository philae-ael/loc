use std::fmt::Display;

const BOX_VERT: &str = "│";
const BOX_HORIZONTAL: &str = "─";
const BOX_CROSS_LEFT: &str = "├";
const BOX_CROSS: &str = "┼";
const BOX_CROSS_RIGHT: &str = "┤";

const BOX_CROSS_LEFT_DOWN: &str = "┌";
const BOX_CROSS_DOWN: &str = "┬";
const BOX_CROSS_RIGHT_DOWN: &str = "┐";

const BOX_CROSS_LEFT_UP: &str = "└";
const BOX_CROSS_UP: &str = "┴";
const BOX_CROSS_RIGHT_UP: &str = "┘";

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
    V: Table<Key = U>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn draw_line_sep(
            f: &mut std::fmt::Formatter<'_>,
            pads: &[usize],
            box_horizontal: &str,
            box_left: &str,
            box_mid: &str,
            box_right: &str,
        ) -> std::fmt::Result {
            let total: usize = pads.iter().sum::<usize>() + pads.len() + 2;
            let mut buf: Vec<&str> = vec![box_horizontal; total];

            let mut sum = 0;
            buf[0] = box_left;
            for s in pads {
                sum += s + 1;
                buf[sum] = box_mid;
            }
            buf[sum] = box_right;
            buf[sum + 1] = "\n";

            f.write_str(&buf.into_iter().collect::<String>())
        }

        fn write_col<T>(
            f: &mut std::fmt::Formatter<'_>,
            entry: &TableEntry<T>,
            t: &T,
            pad: usize,
        ) -> std::fmt::Result {
            let d = format!("{}", entry.disp.call(t));
            write!(f, "{BOX_VERT}")?;
            match entry.format {
                TableFormat::Center => write!(f, "{: ^1$}", d, pad),
                TableFormat::Left => write!(f, "{: <1$}", d, pad),
                TableFormat::Right => write!(f, "{: >1$}", d, pad),
            }
        }
        let table_descriptor = V::describe();

        let min_pad = 8;
        let out = std::cell::Cell::new(None);
        self.data.swap(&out);
        let Some(it) = out.into_inner() else {
            return Ok(());
        };
        let mut pads = vec![];

        pads.push(2 + table_descriptor.key.name.len().max(min_pad));
        for entry in &table_descriptor.v {
            pads.push(2 + entry.name.len().max(min_pad));
        }

        draw_line_sep(
            f,
            &pads,
            BOX_HORIZONTAL,
            BOX_CROSS_LEFT_DOWN,
            BOX_CROSS_DOWN,
            BOX_CROSS_RIGHT_DOWN,
        )?;

        write!(f, "{BOX_VERT}{: ^1$}", table_descriptor.key.name, pads[0])?;
        for (entry, pad) in table_descriptor.v.iter().zip(pads[1..].iter().copied()) {
            write!(f, "{BOX_VERT}{: ^1$}", entry.name, pad)?;
        }
        writeln!(f, "{BOX_VERT}")?;

        draw_line_sep(
            f,
            &pads,
            BOX_HORIZONTAL,
            BOX_CROSS_LEFT,
            BOX_CROSS,
            BOX_CROSS_RIGHT,
        )?;

        for x in it {
            write_col(f, &table_descriptor.key, &x.0, pads[0])?;
            for (entry, pad) in table_descriptor.v.iter().zip(&pads[1..]) {
                write_col(f, entry, &x.1, *pad)?;
            }
            writeln!(f, "{BOX_VERT}")?;
        }

        draw_line_sep(
            f,
            &pads,
            BOX_HORIZONTAL,
            BOX_CROSS_LEFT_UP,
            BOX_CROSS_UP,
            BOX_CROSS_RIGHT_UP,
        )?;
        Ok(())
    }
}

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

pub struct TableDescriptor<T, Key> {
    v: Vec<TableEntry<T>>,
    key: TableEntry<Key>,
}

pub struct TableDescriptorBuilder<T, Key> {
    v: Vec<TableEntry<T>>,
    key: TableEntry<Key>,
}

struct Lens<T>(Box<dyn (for<'a> Fn(&'a T) -> &'a dyn Display)>);

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
        Self {
            v: Vec::new(),
            key: TableEntry {
                name,
                format: TableFormat::Center,
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
    fn describe() -> TableDescriptor<Self, Self::Key>;
}
