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
    NotEnoughArgs,
    AlreadyConnected,
}

struct DagZet {
    /// The current namespace
    pub namespace: Option<String>,
    /// For each graph namespace, have some remarks represented as lines
    pub graph_remarks: HashMap<String, Vec<String>>,

    /// The local ID value of the currently selected node
    pub curnode: Option<u32>,

    /// Nodes stored in a hashmap, used to prevent duplicates
    pub nodes: HashMap<String, u32>,

    /// An inverse lookup table for the nodes. This
    /// assumes it will be updated consistently with
    /// the node hashmap
    pub nodelist: Vec<String>,

    /// Each line can ave text content called "lines" (ln)
    pub lines: HashMap<u32, Vec<String>>,

    /// Edges of the knowledge graph. These are represented
    /// as strings instead of IDs so they can be resolved
    /// later. This allows connections to be made before
    /// nodes are made, which is more flexible.
    pub connections: Vec<[String; 2]>,
}

impl DagZet {
    pub fn new() -> Self {
        DagZet {
            namespace: None,
            graph_remarks: HashMap::new(),
            curnode: None,
            nodes: HashMap::new(),
            nodelist: vec![],
            lines: HashMap::new(),
            connections: vec![],
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

                let mut nodename = String::from(ns);
                nodename.push('/');
                nodename.push_str(args);
                dbg!(nodename.to_string());
                let nodes = &mut self.nodes;

                if nodes.contains_key(&nodename) {
                    return Err(ReturnCode::NodeAlreadyExists);
                }

                let node_id = nodes.len() as u32 + 1;

                self.nodelist.push(nodename.clone());
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
            "co" => {
                let ns = match &self.namespace {
                    Some(n) => n,
                    None => return Err(ReturnCode::NameSpaceNotSet),
                };

                let connect_args: Vec<_> = args.split_whitespace().collect();

                if connect_args.len() < 2 {
                    return Err(ReturnCode::NotEnoughArgs);
                }

                // TODO: add '$' shortcut

                let use_left_shorthand = connect_args[0] == "$";
                let use_right_shorthand = connect_args[1] == "$";

                let _curnode = if use_left_shorthand || use_right_shorthand {
                    match self.curnode {
                        Some(x) => Some(&self.nodelist[x as usize - 1]),
                        None => return Err(ReturnCode::NodeNotSelected),
                    }
                } else {
                    None
                };

                let mut left = ns.to_string();
                let mut right = ns.to_string();

                left.push('/');
                left.push_str(connect_args[0]);

                right.push('/');
                right.push_str(connect_args[1]);

                if self.already_connected(&left, &right) {
                    return Err(ReturnCode::AlreadyConnected);
                }

                self.connections.push([left, right]);
            }
            "cr" => {}

            _ => return Err(ReturnCode::Error),
        }
        Ok(ReturnCode::Okay)
    }

    fn already_connected(&self, left: &str, right: &str) -> bool {
        for con in &self.connections {
            let lmatch = left == con[0];

            if lmatch {
                let rmatch = right == con[1];

                if lmatch && rmatch {
                    return true;
                }
            }
        }
        false
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

        assert_eq!(dz.nodes.len(), 1, "Expected nodes.");
        assert_eq!(
            dz.nodelist.len(),
            dz.nodes.len(),
            "nodelist inconsistency: wrong size."
        );

        let caught_duplicate_node = match dz.parse_line_with_result("nn bbb") {
            Ok(_) => false,
            Err(rc) => matches!(rc, ReturnCode::NodeAlreadyExists),
        };
        assert!(caught_duplicate_node);
        assert!(dz.nodes.contains_key("aaa/bbb"));

        let node_id = dz.nodes.get("aaa/bbb").unwrap();
        let node_id = *node_id as usize - 1;

        let maps_to_nodelist = dz.nodelist[node_id] == "aaa/bbb";

        assert!(
            maps_to_nodelist,
            "nodelist inconsistency: ID mapping broken"
        );
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
    #[test]
    fn test_connect() {
        let mut dz = DagZet::new();
        dz.parse_line("ns top");
        dz.parse_line("nn aaa");
        dz.parse_line("nn bbb");

        let result = match dz.parse_line_with_result("co bbb") {
            Ok(_) => false,
            Err(rc) => matches!(rc, ReturnCode::NotEnoughArgs),
        };

        assert!(result, "Did not catch NotEnoughArgs error");

        dz.parse_line("co bbb aaa");

        assert_eq!(
            dz.connections.len(),
            1,
            "expected a single connection to be made"
        );

        let c = &dz.connections[0];

        let aaa = "top/aaa";
        let bbb = "top/bbb";

        assert_eq!(&c[0], bbb, "expected top/bbb node in left connection");
        assert_eq!(&c[1], aaa, "expected top/aaa node in right connection");

        // Make sure different namespaces work
        dz.parse_line("ns pot");
        dz.parse_line("nn aaa");
        dz.parse_line("nn bbb");
        dz.parse_line("co bbb aaa");

        let c = &dz.connections[1];
        let aaa = "pot/aaa";
        let bbb = "pot/bbb";

        assert_eq!(&c[0], bbb, "expected pot/bbb node in left connection");
        assert_eq!(&c[1], aaa, "expected pot/aaa node in right connection");

        // make sure repeated connections aren't attempted
        let result = match dz.parse_line_with_result("co bbb aaa") {
            Ok(_) => false,
            Err(rc) => matches!(rc, ReturnCode::AlreadyConnected),
        };

        assert!(result, "Did not catch AlreadyConnected error");

        // TODO: make test for shortcuts
    }

    #[test]
    fn test_connect_shorthands() {
        let mut dz = DagZet::new();
        dz.parse_line("ns top");

        // Make sure shorthand returns an error if a node
        // isn't selected.
        // Note that it doesn't matter if 'bbb' exist or not
        // those checks don't happen until after all the nodes
        // are created.
        let result = match dz.parse_line_with_result("co $ bbb") {
            Ok(_) => false,
            Err(rc) => matches!(rc, ReturnCode::NodeNotSelected),
        };
        assert!(result, "Did not catch NodeNotSelected error");

        // Make nodes aaa and bbb, then use shorthand to connect
        // bbb -> aaa
        dz.parse_line("nn aaa");
        dz.parse_line("nn bbb");
        dz.parse_line("co $ aaa");

        assert_eq!(dz.connections.len(), 1, "no connections found");
    }
}
