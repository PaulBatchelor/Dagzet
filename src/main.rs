use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

enum ReturnCode {
    Okay,
    Error,
    NameSpaceNotSet,
    NodeAlreadyExists,
    NodeNotSelected,
}

struct DagZet {
    /// The current namespace
    pub namespace: Option<String>,
    pub graph_remarks: HashMap<String, Vec<String>>,
    pub curnode: Option<u32>,
    pub nodes: HashMap<String, u32>,
    pub lines: HashMap<u32, Vec<String>>,
}

impl DagZet {
    pub fn new() -> Self {
        DagZet {
            namespace: None,
            graph_remarks: HashMap::new(),
            curnode: None,
            nodes: HashMap::new(),
            lines: HashMap::new(),
        }
    }
    pub fn parse_line(&mut self, line: &str) {
        let _ = self.parse_line_with_result(line);
    }

    pub fn parse_line_with_result(&mut self, line: &str) -> Result<ReturnCode, ReturnCode> {
        if line.len() < 3 {
            return Err(ReturnCode::Error);
        }

        let cmd = &line[0..2];
        let args = &line[3..];

        match cmd {
            "ns" => {
                self.namespace = Some(args.to_string());
            }
            "gr" => {
                let gr = &mut self.graph_remarks;

                let ns = match &self.namespace {
                    Some(n) => n,
                    None => return Err(ReturnCode::NameSpaceNotSet),
                };

                match gr.get_mut(ns) {
                    Some(remarks) => remarks.push(args.to_string()),
                    None => {
                        gr.insert(ns.clone(), vec![args.to_string()]);
                    }
                }
            }
            "nn" => {
                let ns = match &self.namespace {
                    Some(n) => n,
                    None => return Err(ReturnCode::NameSpaceNotSet),
                };

                // TODO: append to set/hashmap, make sure it doesn't already exist

                // TODO: make this a path with the namespace, create node ID
                //let nodename = ns.copy() + "/".to_string() + args.to_string();
                let mut nodename = String::from(ns);
                nodename.push('/');
                nodename.push_str(args);
                dbg!(nodename.to_string());
                let nodes = &mut self.nodes;

                if nodes.contains_key(&nodename) {
                    return Err(ReturnCode::NodeAlreadyExists);
                }

                let node_id = nodes.len() as u32 + 1;

                nodes.insert(nodename, node_id);

                self.curnode = Some(node_id);
            }
            "ln" => {
                let curnode = match &self.curnode {
                    Some(id) => *id,
                    _ => return Err(ReturnCode::NodeNotSelected),
                };
                let lines = &mut self.lines;

                match lines.get_mut(&curnode) {
                    Some(ln) => ln.push(args.to_string()),
                    None => {
                        lines.insert(curnode, vec![args.to_string()]);
                    }
                }
            }
            "co" => {}
            "cr" => {}

            _ => return Err(ReturnCode::Error),
        }
        Ok(ReturnCode::Okay)
    }
}

fn main() {
    if env::args().len() < 2 {
        println!("Please supply a dagzet file\n");
        return;
    }

    let filename: &str = &env::args().last().unwrap();
    let f = File::open(filename).unwrap();
    let reader = BufReader::new(f);
    let mut dz = DagZet::new();

    //let _ = reader.read_line(&mut line)?;
    let lines_iter = reader.lines().map(|l| l.unwrap());

    for str in lines_iter {
        // TODO: handle error
        dz.parse_line(&str);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_namespace() {
        let mut dz = DagZet::new();

        dz.parse_line("ns hello");

        assert_eq!(dz.namespace, Some("hello".to_string()));
    }

    #[test]
    fn test_graph_remarks() {
        let mut dz = DagZet::new();
        dz.parse_line("ns hello");
        dz.parse_line("gr this is a graph remark");
        dz.parse_line("gr for the node called hello");

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
    #[test]
    fn test_new_node() {
        let mut dz = DagZet::new();
        let caught_no_namespace = match dz.parse_line_with_result("nn hello") {
            Ok(_) => false,
            Err(rc) => matches!(rc, ReturnCode::NameSpaceNotSet),
        };
        assert!(caught_no_namespace);

        // catch multiple node declared error
        let mut dz = DagZet::new();

        dz.parse_line("ns aaa");
        dz.parse_line("nn bbb");
        let caught_duplicate_node = match dz.parse_line_with_result("nn bbb") {
            Ok(_) => false,
            Err(rc) => matches!(rc, ReturnCode::NodeAlreadyExists),
        };
        assert!(caught_duplicate_node);
        assert!(dz.nodes.contains_key("aaa/bbb"));
    }

    #[test]
    fn test_lines() {
        let mut dz = DagZet::new();
        // attempt to parse lines without select a node
        dz.parse_line("ns aaa");

        let caught_missing_node = match dz.parse_line_with_result("ln hello line") {
            Ok(_) => false,
            Err(rc) => matches!(rc, ReturnCode::NodeNotSelected),
        };

        assert!(caught_missing_node);

        let mut dz = DagZet::new();
        // attempt to parse lines without select a node
        dz.parse_line("ns aaa");
        dz.parse_line("nn bbb");
        dz.parse_line("ln ccc");
        dz.parse_line("ln another line");

        // Make sure the lines are behaving as expected.
        assert_eq!(dz.lines.len(), 1);
        assert!(dz.nodes.contains_key("aaa/bbb"));

        let node_id = dz.nodes.get("aaa/bbb").unwrap();

        if let Some(ln) = dz.lines.get(node_id) {
            assert_eq!(ln.len(), 2);
            assert_eq!(ln[0], "ccc");
            assert_eq!(ln[1], "another line");
        }
    }
}
