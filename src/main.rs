use std::{env, process};

use minigrep::Config;
fn main() { 
    let conf =Config::build_iterator(env::args()).unwrap_or_else(|err|{
        eprintln!("problem parsing arguments: {err}");
        process::exit(1);
    });
    //dbg!(conf.how_many_files());
    let readstdin:usize  = if conf.read_from_stdin() {1} else {0};
    //dbg!(conf.how_many_files() + readstdin);
    for ifile in 0..(conf.how_many_files() +readstdin){
        dbg!(ifile);
        if let Err(e) = minigrep::run(&conf, ifile ) {
            eprintln!{"Errore: {e}"};
        };
    }
} 
