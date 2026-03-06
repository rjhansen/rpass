use crate::terminal;
use terminal::get_words_per_line;
use clap::Parser;
use std::process::exit;

#[derive(Parser, Debug)]
#[command(
    author = "Robert J. Hansen <rob@hansen.engineering>",
    version,
    about,
    name = "rpass",
    about = "Generates high-entropy passwords.",
    long_about = "Constructs cryptographically strong passwords using the HC128\nstream cipher.",
    help_template = "\
{before-help}{name} {version}
{about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}
"
)]
pub struct Args {
    #[arg(
        short = 'c',
        long = "capitalize",
        help = "ensure one or more capital letters",
        default_value_t = false
    )]
    pub(crate) ensure_capitals: bool,

    #[arg(
        short = 'A',
        long = "no-capitalize",
        help = "ensure no capital letters",
        default_value_t = false
    )]
    pub(crate) no_capitals: bool,

    #[arg(
        short = 'n',
        long = "numerals",
        help = "ensure one or more numbers",
        default_value_t = false
    )]
    pub(crate) ensure_numbers: bool,

    #[arg(
        short = '0',
        long = "no-numerals",
        help = "ensure no numbers",
        default_value_t = false
    )]
    pub(crate) no_numbers: bool,

    #[arg(
        short = 'y',
        long = "symbols",
        help = "ensure at least one special character",
        default_value_t = false
    )]
    pub(crate) ensure_symbols: bool,

    #[arg(short = 'r', long = "remove-chars", help = "omit these characters", default_value_t = String::new())]
    pub(crate) remove: String,

    #[arg(
        short = 's',
        long = "secure",
        help = "use a cryptographic-strength RNG",
        default_value_t = true
    )]
    pub(crate) secure: bool,

    #[arg(
        short = 'B',
        long = "ambiguous",
        help = "don't include ambiguous characters",
        default_value_t = false
    )]
    pub(crate) no_ambiguous: bool,

    #[arg(
        short = 'v',
        long = "no-vowels",
        help = "don't use any vowels",
        default_value_t = false
    )]
    pub(crate) no_vowels: bool,

    #[arg(long = "copyright", help = "show copyright notice")]
    pub(crate) copyright: bool,

    #[arg(
        short = '1',
        long = "one-column",
        help = "display results in one column",
        default_value_t = false
    )]
    pub(crate) one_column: bool,

    #[arg(
        short = 'C',
        long = "multicolumn",
        help = "display results in multiple columns",
        default_value_t = true
    )]
    pub(crate) multi_column: bool,

    #[arg(short = 'b', long = "bugs", help = "where to file bug reports")]
    pub(crate) bugs: bool,

    pub(crate) length: Option<u16>,
    pub(crate) count: Option<u16>,
}

pub fn parse_command_line() -> Args {
    let mut rv = Args::parse();
    sanity_checks(&mut rv);
    rv
}

fn sanity_checks(args: &mut Args) {
    args.multi_column = true;

    if args.one_column {
        args.multi_column = false;
    }

    if (args.ensure_capitals && args.no_capitals) || (args.ensure_numbers && args.no_numbers) {
        eprintln!("error: conflicting flags");
        exit(1);
    }

    if args.bugs {
        println!("Submit bugs on GitHub: https://github.com/rjhansen/rpass");
        exit(0);
    }
    if args.copyright {
        println!(
            "{} {} is copyright © 2026, {}.",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_PKG_AUTHORS")
        );
        println!("Released under the Apache 2.0 license. Share and enjoy!");
        exit(0);
    }
    let length = args.length.unwrap_or_else(|| 8);
    let count = args.count.unwrap_or_else(|| 1);
    if length < 6 || length > 43 {
        eprintln!("error: length must be in the range [6, 43].");
        exit(1);
    }
    if count < 1 || count > 1000 {
        eprintln!("error: count must be in range [1, 1000].");
        exit(1);
    }
}

pub fn get_count(args: &Args) -> u16 {
    match args.count {
        None => match args.multi_column {
            true => {
                (get_words_per_line(args) + 1) * 20
            },
            false => 1,
        },
        Some(count) => count,
    }
}
