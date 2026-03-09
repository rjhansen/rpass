use crate::cmdline;
use crate::cmdline::get_count;
use crate::terminal::get_words_per_line;

pub fn make_printer(args: cmdline::Args) -> impl FnMut(&String) {
    let count = get_count(&args);
    let wpl = get_words_per_line(&args);
    let mut index = 0;
    let mut remaining_in_line = wpl;

    move |password: &String| {
        let is_last = index >= count.saturating_sub(1);

        if !args.multi_column || is_last {
            println!("{password}");
        } else {
            remaining_in_line -= 1;
            if remaining_in_line == 0 {
                println!("{password}");
                remaining_in_line = wpl;
            } else {
                print!("{password} ");
            }
        }
        index += 1;
    }
}
