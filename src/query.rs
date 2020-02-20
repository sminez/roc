/// Our parsed input from the command line
#[derive(PartialEq, Eq, Debug)]
struct Query {
    /// are we checking the internal stdlib docs or something that should have
    /// been pulled in by Cargo.
    is_stdlib: bool,
    /// the delimited path that we are going to try to parse for locating docs
    components: Vec<String>,
}

impl From<String> for Query {
    fn from(s: String) -> Query {
        let components: Vec<String> = s
            .split("::")
            .flat_map(|s| s.split(|c| c == '.' || c == ' '))
            .filter(|s| s.len() > 0)
            .map(|s| String::from(s))
            .collect();

        return Query {
            is_stdlib: components[0] == "std",
            components,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("std::fs::File", true, vec!["std", "fs", "File"])]
    #[test_case("std::path::PathBuf.file_name", true, vec!["std", "path", "PathBuf", "file_name"])]
    #[test_case("foo::Foo.bar", false, vec!["foo", "Foo", "bar"])]
    #[test_case("foo::Bar    baz", false, vec!["foo", "Bar", "baz"])]
    fn query_from_input(path: &str, is_stdlib: bool, comps: Vec<&str>) {
        assert_eq!(
            Query::from(String::from(path)),
            Query {
                is_stdlib: is_stdlib,
                components: comps.iter().map(|c| String::from(*c)).collect()
            }
        )
    }
}
