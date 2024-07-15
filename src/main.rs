use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

struct DagZet {
    /// The current namespace
    pub namespace: String,
}

impl DagZet {
    pub fn new() -> Self {
        DagZet {
            namespace: "".to_string(),
        }
    }

    pub fn parse_line(&mut self, line: &str) {
        if line.len() < 3 {
            return;
        }
        let cmd = &line[0..2];
        let args = &line[3..];
        dbg!(cmd, args);

        match cmd {
            "ns" => {
                self.namespace = args.to_string();
            }
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
}

fn main() {
    if env::args().len() < 2 {
        println!("Please supply a dagzet file\n");
        return;
    }

    let filename: &str = &env::args().last().unwrap();
    let f = File::open(&filename).unwrap();
    let reader = BufReader::new(f);
    let mut dz = DagZet::new();

    //let _ = reader.read_line(&mut line)?;
    let lines_iter = reader.lines().map(|l| l.unwrap());

    for str in lines_iter {
        dz.parse_line(&str);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_namespace() {
        let mut dz = DagZet::new();

        dz.parse_line(&"ns hello");

        assert_eq!(dz.namespace, "hello".to_string());
    }
}
