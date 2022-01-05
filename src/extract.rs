//! Use syn to extract definitions and documentation from Rust source files.
//!
//! The primary entrypoint for this module is the extract_items function which returns the parsed
//! items found within the target module.
// TODO: handle impl blocks being written in different files.
use quote::ToTokens;
use std::fmt;
use syn::{
    visit::{self, Visit},
    Attribute, File, ItemFn, Lit, Meta, Result, Signature,
};

#[derive(Debug, Clone, Default)]
pub struct Module {
    name: String,
    docs: Option<String>,
}

impl fmt::Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(s) = &self.docs {
            write!(f, "[{}]\n{}", self.name, s)
        } else {
            write!(f, "[{}]", self.name)
        }
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    // name: String,
    // vis: String,
    sig: String,
    docs: Option<String>,
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(s) = &self.docs {
            write!(f, "{}\n{}", s, self.sig)
        } else {
            write!(f, "{}", self.sig)
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DocItems {
    module: Module,
    fns: Vec<Function>,
}

impl DocItems {
    pub fn render_all(&self) -> String {
        format!(
            "{}\n\n[Functions]\n{}",
            self.module,
            self.fns
                .iter()
                .map(|f| format!("{}\n", f))
                .collect::<String>()
        )
    }
}

#[derive(Debug, Clone, Default)]
struct Extractor {
    module: String,
    items: DocItems,
}

pub fn extract_items(module: &str, contents: &str) -> Result<DocItems> {
    let syntax = syn::parse_file(contents)?;
    let mut ex = Extractor {
        module: module.to_string(),
        items: Default::default(),
    };
    ex.visit_file(&syntax);

    Ok(ex.items)
}

impl<'ast> Visit<'ast> for Extractor {
    fn visit_file(&mut self, node: &'ast File) {
        self.items.module = Module {
            name: self.module.clone(),
            docs: extract_docs(&node.attrs).unwrap_or_default(),
        };

        visit::visit_file(self, node);
    }

    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        self.items.fns.push(Function {
            // name: node.sig.ident.to_string(),
            // vis: format!("{}", node.vis.clone().into_token_stream()),
            sig: format_sig(node.sig.clone()),
            docs: extract_docs(&node.attrs).unwrap_or_default(),
        });
    }
}

fn extract_docs(attrs: &[Attribute]) -> Result<Option<String>> {
    let mut docs = vec![];

    for attr in attrs {
        match attr.parse_meta()? {
            Meta::NameValue(nv) if nv.path.is_ident("doc") => {
                if let Lit::Str(s) = nv.lit {
                    docs.push(s.value().trim().to_string());
                }
            }
            _ => continue,
        }
    }

    Ok(if docs.is_empty() {
        None
    } else {
        Some(docs.join("\n"))
    })
}

fn format_sig(sig: Signature) -> String {
    format!("{}", sig.into_token_stream())
        .replace(" (", "(")
        .replace(" < ", "<")
        .replace(" > ", ">")
        .replace(" >", ">")
        .replace("& ", "&")
        .replace(" ,", ",")
        .replace(" :", ":")
}
