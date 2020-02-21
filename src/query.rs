use std::{ffi, path};

/// Our parsed input from the command line
#[derive(PartialEq, Eq, Debug)]
pub struct Query {
    /// are we checking the internal stdlib docs or something that should have
    /// been pulled in by Cargo.
    pub is_stdlib: bool,
    /// are we looking for a specific method or an individual symbol
    pub is_method: bool,
    /// the delimited path that we are going to try to parse for locating docs
    pub components: Vec<String>,
}

impl From<String> for Query {
    fn from(s: String) -> Query {
        let components: Vec<String> = s
            .split("::")
            .flat_map(|s| s.split('.'))
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();

        Query {
            is_stdlib: components[0] == "std",
            is_method: s.contains('.'),
            components,
        }
    }
}

impl Query {
    pub fn dir_as_path_buf(&self) -> path::PathBuf {
        let mut buf = path::PathBuf::new();
        buf.extend(self.components.iter());
        buf.pop();

        return buf;
    }

    pub fn filename(&self) -> String {
        if let Some(s) = self.components.last() {
            String::from(s) + ".html"
        } else {
            panic!("no last component in method query")
        }
    }

    pub fn last_as_os_string(&self) -> ffi::OsString {
        if let Some(s) = self.components.last() {
            ffi::OsString::from(s)
        } else {
            panic!("no last component in method query")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("std::fs::File", true, false, vec!["std", "fs", "File"])]
    #[test_case("std::path::PathBuf.file_name", true, true, vec!["std", "path", "PathBuf", "file_name"])]
    #[test_case("foo::Foo.bar", false, true, vec!["foo", "Foo", "bar"])]
    fn query_from_input(path: &str, is_stdlib: bool, is_method: bool, comps: Vec<&str>) {
        assert_eq!(
            Query::from(String::from(path)),
            Query {
                is_stdlib: is_stdlib,
                is_method: is_method,
                components: comps.iter().map(|c| String::from(*c)).collect()
            }
        )
    }
}
