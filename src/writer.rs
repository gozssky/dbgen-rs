//! Helpers for writing out table rows.

use crate::{
    error::Error,
    eval::{Schema, State, Table},
    format::Format,
    span::{ResultExt, S},
    value::Value,
};
use std::{convert::TryInto, mem};

/// A generic writer which could accept rows of values.
pub trait Writer {
    /// Writes a single value.
    fn write_value(&mut self, format: &dyn Format, value: &Value) -> Result<(), S<Error>>;

    /// Writes the content at the beginning of each file.
    fn write_file_header(&mut self, format: &dyn Format, schema: &Schema) -> Result<(), S<Error>>;

    /// Writes the content of an INSERT statement before all rows.
    fn write_header(&mut self, format: &dyn Format, schema: &Schema) -> Result<(), S<Error>>;

    /// Writes the column name before a value.
    fn write_value_header(&mut self, format: &dyn Format, column: &str) -> Result<(), S<Error>>;

    /// Writes the separator between the every value.
    fn write_value_separator(&mut self, format: &dyn Format) -> Result<(), S<Error>>;

    /// Writes the separator between the every row.
    fn write_row_separator(&mut self, format: &dyn Format) -> Result<(), S<Error>>;

    /// Writes the content of an INSERT statement after all rows.
    fn write_trailer(&mut self, format: &dyn Format) -> Result<(), S<Error>>;
}

/// The state of a table within [`Env`].
#[derive(Debug, Clone)]
struct TableState<W: Writer> {
    /// The parsed table.
    table: Table,
    /// The table's schema.
    schema: Schema,
    /// Writer associated with the table.
    writer: W,
    /// Records that, within an [`Env::write_row()`] call, whether this table has not been visited
    /// yet (either as a root or derived tables). This member will be reset to `true` at the start
    /// of every `Env::write_row()` call.
    fresh: bool,
    /// Records if any rows have been written out. This determines whether an INSERT statement is
    /// needed to be written or not. This member will be reset to `true` after calling
    /// [`Env::write_trailer()`].
    empty: bool,
}

/// An environment for writing rows from multiple tables generated from a single template.
#[derive(Debug)]
pub struct Env<W: Writer> {
    state: State,
    tables: Vec<TableState<W>>,
}

impl<W: Writer> Env<W> {
    /// Constructs a new row-writing environment.
    pub fn new(
        tables: Vec<Table>,
        state: State,
        qualified: bool,
        mut new_writer: impl FnMut(&Table) -> Result<W, S<Error>>,
    ) -> Result<Self, S<Error>> {
        Ok(Self {
            tables: tables
                .iter()
                .map(|table| {
                    let writer = new_writer(table)?;
                    let schema = table.schema(qualified);
                    Ok::<_, S<Error>>(TableState {
                        table: table.clone(),
                        schema,
                        writer,
                        fresh: true,
                        empty: true,
                    })
                })
                .collect::<Result<_, _>>()?,
            state,
        })
    }

    /// Returns an iterator of tables and writers associated with this environment.
    pub fn tables(&mut self) -> impl Iterator<Item = (&Table, &mut W)> {
        self.tables.iter_mut().map(|table| (&table.table, &mut table.writer))
    }

    fn write_one_row(&mut self, format: &dyn Format, table_index: usize) -> Result<(), S<Error>> {
        let table = &mut self.tables[table_index];

        if mem::take(&mut table.empty) {
            table.writer.write_header(format, &table.schema)
        } else {
            table.writer.write_row_separator(format)
        }?;

        let values = table.table.row.eval(&mut self.state)?;

        for (col_index, (column, value)) in table.schema.column_names().zip(&values).enumerate() {
            if col_index != 0 {
                table.writer.write_value_separator(format)?;
            }
            table.writer.write_value_header(format, column)?;
            table.writer.write_value(format, value)?;
        }

        if table.table.derived.is_empty() {
            return Ok(());
        }

        let child = table.table.derived[0].0;
        let count = &table.table.derived[0].1;
        let count = count.eval(&mut self.state)?.try_into().span_err(count.0.span)?;

        for r in 1..=count {
            self.state.sub_row_num = r;
            self.write_one_row(format, child)?;
        }

        Ok(())
    }

    fn mark_descendant_visited(&mut self, root: usize) {
        let mut ids = vec![root];
        while let Some(id) = ids.pop() {
            let table = &mut self.tables[id];
            table.fresh = false;
            ids.extend(table.table.derived.iter().map(|child| child.0));
        }
    }

    /// Writes one row from each root table
    pub fn write_row(&mut self, format: &dyn Format) -> Result<(), S<Error>> {
        for table in &mut self.tables {
            table.fresh = true;
        }
        for i in 0..self.tables.len() {
            if self.tables[i].fresh {
                self.mark_descendant_visited(i);
                self.state.sub_row_num = 1;
                self.write_one_row(format, i)?;
            }
        }
        self.state.increase_row_num();
        Ok(())
    }

    /// Concludes an INSERT statement after writing multiple rows.
    ///
    /// This method delegates to [`Writer::write_trailer()`] if any rows have been written out
    /// previously for a table. Otherwise, if no rows have been written, this method does nothing.
    pub fn write_trailer(&mut self, format: &dyn Format) -> Result<(), S<Error>> {
        for table in &mut self.tables {
            if !mem::replace(&mut table.empty, true) {
                table.writer.write_trailer(format)?;
            }
        }
        Ok(())
    }
}
