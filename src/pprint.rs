/*!
 * Formatted output and pretty printing
 */
use colored::*;
use std::cmp;
use term_size;

// Colors used when outputting with color
pub(crate) const CRATE_LIST_HEADING_COLOR: &'static str = "blue";
pub(crate) const SECTION_HEADING_COLOR: &'static str = "yellow";
pub(crate) const ENUM_HEADING_COLOR: &'static str = "green";

// Space between columns when pretty printing
const SPACER: &'static str = "  ";

// Default maximum output width when pretty printing
const DEFAULT_TERM_WIDTH: usize = 90;

/// A simple colored header for sections
pub(crate) fn header(s: &str, color: &str) -> String {
    format!("{} {}", "::".color(color).bold(), s.bold())
}

/// Print a vec of strings as column spaced rows
pub(crate) fn pprint_as_columns(elems: Vec<String>) -> String {
    let col_width = elems.iter().map(String::len).max().unwrap();
    let per_row = max_width() / (col_width + 1);

    elems
        .chunks(per_row)
        .map(|chunk| {
            chunk
                .iter()
                .map(|c| format!("{:01$}", c, col_width))
                .collect::<Vec<String>>()
                .join(" ")
        })
        .collect::<Vec<String>>()
        .join("\n")
}

fn max_width() -> usize {
    if let Some((w, _)) = term_size::dimensions() {
        w
    } else {
        DEFAULT_TERM_WIDTH
    }
}

// A single row within a table layout
struct Row {
    cells: Vec<String>,
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

        let max = max_width();
        if column_widths.iter().sum::<usize>() + SPACER.len() <= max {
            return self.formatted(&column_widths);
        }

        let mut buf = String::new();
        let mut current = String::new();
        let offset = vec![" "; column_widths[0] + SPACER.len() + 1].join("");
        current.push_str(&(format!("{:01$}", self.cells[0], column_widths[0]) + SPACER));

        for word in self.cells[1].split_whitespace() {
            if current.len() + word.len() + 1 < max {
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

/// A very simple plain text table that knows the width of each of its columns.
pub(crate) struct Table {
    rows: Vec<Row>,
    column_widths: Vec<usize>,
}

impl Table {
    /// Create a new empty Table without any rows
    pub fn new() -> Self {
        Table {
            rows: vec![],
            column_widths: vec![],
        }
    }

    /**
     * Create a new Table from a vec of vecs as a one time operation.
     * This is a convenience method that is intended to be used at the
     * end of an iterator chain that make use of 'add_row' to construct
     * the table from the input.
     */
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
