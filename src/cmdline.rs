use clap::Parser;
use std::process::exit;

#[derive(Parser, Debug)]
#[command(
    author = "Robert J. Hansen <rob@hansen.engineering>",
    version,
    about,
    name = "rpass",
    about = "Generates high-entropy passwords.",
    long_about = "Constructs cryptographically strong passwords using the HC128 stream cipher.",
    help_template = "\
{before-help}{name} {version}
{about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}
"
)]
pub struct Args {
    #[arg(
        short,
        long,
        help = "bits of entropy per password",
        default_value_t = 128
    )]
    pub(crate) bits: u16,

    #[arg(
        short,
        long,
        help = "number of passwords to generate",
        default_value_t = 1
    )]
    pub(crate) count: u32,

    #[arg(long = "copyright", help = "show copyright notice")]
    copyright: bool,

    #[arg(long = "bugs", help = "show contact information for bug reports")]
    bugs: bool,
}

pub fn parse_command_line() -> Args {
    let rv = Args::parse();
    sanity_checks(&rv);
    rv
}

fn sanity_checks(args: &Args) {
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
    if (0 != (args.bits % 8)) || (args.bits < 64) || (args.bits > 256) {
        eprintln!("error: bits must be a multiple of eight in the range [64, 256].");
        exit(1);
    }
    if args.count > 20 || args.count < 1 {
        eprintln!("error: count must be in range [1, 20].");
        exit(1);
    }
}
