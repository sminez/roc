use std::{env, ffi, path, process};

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Tag {
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
    without_prefix: Option<ffi::OsString>,
    pub file_name: String,
    pub tag: Tag,
}

impl TaggedPath {
    pub fn path(&self) -> String {
        String::from(self.path_buf.to_str().unwrap())
    }
}

impl From<path::PathBuf> for TaggedPath {
    fn from(path_buf: path::PathBuf) -> TaggedPath {
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

/// Local documentation is stored in different locations depending on whether or
/// not the query resolves to something that is from the standard library or a
/// third party crate that the user is pulling in via Cargo.
#[derive(PartialEq, Eq, Debug)]
enum CrateType {
    StdLib,
    Cargo,
}

/// We can't resolve all cases when we parse the Query but it is useful to know
/// if the query is for a method or a concrete symbol that will have its own
/// documentation file
#[derive(PartialEq, Eq, Debug)]
enum QueryType {
    StaticMethod,
    InstanceMethod,
    ConcreteSymbol, // TODO: This needs a better name. Essentially this is "a file"
    Unknown,
}

/// A Locator handles mapping a user query string from the command line to a file
/// location on disk. It also provides information about what kind of documentation
/// file it has found so that the appropriate parsing of the file contents can be
/// carried out.
#[derive(PartialEq, Eq, Debug)]
pub struct Locator {
    root: path::PathBuf,
    crate_type: CrateType,
    query_type: QueryType,
    components: Vec<String>,
}

impl Locator {
    pub fn new(query: String) -> Self {
        let components: Vec<String> = query
            .split("::")
            .flat_map(|s| s.split('.'))
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();

        let crate_type = if components[0] == "std" {
            CrateType::StdLib
        } else {
            CrateType::Cargo
        };

        let query_type = if query.contains('.') {
            QueryType::InstanceMethod
        } else {
            QueryType::Unknown
        };

        let root = get_doc_root(&crate_type).expect("unable to locate documentation root");

        return Locator {
            root,
            crate_type,
            query_type,
            components,
        };
    }

    pub fn target_file_path(&self) -> Option<String> {
        self.determine_tagged_path().map(|p| p.path())
    }

    pub fn determine_tagged_path(&self) -> Option<TaggedPath> {
        let mut search_path = self.root.clone();
        search_path.extend(self.query_dir_as_path_buf().iter());

        // Check to see if we are targeting a module and grab index.html
        // if we are.
        search_path.push(self.last_component().clone());
        if search_path.is_dir() {
            search_path.push("index.html");
            return Some(TaggedPath::from(search_path));
        } else {
            search_path.pop();
        }

        let target_filename = self.query_filename();

        while search_path != self.root {
            if let Ok(entries) = search_path.read_dir() {
                for entry in entries.filter_map(|p| p.ok()) {
                    let tagged = TaggedPath::from(entry.path());
                    if let Some(without_prefix) = &tagged.without_prefix {
                        if without_prefix == &target_filename {
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

    fn query_dir_as_path_buf(&self) -> path::PathBuf {
        let mut buf = path::PathBuf::new();
        buf.extend(self.components.iter());
        buf.pop();

        return buf;
    }

    fn query_filename(&self) -> ffi::OsString {
        if let Some(s) = self.components.last() {
            ffi::OsString::from(String::from(s) + ".html")
        } else {
            panic!("no last component in method query")
        }
    }

    fn last_component(&self) -> ffi::OsString {
        if let Some(s) = self.components.last() {
            ffi::OsString::from(s)
        } else {
            panic!("no last component in method query")
        }
    }
}

fn get_doc_root(crate_type: &CrateType) -> Option<path::PathBuf> {
    match crate_type {
        CrateType::StdLib => get_sys_root().map(|r| r.join(path::Path::new("share/doc/rust/html"))),
        CrateType::Cargo => get_crate_root().map(|r| r.join(path::Path::new("target/doc"))),
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
    let cargo_toml = ffi::OsStr::new("Cargo.toml");
    let file_system_root = path::Path::new("/");

    while cur_dir != file_system_root {
        if let Ok(paths) = cur_dir.read_dir() {
            for entry in paths {
                if let Ok(entry) = entry {
                    if let Some(fname) = entry.path().as_path().file_name() {
                        if fname == cargo_toml {
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

    #[test_case("std::fs::File", CrateType::StdLib, QueryType::Unknown, vec!["std", "fs", "File"])]
    #[test_case("std::path::PathBuf.file_name", CrateType::StdLib, QueryType::InstanceMethod, vec!["std", "path", "PathBuf", "file_name"])]
    #[test_case("foo::Foo.bar", CrateType::Cargo, QueryType::InstanceMethod, vec!["foo", "Foo", "bar"])]
    fn locator_from_input(
        path: &str,
        crate_type: CrateType,
        query_type: QueryType,
        comps: Vec<&str>,
    ) {
        let root = get_doc_root(&crate_type).unwrap();
        assert_eq!(
            Locator::new(String::from(path)),
            Locator {
                root: root,
                crate_type: crate_type,
                query_type: query_type,
                components: comps.iter().map(|c| String::from(*c)).collect()
            }
        )
    }
}
