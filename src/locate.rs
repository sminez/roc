use std::{env, ffi, path, process};

#[derive(PartialEq, Eq, Debug)]
enum SymbolType {
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

impl From<path::PathBuf> for SymbolType {
    fn from(path_buf: path::PathBuf) -> SymbolType {
        if path_buf.is_file() {
            match path_buf.file_name() {
                Some(p) => match p.to_os_string().into_string() {
                    Err(_) => return SymbolType::Unknown,
                    Ok(s) => match s.split('.').collect::<Vec<&str>>()[0] {
                        "constant" => SymbolType::Constant,
                        "enum" => SymbolType::Enum,
                        "fn" => SymbolType::Function,
                        "macro" => SymbolType::Macro,
                        "primative" => SymbolType::Primative,
                        "struct" => SymbolType::Struct,
                        "trait" => SymbolType::Trait,
                        _ => SymbolType::Unknown,
                    },
                },
                None => return SymbolType::Unknown,
            }
        } else {
            SymbolType::Module
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
struct Symbol {
    symbol_type: SymbolType,
    path_buf: path::PathBuf,
    name: ffi::OsString,
}

impl From<path::PathBuf> for Symbol {
    fn from(path_buf: path::PathBuf) -> Symbol {
        let symbol_type = SymbolType::from(path_buf.clone());
        let name = path_buf.file_name().unwrap().to_os_string();
        return Symbol {
            symbol_type,
            path_buf,
            name,
        };
    }
}

#[derive(Debug)]
pub struct Locator {
    sys_root: Option<path::PathBuf>,
    crate_root: Option<path::PathBuf>,
}

impl Locator {
    pub fn new() -> Locator {
        let sys_root = get_sys_root();
        let crate_root = get_crate_root();

        return Locator {
            sys_root,
            crate_root,
        };
    }

    fn parse_directory(dir: path::PathBuf) -> Vec<SymbolType> {
        // fetch and parse contents, dropping unknowns
        match dir.read_dir() {
            Err(_) => panic!("unable to read directory"),
            Ok(paths) => paths
                .filter_map(|p| p.ok())
                .map(|p| SymbolType::from(p.path()))
                .collect(),
        }
    }

    // Starting directory when looking for std::* documentation
    // fn std_doc_root(&self) -> Option<path::PathBuf> {
    //     self.sys_root
    //         .clone()
    //         .map(|r| r.join(path::Path::new("share/docs/rust/html/std")))
    // }

    // Starting directory when looking for !std::* documentation
    // fn crate_doc_root(&self) -> Option<path::PathBuf> {
    //     self.crate_root
    //         .clone()
    //         .map(|r| r.join(path::Path::new("target/doc")))
    // }
}

fn get_sys_root() -> Option<path::PathBuf> {
    return process::Command::new("rustc")
        .arg("--print")
        .arg("sysroot")
        .output()
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map(|s| path::Path::new(s.trim()).to_path_buf());
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
    use test_case::test_case;

    use super::*;

    #[test_case("/home/foo/enum.elon.html", SymbolType::Enum)]
    #[test_case("/home/foo/fn.foo.html", SymbolType::Function)]
    #[test_case("/home/foo/macro.makrow.html", SymbolType::Macro)]
    #[test_case("/home/foo/index.html", SymbolType::Unknown)]
    #[test_case("/home/foo/primative.ug.html", SymbolType::Primative)]
    #[test_case("/home/foo/struct.structural.html", SymbolType::Struct)]
    #[test_case("/home/foo/trait.fooable.html", SymbolType::Trait)]
    fn path_buf_into_symbol_type(path: &str, expected: SymbolType) {
        let path_buf = path::PathBuf::from(path);
        let symbol_type = SymbolType::from(path_buf);

        assert_eq!(symbol_type, expected);
    }
}
