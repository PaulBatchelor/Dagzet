use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

struct DagZet {
    /// The current namespace
    pub namespace: String,
    pub graph_remarks: HashMap<String, Vec<String>>,
}

impl DagZet {
    pub fn new() -> Self {
        DagZet {
            namespace: "".to_string(),
            graph_remarks: HashMap::new(),
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
            "gr" => {
                let gr = &mut self.graph_remarks;

                match gr.get_mut(&self.namespace) {
                    Some(remarks) => remarks.push(args.to_string()),
                    None => {
                        gr.insert(self.namespace.clone(), vec![args.to_string()]);
                    }
                }
            }
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

    #[test]
    fn test_graph_remarks() {
        let mut dz = DagZet::new();
        dz.parse_line(&"ns hello");
        dz.parse_line(&"gr this is a graph remark");
        dz.parse_line(&"gr for the node called hello");

        assert_eq!(dz.graph_remarks.len(), 1);
        assert!(dz.graph_remarks.contains_key("hello"));

        let gr = dz.graph_remarks;

        match gr.get("hello") {
            Some(remarks) => {
                assert_eq!(remarks.len(), 2);
                assert_eq!(remarks[0], "this is a graph remark");
                assert_eq!(remarks[1], "for the node called hello");
            }
            None => {
                // Shouldn't happen, since there was a check before this
            }
        };
    }
}
