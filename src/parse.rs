extern crate select;
use crate::locate;
use select::document::Document;
use select::predicate::{Class, Name};
use std::fs;

pub struct DocParser {
    contents: Document,
    tag: locate::Tag,
}

impl DocParser {
    pub fn new(tagged_path: locate::TaggedPath) -> Self {
        let file_name = tagged_path.file_name.clone();
        let tag = tagged_path.tag.clone();
        let file = fs::File::open(tagged_path.path())
            .expect(&format!("unable to open file: {}", file_name));
        let contents =
            Document::from_read(file).expect(&format!("unable to parse HTML file: {}", file_name));

        return DocParser { contents, tag };
    }

    pub fn parse(&self) -> String {
        let mut sections: Vec<String> = vec![];

        match self.tag {
            locate::Tag::Struct => {
                sections.push(self.extract_type_declaration());
                sections.push(self.extract_summary());
                sections.push(self.extract_method_signatures());
            }
            _ => sections.push(self.extract_summary()),
        }

        return sections.join("\n\n");
    }

    fn extract_summary(&self) -> String {
        let docblock = self
            .contents
            .find(Class("docblock"))
            .filter(|n| !n.is(Class("type-decl")))
            .next()
            .unwrap();

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
        return paragraphs.join("\n\n");
    }

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

    // fn extract_structs(&self) -> String {}
    // fn extract_functions(&self) -> String {}
    // fn extract_traits(&self) -> String {}
    // fn extract_macros(&self) -> String {}
    // fn extract_enums(&self) -> String {}
    // fn extract_constants(&self) -> String {}
    // fn extract_modules(&self) -> String {}
}
