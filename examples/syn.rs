use quote::ToTokens;
use std::fs::read_to_string;
use syn::{visit::Visit, Attribute, File, ItemFn, Lit, Meta, Result, Signature};

/// Find a node in the tree with a function name that matches the target.
///
/// This also has a multi-line doc comment
fn find_fn_docs<'a>(_name: &'a mut str, _file: &File) -> Option<String> {
    None
}

struct FnVisitor;

fn format_signature(sig: Signature) -> String {
    format!("{}", sig.into_token_stream())
        .replace(" (", "(")
        .replace(" < ", "<")
        .replace(" > ", ">")
        .replace(" >", ">")
        .replace("& ", "&")
        .replace(" ,", ",")
        .replace(" :", ":")
}

impl<'ast> Visit<'ast> for FnVisitor {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        if let Ok(Some(docs)) = extract_docs(&node.attrs) {
            println!("{}", docs);
        }
        println!("{}\n", format_signature(node.sig.clone()));
    }
}

/// Extract documentation strings from a slice of attributes
fn extract_docs(attrs: &[Attribute]) -> Result<Option<String>> {
    let mut docs = vec![];

    for attr in attrs {
        match attr.parse_meta()? {
            Meta::NameValue(nv) if nv.path.is_ident("doc") => {
                if let Lit::Str(s) = nv.lit {
                    docs.push(format!("/// {}", s.value().trim().to_string()));
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

fn main() {
    let contents = read_to_string("examples/syn.rs").expect("wrong file path");
    let syntax = syn::parse_file(&contents).expect("Unable to parse file");

    FnVisitor.visit_file(&syntax);
}
