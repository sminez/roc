use clap::Clap;
use roc::locate;
use roc::parse;
use std::process;

/// roc -- command lines rust documentation that rocks
#[derive(Clap, Debug)]
struct Options {
    /// list out child modules under the given path
    #[clap(short = "l", long = "list")]
    list: bool,

    // /// show any example code that is provided in the full docs
    // #[clap(short = "e", long = "show-examples")]
    // show_examples: bool,
    /// open the selected doc page in the browser using full rustdoc
    #[clap(short = "o", long = "open")]
    open_in_browser: bool,

    // /// grep through the documentation for partial matches (ignores case)
    // #[clap(short = "s", long = "string")]
    // grep: Option<String>,
    /// <mod>[::<symbol>[.<method>]]
    query: String,
}

fn main() {
    let opts: Options = Options::parse();
    let locator = locate::Locator::new(opts.query);
    let tagged_path = match locator.determine_tagged_path() {
        Some(p) => p,
        None => {
            println!("unable to resolve query path");
            process::exit(1);
        }
    };

    if opts.open_in_browser {
        open_in_browser(tagged_path);
    } else if opts.list {
        parse::DocParser::new(tagged_path).show_child_modules();
    } else {
        parse::DocParser::new(tagged_path).parse_and_print();
    }
}

fn open_in_browser(tp: locate::TaggedPath) {
    let path = tp.path();
    let res = process::Command::new("xdg-open").arg(&path).spawn();
    if let Err(e) = res {
        match e.kind() {
            std::io::ErrorKind::NotFound => {
                process::Command::new("open")
                    .arg(path)
                    .spawn()
                    .expect("failed to open using the 'open' command");
            }
            _ => panic!("failed to open using the 'xdg-open' command"),
        }
    }
}
