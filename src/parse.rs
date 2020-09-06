/*!
 * Parse the contents of rustdoc generated HTML files
 */
use crate::{
    locate, pprint,
    pprint::{header, ENUM_HEADING_COLOR, SECTION_HEADING_COLOR},
};
use grep::{
    regex::RegexMatcher,
    searcher::{sinks::UTF8, Searcher},
};
use select::{
    document::Document,
    node::Node,
    predicate::{And, Class, Name, Not},
};
use std::{error::Error, fs};

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
        let s = if let Some(ms) = self.table_with_header("modules", &None) {
            ms
        } else {
            "No child modules found".into()
        };

        println!("{}", s);
    }

    /// Parse the contents of a located doc file and pretty print them to the terminal
    pub fn parse_and_print(&self, grep: Option<String>) {
        let mut sections: Vec<String> = vec![];

        match self.tag {
            locate::Tag::Module => {
                if let Some(s) = self.extract_summary() {
                    sections.push(s)
                };
                if let Some(s) = self.table_with_header("modules", &grep) {
                    sections.push(s)
                };
                if let Some(s) = self.table_with_header("traits", &grep) {
                    sections.push(s)
                };
                if let Some(s) = self.table_with_header("constants", &grep) {
                    sections.push(s)
                };
                if let Some(s) = self.table_with_header("structs", &grep) {
                    sections.push(s)
                };
                if let Some(s) = self.table_with_header("enums", &grep) {
                    sections.push(s)
                };
                if let Some(s) = self.table_with_header("functions", &grep) {
                    sections.push(s)
                };
                if let Some(s) = self.table_with_header("macros", &grep) {
                    sections.push(s)
                };
            }

            locate::Tag::Struct => {
                sections.push(self.extract_type_declaration());
                if let Some(s) = self.extract_summary() {
                    sections.push(s)
                }
                sections.push(self.extract_method_signatures(&grep));
            }

            locate::Tag::Method => {
                let s = match self.extract_method() {
                    Some(s) => s,
                    None => format!("{} is not method", self.method_name.clone().unwrap()),
                };
                sections.push(s)
            }

            locate::Tag::Enum => {
                if let Some(s) = self.extract_summary() {
                    sections.push(s)
                }
                if let Some(s) = self.extract_enum_variants(&grep) {
                    sections.push(s)
                };
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

    fn extract_method_signatures(&self, grep: &Option<String>) -> String {
        let impl_block = self.contents.find(Class("impl-items")).next().unwrap();
        let mut methods: Vec<String> = vec![];

        for node in impl_block.children() {
            if node.is(Class("method")) {
                methods.push(node.text());
            } else {
                continue;
            }
        }

        let raw = methods.join("\n");
        match grep {
            Some(grep_str) => matching_lines(raw, grep_str).unwrap(),
            None => raw,
        }
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

    fn extract_enum_variants(&self, grep: &Option<String>) -> Option<String> {
        let raw = self
            .contents
            .find(|n: &Node| n.attr("id").map_or(false, |i| i.starts_with("variant.")))
            .map(|n| {
                let mut lines = vec![header(&n.text(), ENUM_HEADING_COLOR)];
                if let Some(n) = n.next() {
                    if n.is(Class("docblock")) {
                        lines.push(n.text());
                    }
                }
                lines.join("\n")
            })
            .collect::<Vec<String>>()
            .join("\n");

        match grep {
            Some(grep_str) => matching_lines(raw, grep_str).ok(),
            None => Some(raw),
        }
    }

    fn table_after_header(&self, header: &str) -> Option<String> {
        Some(
            pprint::Table::from_rows(
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

    fn table_with_header(&self, header_str: &str, grep: &Option<String>) -> Option<String> {
        self.table_after_header(header_str).map(|t| {
            let s = match grep {
                Some(grep_str) => match matching_lines(t, grep_str) {
                    Ok(lines) => lines,
                    Err(e) => panic!("{}", e),
                },
                None => t,
            };
            format!("{}\n{}", header(header_str, SECTION_HEADING_COLOR), s)
        })
    }
}

fn matching_lines(s: String, pattern: &str) -> Result<String, Box<dyn Error>> {
    let matcher = RegexMatcher::new(pattern)?;
    let mut matches: Vec<String> = vec![];
    let lines: Vec<&str> = s.split('\n').collect();

    Searcher::new().search_slice(
        &matcher,
        s.as_bytes(),
        UTF8(|lnum, _| {
            matches.push(lines[lnum as usize].to_string());
            Ok(true)
        }),
    )?;

    Ok(matches.join("\n"))
}
