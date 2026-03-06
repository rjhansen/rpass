use crate::cmdline;
use crate::cmdline::get_count;
use crate::terminal::get_words_per_line;

pub fn make_printer(args: &cmdline::Args) -> impl FnMut(&String, u16) -> () {
    let count = get_count(&args);
    let wpl = get_words_per_line(&args);
    let mut remaining_in_line = wpl;
    move |password: &String, index: u16| -> () {
        match args.multi_column {
            false => println!("{}", password),
            true => match remaining_in_line > 0 {
                false => {
                    println!("{}", password);
                    remaining_in_line = wpl;
                }
                true => match index >= count - 1 {
                    true => {
                        println!("{}", password);
                        remaining_in_line -= 1;
                    }
                    false => {
                        print!("{} ", password);
                        remaining_in_line -= 1;
                    }
                },
            },
        }
    }
}
