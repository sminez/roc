//! A very simple pretty printing table implementation that does not permit removal of rows.
//! Cells are left justified and trailing whitespace is stripped from the input.
use std::cmp;

/// Maximum output width when pretty printing
const MAX_WIDTH: usize = 90;
/// Space between columns when pretty printing
const SPACER: &'static str = "  ";

struct Row {
    cells: Vec<String>,
}

/// A very simple plain text table that knows the width of each of its columns.
pub struct Table {
    rows: Vec<Row>,
    column_widths: Vec<usize>,
}

impl Row {
    fn formatted(&self, column_widths: &Vec<usize>) -> String {
        self.cells
            .iter()
            .zip(column_widths)
            .map(|e| format!("{:01$}", e.0, e.1))
            .collect::<Vec<String>>()
            .join(SPACER)
    }

    fn two_column_wrapped(&self, column_widths: &Vec<usize>) -> String {
        if self.cells.len() != 2 {
            panic!(
                "two_column_wrapped called on row with {} columns",
                self.cells.len()
            );
        }

        if column_widths.iter().sum::<usize>() + SPACER.len() <= MAX_WIDTH {
            return self.formatted(&column_widths);
        }

        let mut buf = String::new();
        let mut current = String::new();
        let offset = vec![" "; column_widths[0] + SPACER.len() + 1].join("");
        current.push_str(&(format!("{:01$}", self.cells[0], column_widths[0]) + SPACER));

        for word in self.cells[1].split_whitespace() {
            if current.len() + word.len() + 1 < MAX_WIDTH {
                current.push_str(&format!(" {}", word));
            } else {
                buf.push_str(&format!("{}\n", current));
                current.clear();
                current.push_str(&format!("{}{}", offset, word));
            }
        }

        buf.push_str(&current);
        return buf;
    }
}

impl Table {
    /// Create a new empty Table without any rows
    pub fn new() -> Self {
        Table {
            rows: vec![],
            column_widths: vec![],
        }
    }

    /// Create a new Table from a vec of vecs as a one time operation.
    /// This is a convenience method that is intended to be used at the
    /// end of an iterator chain that make use of 'add_row' to construct
    /// the table from the input.
    pub fn from_rows(rows: Vec<Vec<String>>) -> Self {
        let mut t = Table::new();
        rows.iter().for_each(|r| t.add_row(r.clone()));
        return t;
    }

    /// Add a single row to an existing table and update column widths if needed.
    pub fn add_row(&mut self, cells: Vec<String>) {
        let diff = cells.len() - self.column_widths.len();
        if diff > 0 {
            self.column_widths.extend(vec![0; diff])
        }

        cells.iter().enumerate().for_each(|(i, cell)| {
            self.column_widths[i] = cmp::max(cell.len(), self.column_widths[i])
        });

        self.rows.push(Row { cells });
    }

    /// Convert this table to a left justified, column aligned string
    pub fn as_string(&self) -> String {
        self.rows
            .iter()
            .map(|r| r.two_column_wrapped(&self.column_widths))
            .collect::<Vec<String>>()
            .join("\n")
    }
}
