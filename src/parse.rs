/*!
 * Parse the contents of rustdoc generated HTML files
 */
extern crate select;

use crate::{locate, table};
use select::document::Document;
use select::node::Node;
use select::predicate::{And, Class, Name, Not};
use std::fs;

fn header(s: &str) -> String {
    format!("{}\n{}", s, vec!["-"; s.len()].join(""),)
}

/**
 * Parses generated HTML output from rustdoc to give summarised results.
 */
pub struct DocParser {
    contents: Document,
    tag: locate::Tag,
    method_name: Option<String>,
}

impl DocParser {
    /// Create a new DocParser rooted at the given tagged search path
    pub fn new(tagged_path: locate::TaggedPath) -> Self {
        let file_name = tagged_path.file_name.clone();
        let file = fs::File::open(tagged_path.path())
            .expect(&format!("unable to open file: {}", file_name));
        let contents = Document::from_read(file).expect(&format!(
            "unable to parse rustdoc generated HTML file: {}",
            file_name
        ));

        return DocParser {
            contents,
            tag: tagged_path.tag.clone(),
            method_name: tagged_path.method_name.clone(),
        };
    }

    /// Instead of parsing the contents of the search result, show child modules instead
    pub fn show_child_modules(&self) {
        let s = if let Some(ms) = self.extract_modules() {
            ms
        } else {
            "No child modules found".into()
        };

        println!("{}", s);
    }

    /// Parse the contents of a located doc file and pretty print them to the terminal
    pub fn parse_and_print(&self) {
        let mut sections: Vec<String> = vec![];

        match self.tag {
            locate::Tag::Module => {
                if let Some(s) = self.extract_summary() {
                    sections.push(s)
                };
                if let Some(s) = self.extract_modules() {
                    sections.push(s)
                };
                if let Some(s) = self.extract_traits() {
                    sections.push(s)
                };
                if let Some(s) = self.extract_constants() {
                    sections.push(s)
                };
                if let Some(s) = self.extract_structs() {
                    sections.push(s)
                };
                if let Some(s) = self.extract_enums() {
                    sections.push(s)
                };
                if let Some(s) = self.extract_functions() {
                    sections.push(s)
                };
                if let Some(s) = self.extract_macros() {
                    sections.push(s)
                };
            }

            locate::Tag::Struct => {
                sections.push(self.extract_type_declaration());
                if let Some(s) = self.extract_summary() {
                    sections.push(s)
                }
                sections.push(self.extract_method_signatures());
            }

            locate::Tag::Method => {
                let s = match self.extract_method() {
                    Some(s) => s,
                    None => format!("{} is not method", self.method_name.clone().unwrap()),
                };
                sections.push(s)
            }

            _ => {
                if let Some(s) = self.extract_summary() {
                    sections.push(s)
                }
            }
        }

        // TODO: Current parsing leaves '[src]' at the end of a lot of lines
        //       This is a quick hack to tidy that up but we should do this in
        //       a smarter way really...
        sections.retain(|s| s.len() > 0);
        println!("{}", sections.join("\n\n").replace("[src]", ""));
    }

    fn extract_summary(&self) -> Option<String> {
        let docblock = self
            .contents
            .find(And(Class("docblock"), Not(Class("type-decl"))))
            .next()?;

        let mut paragraphs: Vec<String> = vec![];
        for node in docblock.children() {
            if node.is(Name("p")) {
                paragraphs.push(node.text());
            } else if node.text() == "\n" {
                continue;
            } else {
                break;
            }
        }
        return Some(paragraphs.join("\n\n"));
    }

    // Not Option-al as all structs must have a type declaration
    fn extract_type_declaration(&self) -> String {
        self.contents
            .find(Class("type-decl"))
            .map(|n| n.text())
            .collect::<Vec<String>>()
            .join("\n")
    }

    fn extract_method_signatures(&self) -> String {
        let impl_block = self.contents.find(Class("impl-items")).next().unwrap();
        let mut methods: Vec<String> = vec![];

        for node in impl_block.children() {
            if node.is(Class("method")) {
                methods.push(node.text());
            } else {
                continue;
            }
        }

        return methods.join("\n");
    }

    fn extract_method(&self) -> Option<String> {
        let mut sections: Vec<String> = vec![];
        let id = format!("method.{}", self.method_name.clone()?);
        let node = self
            .contents
            .find(|n: &Node| n.attr("id").map_or(false, |i| i == id))
            .next()?;

        sections.push(node.text());

        if let Some(n) = node.next() {
            if n.is(Class("docblock")) {
                // TODO: the raw formatting here isn't great as it becomes one big blob
                //       probably want to try our own iteration over the children?
                sections.push(n.text());
            }
        }

        return Some(sections.join("\n\n"));
    }

    fn table_after_header(&self, header: &str) -> Option<String> {
        Some(
            table::Table::from_rows(
                self.contents
                    .find(And(Class("section-header"), |n: &Node| {
                        n.attr("id").map_or(false, |i| i == header)
                    }))
                    .next()? // the header itself
                    .next()? // a newline...
                    .next()? // the table
                    .first_child()? // tbody
                    .children()
                    .map(|n| {
                        n.children()
                            .map(|c| String::from(c.text().replace("\n", " ").trim_end()))
                            .collect::<Vec<String>>()
                    })
                    .collect::<Vec<Vec<String>>>(),
            )
            .as_string(),
        )
    }

    fn table_with_header(&self, header_str: &str) -> Option<String> {
        self.table_after_header(header_str)
            .map(|t| format!("{}\n{}", header(header_str), t))
    }

    fn extract_structs(&self) -> Option<String> {
        self.table_with_header("structs")
    }

    fn extract_functions(&self) -> Option<String> {
        self.table_with_header("functions")
    }

    fn extract_traits(&self) -> Option<String> {
        self.table_with_header("traits")
    }

    fn extract_macros(&self) -> Option<String> {
        self.table_with_header("macros")
    }

    fn extract_enums(&self) -> Option<String> {
        self.table_with_header("enums")
    }

    fn extract_constants(&self) -> Option<String> {
        self.table_with_header("constants")
    }

    fn extract_modules(&self) -> Option<String> {
        self.table_with_header("modules")
    }
}
