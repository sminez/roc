use std::path;

/// Our parsed input from the command line
#[derive(PartialEq, Eq, Debug)]
pub struct Query {
    /// are we checking the internal stdlib docs or something that should have
    /// been pulled in by Cargo.
    pub is_stdlib: bool,
    /// are we looking for a specific method or an individual symbol
    pub is_method: bool,
    /// the delimited path that we are going to try to parse for locating docs
    components: Vec<String>,
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
    fn path_buf(&self) -> path::PathBuf {
        let mut buf = path::PathBuf::new();
        for comp in &self.components {
            buf.push(comp.clone());
        }
        if self.is_method {
            buf.pop();
        }

        buf
    }

    fn method(&self) -> Option<String> {
        if self.is_method {
            if let Some(s) = self.components.last() {
                Some(String::from(s))
            } else {
                panic!("no last component in method query")
            }
        } else {
            None
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
