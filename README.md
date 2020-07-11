roc -- cli rust documentation that rocks
----------------------------------------

### caveats
* assumes that you are using rustup and that you have stdlib docs downloaded
* requires that you build any dependency crate docs before they can be found

### usage
```bash
# show summary comment for the Eq trait from stdlib
$ roc std::cmp::Eq
Trait for equality comparisons which are equivalence relations.

This means, that in addition to a == b and a != b being strict inverses, the equality must
be (for all a, b and c)

# generate the documentation for roc itself
$ cd roc && cargo doc

# show top level summary details for the roc crate
$ roc roc
roc - command line doucmentation that rocks

roc is an attempt at bringing godoc style quick docs searching to the command
line for rust. It doesn\'t generate any documentation itself, instead it relies
entirely on the local HTML output created by running cargo doc in the root of
your crate. You will need to have rust installed via rustup and have the std lib
docs downloaded in order to look at std lib.

modules
-------
locate   Locate the generated docs that we have available within the current workspace
parse    Parse the contents of rustdoc generated HTML files
table    A very simple pretty printing table implementation that does not permit removal


# show specific information about the Locator struct
$ roc roc::locate::Locator
pub struct Locator { /* fields omitted */ }

A Locator handles mapping a user query string from the command line to a file
location on disk. It also provides information about what kind of documentation
file it has found so that the appropriate parsing of the file contents can be
carried out.

pub fn new(query: String) -> Self
pub fn target_file_path(&self) -> Option<String>
pub fn determine_tagged_path(&self) -> Option<TaggedPath>
```


### flags
Probably want ways to hunt through known impls and Traits etc?
```
-l, --list          list out modules under the current path
-o, --open          open the selected doc page in the browser (local copy)
```


### doc locations
```
std::* -> $(rustc --print sysroot)/share/doc/rust/html/std
*      -> $(dirname Cargo.toml)/target/doc
```
