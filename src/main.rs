use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

fn parse_line(line: &str) {
    if line.len() < 3 {
        return;
    }
    let cmd = &line[0..2];
    let args = &line[3..];
    dbg!(cmd, args);

    match cmd {
        "ns" => {}
        "gr" => {}
        "ln" => {}
        "co" => {}
        "cr" => {}
        "nn" => {}

        c => {
            // TODO: (better) error handling
            panic!("could not find: {c}");
        }
    }
}

fn main() {
    if env::args().len() < 2 {
        println!("Please supply a dagzet file\n");
        return;
    }

    let filename: &str = &env::args().last().unwrap();
    let f = File::open(&filename).unwrap();
    let reader = BufReader::new(f);

    //let _ = reader.read_line(&mut line)?;
    let lines_iter = reader.lines().map(|l| l.unwrap());

    for str in lines_iter {
        parse_line(&str);
    }
}
