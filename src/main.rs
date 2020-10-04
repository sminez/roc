use clap::Clap;
use rocdoc::locate;
use rocdoc::parse;
use std::process;

const CRATE_ROOT_QUERIES: &[&'static str] = &[".", "crate"];

/**
 * roc :: command lines rust documentation that rocks
 */
#[derive(Clap, Debug)]
struct Options {
    /// list out child modules under the given path
    #[clap(short = 'l', long = "list")]
    list: bool,

    /// open the selected doc page in the browser using full rustdoc
    #[clap(short = 'o', long = "open")]
    open_in_browser: bool,

    /// grep the resulting output to only show lines matching this query
    #[clap(short = 'g', long = "grep")]
    grep: Option<String>,

    /// <crate/mod>[::<symbol>[.<method>]]
    query: String,
}

fn main() {
    let opts: Options = Options::parse();

    if CRATE_ROOT_QUERIES.contains(&opts.query.as_ref()) {
        if let Err(e) = locate::list_known_crates() {
            eprintln!("{}", e);
        }
        return;
    }

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
        parse::DocParser::new(tagged_path).parse_and_print(opts.grep);
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
