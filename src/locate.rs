use super::query;
use std::{env, ffi, fs, path, process};

#[derive(PartialEq, Eq, Debug, Clone)]
enum Tag {
    Constant,
    Enum,
    Function,
    Macro,
    Module,
    Primative,
    Struct,
    Trait,
    Unknown,
}

impl From<path::PathBuf> for Tag {
    fn from(path_buf: path::PathBuf) -> Tag {
        let file_name = path_buf.file_name().unwrap();
        if file_name == ffi::OsString::from("index.html") {
            return Tag::Module;
        }

        match file_name.to_os_string().into_string() {
            Err(_) => Tag::Unknown,
            Ok(s) => match s.split('.').collect::<Vec<&str>>()[0] {
                "constant" => Tag::Constant,
                "enum" => Tag::Enum,
                "fn" => Tag::Function,
                "macro" => Tag::Macro,
                "primative" => Tag::Primative,
                "struct" => Tag::Struct,
                "trait" => Tag::Trait,
                _ => Tag::Unknown,
            },
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct TaggedPath {
    path_buf: path::PathBuf,
    file_name: String,
    without_prefix: Option<ffi::OsString>,
    tag: Tag,
}

impl TaggedPath {
    pub fn path(&self) -> String {
        String::from(self.path_buf.to_str().unwrap())
    }
}

impl From<path::PathBuf> for TaggedPath {
    fn from(path_buf: path::PathBuf) -> TaggedPath {
        // TODO: handle gracefully / determine an OS independent way of doing this
        let file_name = match path_buf.file_name().unwrap().to_str() {
            Some(s) => String::from(s),
            None => panic!("file path is not valid utf8"),
        };
        let tag = Tag::from(path_buf.clone());
        let without_prefix = match tag {
            Tag::Module | Tag::Unknown => None,
            _ => Some(ffi::OsString::from(
                file_name.splitn(2, '.').collect::<Vec<&str>>()[1],
            )),
        };

        TaggedPath {
            path_buf,
            file_name,
            without_prefix,
            tag,
        }
    }
}

#[derive(Debug)]
pub struct Locator {
    pub root: path::PathBuf,
    query: query::Query,
}

impl Locator {
    pub fn new(query: query::Query) -> Locator {
        let root = match get_doc_root(query.is_stdlib) {
            Some(r) => r,
            None => {
                // TODO: these should be user friendly error messages
                //       that then exit the program rather than panics
                if query.is_stdlib {
                    panic!("unable to locate sysroot")
                } else {
                    panic!("no cargo generated docs")
                }
            }
        };

        Locator { root, query }
    }

    pub fn determine_target_file_path(&self) -> Option<TaggedPath> {
        let last_element = self.query.last_as_os_string();
        let target_filename = ffi::OsString::from(self.query.filename());

        let mut search_path = self.root.clone();
        search_path.extend(self.query.dir_as_path_buf().iter());

        // Check to see if we are targeting a module and grab index.html
        // if we are.
        search_path.push(last_element.clone());
        if search_path.is_dir() {
            search_path.push("index.html");
            return Some(TaggedPath::from(search_path));
        } else {
            search_path.pop();
        }

        while search_path != self.root {
            if let Ok(entries) = search_path.read_dir() {
                let valid_entries = entries
                    .filter_map(|p| p.ok())
                    .collect::<Vec<fs::DirEntry>>();

                for entry in valid_entries {
                    let tagged = TaggedPath::from(entry.path());
                    if let Some(without_prefix) = tagged.clone().without_prefix {
                        if without_prefix == target_filename {
                            return Some(tagged);
                        }
                    }
                }
            };

            if let Some(p) = search_path.file_name() {
                if p == target_filename {
                    return Some(TaggedPath::from(search_path));
                }
            }

            search_path.pop();
        }
        return None;
    }
}

fn get_doc_root(is_stdlib: bool) -> Option<path::PathBuf> {
    if is_stdlib {
        get_sys_root().map(|r| r.join(path::Path::new("share/doc/rust/html")))
    } else {
        get_crate_root().map(|r| r.join(path::Path::new("target/doc")))
    }
}

fn get_sys_root() -> Option<path::PathBuf> {
    process::Command::new("rustc")
        .arg("--print")
        .arg("sysroot")
        .output()
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map(|s| path::Path::new(s.trim()).to_path_buf())
}

fn get_crate_root() -> Option<path::PathBuf> {
    let mut cur_dir = env::current_dir().ok().unwrap();

    while cur_dir != path::Path::new("/") {
        if let Ok(paths) = cur_dir.read_dir() {
            for entry in paths {
                if let Ok(entry) = entry {
                    if let Some(fname) = entry.path().as_path().file_name() {
                        if fname == ffi::OsStr::new("Cargo.toml") {
                            return Some(cur_dir);
                        }
                    }
                };
            }
        };
        cur_dir.pop();
    }
    return None;
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("test_resources/foo/enum.elon.html", Tag::Enum)]
    #[test_case("test_resources/foo/fn.foo.html", Tag::Function)]
    #[test_case("test_resources/foo/macro.makrow.html", Tag::Macro)]
    #[test_case("test_resources/foo/index.html", Tag::Module)]
    #[test_case("test_resources/foo/primative.ug.html", Tag::Primative)]
    #[test_case("test_resources/foo/struct.structural.html", Tag::Struct)]
    #[test_case("test_resources/foo/trait.fooable.html", Tag::Trait)]
    #[test_case("test_resources/foo/some_other_unknown.html", Tag::Unknown)]
    fn path_buf_into_symbol_type(path: &str, expected: Tag) {
        let path_buf = path::PathBuf::from(path);
        let symbol_type = Tag::from(path_buf);

        assert_eq!(symbol_type, expected);
    }
}
