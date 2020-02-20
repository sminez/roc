# roc - notes on desired behaviour and implementation decisions

### Why parse rustdoc HTML output?
In theory we _should_ be able to use the rust stdlib code directly (libsyntax?)
to parse Rust source files and generate the documentation directly. While this
does feel like the "right" way to go about this there are two major drawbacks:
  1) We need to make sure that our parsing of Rust source is always up to date
  with language changes and that we can handle both different editions and minor
  point versions of the language.
  2) Parsing from source is both a lot more complicated and potentially slower:
  `racer` frequently hangs / crashes in Vim for me and genreating full `rustdoc`
  output is painfully slow (on a par with compile times as opposed to near
  instant feedback as with `go doc`)

So, if instead we require that the user has generated documentation (via
`rustdoc` / `cargo doc`) which we then parse in order to generate our stubbed
output, we gain in a couple of key areas:
  1) Parsing HTML is stable so all we need to worry about is any changes to
  `rustdoc` that alter the tags/headings used in the HTML output which breaks
  our parsing / scraping of the `rustdoc` output. In the case where there is a
  breaking change for us, we are altering tags that we pull out not worrying
  about parsing rust source.
  2) While `rustdoc` itself is slow, we can treat the output as a cache that we
  then read directly from disk. The format (at time or writing) prepends 'type'
  information to the file names (e.g. `struct.mystruct.html`) that allows us to
  use a simple walk of the doc output directory to resolve a large number of
  common queries and also determine what we need to pull out of each file to
  generate the output for `roc`.


### Query semantics
The following outlines what we expect to see returned (/printed) as the result
of each category of query when using `roc` on the command line. In cases where
it is possible to make a direct comparison to `go doc`, `racer` or `rustdoc` the
output from those commands will be marked with their source.

Note that the `cut` command used for `racer` output is used to make the output
easier to parse visually here, it removes some of the output that is useful for
programatic use in auto-completion (which is what racer is for!)


1) Crate/module level
```bash
$ roc [std|crate]::SomeModule

# racer will list the _source files_ under a given module but only if you append
# a trailing '::', without that it will simply report the location of the crate
# root itself. Output is an ordered, typed list of module contents including
# signatures
$ racer complete std::fs:: | cut -d, -f5-

# Output is an example of how to import this module, summary documentation about
# the module and an ordered, typed list of module contents including signatures
$ go doc os
```

Sometimes the top level summary for a crate/module is a bit much to blat out
onto the command line so we may want a flag to toggle between "just give me the
names and signatures" and "summarise what each item is".


2) A struct
```bash
$ roc [std|crate]::SomeModule::SomeStruct

# With racer, no trailing '::' will give the details for the struct itself (no
# description of fields though) and trailing '::' will give details for methods.
$ racer complete std::fs::File | cut -d, -f5-
$ racer complete std::fs::File:: | cut -d, -f5-

# Again, shows how to import, the struct definition, functions that _return_ the
# struct and methods. Methods and functions get their signatures but not their
# docstrings.
$ go doc os File
```

We want to mirror the `go doc` behaviour of defaulting to showing all methods
(static and instance) along with any top level documentation on the struct and
public data members itself.


3) A method on a struct (static and instance based) or top level function
```bash
$ roc [std|crate]::SomeModule::SomeStruct::SomeStaticMethod
$ roc [std|crate]::SomeModule::SomeStruct.SomeInstanceMethod
$ roc [std|crate]::SomeModule::SomeFunction

# Only gives the signature
$ racer complete std::fs::File::open | cut -d, -f5-

# Signature and docstring
$ go doc os File.Name
```


4) Top level constant, variable or interface/trait
```bash
$ roc [std|crate]::SomeModule::SomeConstant
$ roc [std|crate]::SomeModule::SomeVar
$ roc [std|crate]::SomeModule::SomeTrait

$ racer ?

# Declaration showing the value along with docstring. For interfaces, go doc
# will show the signature of each instance method as well (effectively the raw
# source definition)
$ go doc os DevNull
```

We want to copy `go doc`
