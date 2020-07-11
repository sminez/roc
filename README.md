roc -- cli rust documentation that rocks
----------------------------------------

`https://github.com/utkarshkukreti/select.rs`

### caveats
* assumes that you are using rustup and that you have stdlib docs downloaded
* requires that you build any dependency crate docs before they can be found

### usage
```bash
$ roc std::fs              # show summary details for the std::fs module
$ roc std::fs::File        # show fields, methods and impls for fs::File
$ roc std::fs::File::open  # show signature and summary for File::open()
```


### flags
Probably want ways to hunt through known impls and Traits etc?
```
-l, --list          list out all known modules
-g, --grep          grep through the documentation for partial matches (ignores case)
                    -- this probably also wants to be able to specify that
                       you are looking for a type/struct/enum/trait/function...
-e, --show-examples show any example code that is provided in the full docs
-o, --open          open the selected doc page in the browser using full rustdoc
```


### doc locations
```
std::* -> $(rustc --print sysroot)/share/doc/rust/html/std
*      -> $(dirname Cargo.toml)/target/doc
```

We should be able to grab sysroot fairly simply and then grab the docs from
there for stdlib stuff and this _might_ also mean that we're not tied to the
user having rustup vs some other method of installing Rust?

For in Crate docs (and deps) we probably don't want to build the docs without
asking for confirmation from the user. We will need to walk back up from the
current working directory until we hit a `Cargo.toml` that we can use to root
ourselves and find the doc output in.


### plan of attack
-- basic usage --
- split based on whether the user is after `std` or anything else:
  - `std` we start the search in sysroot
  - everything else we look in the per-crate `target/doc` directory.
- use the filesystem to locate the correct HTML file to parse (based on the
  first part of the target we were given)
  - If we can't find the file then show an error and prompt for grepping
- hunt through that file for the identifier we are after:
  - need to work out what HTML tags we are after here as this could be anything
  - if we can't find it, error out and prompt for listing the module.
- glob out the content we want (defined below) and process for printing to the
  terminal (ignore ANSI colors for the moment: there is probably a crate that
  just pretty prints HTML in color actually...)
- print the results


<< PROBABLY EASIEST TO START WITH THESE >>
-- open --
- locate the target file
- punt to the browser if found, error out if not


-- list --
- use the filesystem to grab all top level modules in `std` and the local
  `target/doc` if there is one.
- pretty print out the results (calling out parent crate in each case)
