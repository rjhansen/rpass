use crate::cmdline;
use crate::cmdline::get_count;
use crate::terminal::get_words_per_line;

pub fn make_printer(args: &cmdline::Args) -> impl FnMut(&String, u16) {
    let count = get_count(args);
    let wpl = get_words_per_line(args);
    let mut remaining_in_line = wpl;

    move |password: &String, index: u16| -> () {
        if args.multi_column {
            if remaining_in_line == 0 {
                println!("{password}");
                remaining_in_line = wpl;
            } else if index >= count - 1 {
                println!("{password}");
                remaining_in_line -= 1;
            } else {
                print!("{password} ");
                remaining_in_line -= 1;
            }
        } else {
            println!("{password}");
        }
    }
}
