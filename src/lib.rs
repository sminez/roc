/*!
 * roc - command line doucmentation that rocks
 *
 * roc is an attempt at bringing godoc style quick docs searching to the command
 * line for rust. It doesn't generate any documentation itself, instead it relies
 * entirely on the local HTML output created by running `cargo doc` in the root of
 * your crate. You will need to have rust installed via rustup and have the std lib
 * docs downloaded in order to look at std lib.
 */
#![warn(missing_docs)]
pub mod locate;
pub mod parse;
pub mod pprint;
