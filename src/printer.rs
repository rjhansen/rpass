use crate::cmdline;
use crate::cmdline::get_count;
use crate::terminal::get_words_per_line;

pub fn make_printer(args: &cmdline::Args) -> impl FnMut(&String) {
    let count = get_count(args);
    let wpl = get_words_per_line(args);
    let mut index = 0;
    let mut remaining_in_line = wpl;

    move |password: &String| -> () {
        if args.multi_column {
            if remaining_in_line == 0 {
                println!("{password}");
                remaining_in_line = wpl;
                index += 1;
            } else if index >= count - 1 {
                println!("{password}");
                remaining_in_line -= 1;
                index += 1;
            } else {
                print!("{password} ");
                remaining_in_line -= 1;
                index += 1;
            }
        } else {
            println!("{password}");
            index += 1;
        }
    }
}
