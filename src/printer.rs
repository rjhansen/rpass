use crate::cmdline;
use crate::cmdline::get_count;
use crate::terminal::get_words_per_line;

pub fn make_printer(args: &cmdline::Args) -> impl FnMut(&String, u16) -> () {
    let count = get_count(&args);
    let wpl = get_words_per_line(&args);
    let mut remaining_in_line = wpl;
    
    move |password: &String, index: u16| -> () {
        match args.multi_column {
            // In one-column mode, terminate each password with a newline.
            // This is the simple case. Past here, check the breadcrumbs.
            false => println!("{}", password),
            
            true => match remaining_in_line > 0 {
                // If we don't have anything else to print to this line,
                // terminate the password with a newline and reset our
                // remaining_in_line variable.
                false => {
                    println!("{}", password);
                    remaining_in_line = wpl;
                }
                // But we can still need to print a newline even if we have
                // room left on our line.
                true => match index >= count - 1 {
                    // If the password we're printing is the final one of the 
                    // series, terminate it with a newline.
                    true => {
                        println!("{}", password);
                        remaining_in_line -= 1;
                    }
                    // Otherwise, append a space to create a column break.
                    false => {
                        print!("{} ", password);
                        remaining_in_line -= 1;
                    }
                },
            },
        }
    }
}
