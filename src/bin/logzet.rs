use dagzet::logzet;
use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;

fn main() {
    logzet::hello();
    let mut stdin = false;

    if env::args().len() < 2 {
        stdin = true;
    }

    if stdin {
        let reader = BufReader::new(io::stdin());
        for line in reader.lines().map_while(Result::ok) {
            println!("{}", line);
        }
        return;
    }

    let filenames = env::args().skip(1);

    for filename in filenames {
        let f = File::open(filename).unwrap();
        let reader = BufReader::new(f);
        for line in reader.lines().map_while(Result::ok) {
            println!("{}", line);
        }
    }
}
