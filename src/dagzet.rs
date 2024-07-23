use core::fmt;
use std::collections::hash_map;
use std::collections::HashMap;
use std::collections::HashSet;

pub enum ReturnCode {
    Okay,
    Error,
    InvalidCommand,
    NameSpaceNotSet,
    NodeAlreadyExists,
    NodeNotSelected,
    NotEnoughArgs,
    AlreadyConnected,
    NoConnections,
}

impl fmt::Display for ReturnCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ReturnCode::Okay => write!(f, "Everything is okay!"),
            ReturnCode::Error => write!(f, "Something went wrong."),
            ReturnCode::InvalidCommand => write!(f, "Invalid command"),
            ReturnCode::NameSpaceNotSet => write!(f, "Namespace not set"),
            ReturnCode::NodeAlreadyExists => write!(f, "Node Already Exists"),
            ReturnCode::NodeNotSelected => write!(f, "Node Not Selected."),
            ReturnCode::NotEnoughArgs => write!(f, "Not Enough arguments"),
            ReturnCode::AlreadyConnected => write!(f, "Already connected"),
            ReturnCode::NoConnections => write!(f, "No connections made"),
        }
    }
}

#[allow(dead_code)]
pub struct FileRange {
    pub filename: String,
    pub start: i32,
    pub end: i32,
}

#[derive(Default)]
pub struct DagZet {
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

    /// Each line can have text content called "lines" (ln)
    pub lines: HashMap<u32, Vec<String>>,

    /// Edges of the knowledge graph. These are represented
    /// as strings instead of IDs so they can be resolved
    /// later. This allows connections to be made before
    /// nodes are made, which is more flexible.
    pub connections: Vec<[String; 2]>,

    /// Remarks can be made about last connection made
    pub connection_remarks: HashMap<usize, Vec<String>>,

    /// Remarks can be made about last node selected
    pub node_remarks: HashMap<u32, Vec<String>>,

    /// tie a node to a range of lines of a file
    pub file_ranges: HashMap<u32, FileRange>,

    last_filename: Option<String>,

    // Tie a hyperlink URL to a node. One per node.
    pub hyperlinks: HashMap<u32, String>,

    // Add a TODO item, one per node
    pub todos: HashMap<u32, String>,
}

fn does_loop_exist(edges: &Vec<[u32; 2]>, a: u32, b: u32) -> bool {
    for edge in edges {
        if edge[0] == b && edge[1] == a {
            return true;
        }
    }
    false
}

fn remove_edge(edges: &mut Vec<[u32; 2]>, a: u32, b: u32) {
    let mut edges_to_remove = vec![];
    for (idx, edge) in edges.iter_mut().enumerate() {
        if edge[0] == a && edge[1] == b {
            edges_to_remove.push(idx);
        }
    }

    for idx in edges_to_remove {
        edges.remove(idx);
    }
}

fn no_incoming_nodes(node: u32, edges: &Vec<[u32; 2]>) -> bool {
    for edge in edges {
        if edge[1] == node {
            return false;
        }
    }
    true
}

fn nodes_connected_to(node: u32, edges: &Vec<[u32; 2]>) -> HashSet<u32> {
    let mut connected: HashSet<u32> = HashSet::new();

    for edge in edges {
        if edge[0] == node {
            connected.insert(edge[1]);
        }
    }

    connected
}

impl DagZet {
    // TODO: deprecate new()
    pub fn new() -> Self {
        DagZet::default()
    }

    #[allow(dead_code)]
    pub fn parse_line(&mut self, line: &str) {
        let _ = self.parse_line_with_result(line);
    }

    pub fn parse_line_with_result(&mut self, line: &str) -> Result<ReturnCode, ReturnCode> {
        if line.is_empty() {
            return Ok(ReturnCode::Okay);
        }

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

                let use_left_shorthand = connect_args[0] == "$";
                let use_right_shorthand = connect_args[1] == "$";

                let curnode = if use_left_shorthand || use_right_shorthand {
                    match self.curnode {
                        Some(x) => Some(&self.nodelist[x as usize - 1]),
                        None => return Err(ReturnCode::NodeNotSelected),
                    }
                } else {
                    None
                };

                let process_arg = |arg: &str, use_shorthand: bool| -> String {
                    if use_shorthand {
                        curnode.unwrap().to_string()
                    } else {
                        let mut outstr = ns.to_string();
                        outstr.push('/');
                        outstr.push_str(arg);
                        outstr
                    }
                };

                let left = process_arg(connect_args[0], use_left_shorthand);
                let right = process_arg(connect_args[1], use_right_shorthand);

                if self.already_connected(&left, &right) {
                    return Err(ReturnCode::AlreadyConnected);
                }

                self.connections.push([left, right]);
            }
            "cr" => {
                if self.connections.is_empty() {
                    return Err(ReturnCode::NoConnections);
                }

                let cid = self.connections.len() - 1;

                if let hash_map::Entry::Vacant(e) = self.connection_remarks.entry(cid) {
                    e.insert(vec![args.to_string()]);
                } else {
                    let rm = &mut self.connection_remarks.get_mut(&cid).unwrap();
                    rm.push(args.to_string());
                }
            }
            "zz" => {}
            "rm" => {
                let curnode = match &self.curnode {
                    Some(id) => *id,
                    _ => return Err(ReturnCode::NodeNotSelected),
                };
                let remarks = &mut self.node_remarks;

                match remarks.get_mut(&curnode) {
                    Some(ln) => ln.push(args.to_string()),
                    None => {
                        remarks.insert(curnode, vec![args.to_string()]);
                    }
                }
            }
            "fr" => {
                let curnode = match &self.curnode {
                    Some(id) => *id,
                    _ => return Err(ReturnCode::NodeNotSelected),
                };

                let args: Vec<_> = args.split_whitespace().collect();

                if args.is_empty() {
                    return Err(ReturnCode::NotEnoughArgs);
                }

                let filename = if args[0] == "$" {
                    match &self.last_filename {
                        Some(x) => x.to_string(),
                        // TODO: better error handling
                        None => return Err(ReturnCode::Error),
                    }
                } else {
                    args[0].to_string()
                };

                let (start, end) = if args.len() == 2 {
                    let start = match args[1].to_string().parse::<i32>() {
                        Ok(x) => x,

                        // TODO: better error handling
                        Err(_) => return Err(ReturnCode::Error),
                    };
                    (start, -1)
                } else if args.len() >= 3 {
                    let start = match args[1].to_string().parse::<i32>() {
                        Ok(x) => x,

                        // TODO: better error handling
                        Err(_) => return Err(ReturnCode::Error),
                    };
                    let end = match args[2].to_string().parse::<i32>() {
                        Ok(x) => x,
                        Err(_) => return Err(ReturnCode::Error),
                    };
                    (start, end)
                } else {
                    (-1, -1)
                };

                if start >= 0 && end >= 0 && start > end {
                    return Err(ReturnCode::Error);
                }

                self.last_filename = Some(filename.clone());

                self.file_ranges.insert(
                    curnode,
                    FileRange {
                        filename,
                        start,
                        end,
                    },
                );
            }

            "hl" => {
                let args: Vec<_> = args.split_whitespace().collect();
                if args.is_empty() {
                    // realistically, this error will never happen
                    return Err(ReturnCode::NotEnoughArgs);
                }

                let curnode = match &self.curnode {
                    Some(id) => *id,
                    _ => return Err(ReturnCode::NodeNotSelected),
                };
                self.hyperlinks.insert(curnode, args[0].to_string());
            }

            "td" => {
                if args.is_empty() {
                    // realistically, this error will never happen
                    return Err(ReturnCode::NotEnoughArgs);
                }
                let curnode = match &self.curnode {
                    Some(id) => *id,
                    _ => return Err(ReturnCode::NodeNotSelected),
                };
                self.todos.insert(curnode, args.to_string());
            }

            _ => return Err(ReturnCode::InvalidCommand),
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

    pub fn check_unknown_nodes(&self) -> HashSet<String> {
        let mut unknown_nodes = HashSet::new();

        for co in &self.connections {
            let left = &co[0];
            let right = &co[1];

            if !self.nodes.contains_key(left) {
                unknown_nodes.insert(left.to_string());
            }

            if !self.nodes.contains_key(right) {
                unknown_nodes.insert(right.to_string());
            }
        }
        unknown_nodes
    }

    pub fn generate_edges(&self) -> Vec<[u32; 2]> {
        let mut edges = vec![];

        for co in &self.connections {
            let left_id = self.nodes.get(&co[0]).unwrap();
            let right_id = self.nodes.get(&co[1]).unwrap();
            edges.push([*left_id, *right_id]);
        }

        edges
    }

    #[allow(dead_code)]
    pub fn check_for_loops(&mut self, edges: &[[u32; 2]]) -> Result<ReturnCode, Vec<[u32; 2]>> {
        // Generate set of nodes
        let mut nodelist: HashSet<u32> = HashSet::new();

        for id in self.nodes.values() {
            nodelist.insert(*id);
        }

        // deep copy edge
        let mut edges = edges.to_owned();

        // Determine initial set of nodes with no incoming nodes
        let mut no_incoming: HashSet<u32> = HashSet::new();

        for node in &nodelist {
            if no_incoming_nodes(*node, &edges) {
                no_incoming.insert(*node);
            }
        }

        // Main Loop

        while !no_incoming.is_empty() {
            let new_no_incoming = HashSet::new();
            for n in &no_incoming {
                let nodesfrom = nodes_connected_to(*n, &edges);

                for m in &nodesfrom {
                    remove_edge(&mut edges, *n, *m);
                }
            }
            no_incoming = new_no_incoming;
        }
        // Look for any remaining edges
        if !edges.is_empty() {
            // Check remaining edges for loops
            let mut found_loops: Vec<[u32; 2]> = vec![];
            for edge in &edges {
                if does_loop_exist(&edges, edge[0], edge[1]) {
                    found_loops.push(*edge);
                }
            }

            // Keep track of loops found and return.
            if !found_loops.is_empty() {
                return Err(found_loops);
            } else {
                // If there are remaining edges but no loops,
                // panic. That's weird and probably shouldn't happen?
                panic!("Not sure why there are remaining edges.");
            }
        }

        Ok(ReturnCode::Okay)
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

        let co = &dz.connections[0];
        // Test lefthand shorthand
        assert_eq!(co[0], "top/bbb", "left shorthand does not work");

        // Test righthand shorthand for bbb -> ccc
        dz.parse_line("nn ccc");
        dz.parse_line("co bbb $");

        let co = &dz.connections[1];
        assert_eq!(co[1], "top/ccc", "right shorthand does not work");
    }

    #[test]
    fn test_connection_remarks() {
        let mut dz = DagZet::new();

        let result = match dz.parse_line_with_result("cr no connections have been made yet") {
            Ok(_) => false,
            Err(rc) => matches!(rc, ReturnCode::NoConnections),
        };
        assert!(result, "Did not catch NoConnections error");

        dz.parse_line("ns top");
        dz.parse_line("co aaa bbb");
        dz.parse_line("cr this is a remark");

        // make sure connection remark is made
        assert_eq!(
            dz.connection_remarks.len(),
            1,
            "Expected a connection remark to appear."
        );

        // Make sure appending works

        dz.parse_line("cr this is a remark on another line");

        // grab the connection remark, make sure appending works

        let co = dz.connection_remarks.get(&0).unwrap();

        assert_eq!(co.len(), 2, "Expected 2 lines in this remark");

        assert_eq!(co[0], "this is a remark");
        assert_eq!(co[1], "this is a remark on another line");
    }

    #[test]
    fn test_invalid_command() {
        let mut dz = DagZet::new();

        let result = match dz.parse_line_with_result("xx this isn't a real command") {
            Ok(_) => false,
            Err(rc) => matches!(rc, ReturnCode::InvalidCommand),
        };
        assert!(result, "Did not catch InvalidCommand error");
    }

    #[test]
    fn test_unknown_nodes() {
        let mut dz = DagZet::new();

        dz.parse_line("ns top");
        dz.parse_line("nn aaa");
        dz.parse_line("nn bbb");

        dz.parse_line("co aaa bbb");
        dz.parse_line("co aaa ccc");
        dz.parse_line("co ccc ddd");

        let unknown = dz.check_unknown_nodes();

        assert_eq!(unknown.len(), 2, "Wrong number of expected nodes");

        assert!(unknown.contains("top/ccc"));
        assert!(unknown.contains("top/ddd"));
    }

    #[test]
    fn test_check_for_loops() {
        let mut dz = DagZet::new();

        dz.parse_line("ns top");
        dz.parse_line("nn aaa");
        dz.parse_line("nn bbb");

        dz.parse_line("co aaa bbb");
        dz.parse_line("co bbb aaa");

        assert_eq!(dz.check_unknown_nodes().len(), 0, "Found unknown nodes");
        let edges = dz.generate_edges();

        assert!(dz.check_for_loops(&edges).is_err(), "Did not catch cycles");
    }

    #[test]
    fn test_comments() {
        let mut dz = DagZet::new();

        let result = dz.parse_line_with_result("zz this is a comment").is_ok();
        assert!(result, "Did not properly ignore comment");
    }

    #[test]
    fn test_node_remarks() {
        let mut dz = DagZet::new();
        // attempt to parse lines without select a node
        dz.parse_line("ns aaa");

        let caught_missing_node = match dz.parse_line_with_result("rm hello remark") {
            Ok(_) => false,
            Err(rc) => matches!(rc, ReturnCode::NodeNotSelected),
        };

        assert!(
            caught_missing_node,
            "tried to make a remark on unselected node"
        );

        let mut dz = DagZet::new();
        // attempt to parse lines without select a node
        dz.parse_line("ns aaa");
        dz.parse_line("nn bbb");
        dz.parse_line("rm ccc");
        dz.parse_line("rm another line");

        // Make sure the remarks are behaving as expected.
        assert_eq!(dz.node_remarks.len(), 1, "couldn't find remarks");

        let node_id = dz.nodes.get("aaa/bbb").unwrap();

        if let Some(rm) = dz.lines.get(node_id) {
            assert_eq!(rm.len(), 2);
            assert_eq!(rm[0], "ccc");
            assert_eq!(rm[1], "another line");
        }
    }
    #[test]
    fn test_file_range() {
        let mut dz = DagZet::new();
        dz.parse_line("ns aaa");
        dz.parse_line("nn bbb");
        dz.parse_line("fr foo 1 4");
        let fr = &dz.file_ranges[&dz.curnode.unwrap()];
        assert!(
            fr.start == 1 && fr.end == 4,
            "could not properly handle full file range"
        );

        // make sure file range is in the valid order
        let mut dz = DagZet::new();
        dz.parse_line("ns aaa");
        dz.parse_line("nn bbb");
        let result = dz.parse_line_with_result("fr foo 4 1");
        assert!(result.is_err(), "wrong order for line not caught");

        // make sure file range is valid number
        let mut dz = DagZet::new();
        dz.parse_line("ns aaa");
        dz.parse_line("nn bbb");
        let result = dz.parse_line_with_result("fr foo one 4");
        assert!(
            result.is_err(),
            "didn't catch invalid numbers for file range"
        );

        // file range with one line
        let mut dz = DagZet::new();
        dz.parse_line("ns aaa");
        dz.parse_line("nn bbb");
        dz.parse_line("fr foo 4");
        let fr = &dz.file_ranges[&dz.curnode.unwrap()];
        assert!(
            fr.start == 4 && fr.end == -1,
            "could not handle file range with one line"
        );

        // file range with no lines (whole file)
        let mut dz = DagZet::new();
        dz.parse_line("ns aaa");
        dz.parse_line("nn bbb");
        dz.parse_line("fr foo");
        let fr = &dz.file_ranges[&dz.curnode.unwrap()];
        assert!(
            fr.start == -1 && fr.end == -1,
            "could not handle file range with one line"
        );

        // shorthand working as expected
        let mut dz = DagZet::new();
        dz.parse_line("ns aaa");
        dz.parse_line("nn bbb");
        dz.parse_line("fr foo 1 4");
        dz.parse_line("nn ccc");
        dz.parse_line("fr $ 3 5");
        let fr = &dz.file_ranges[&dz.curnode.unwrap()];
        assert!(
            fr.start == 3 && fr.end == 5,
            "could not handle shorthand as expected"
        );

        // attempt shorthand without setting file beforehand
        let mut dz = DagZet::new();
        dz.parse_line("ns aaa");
        dz.parse_line("nn bbb");
        let result = dz.parse_line_with_result("fr $ 1 4");
        assert!(
            result.is_err(),
            "shorthand did not fail as it was supposed to"
        );
    }

    #[test]
    fn test_hyperlinks() {
        // Test usuual functionality
        let mut dz = DagZet::new();
        dz.parse_line("ns links");
        dz.parse_line("nn internet_archive");
        dz.parse_line("hl http://archive.org");

        assert_eq!(
            dz.hyperlinks.len(),
            1,
            "Expected exactly one entry in hyperlinks"
        );

        let curnode = &dz.curnode.unwrap();

        let hl = &dz.hyperlinks[curnode];

        assert_eq!(hl, "http://archive.org", "wrong hyperlink found");

        // Test hyperlink without node selected
        let mut dz = DagZet::new();
        dz.parse_line("ns links");
        let result = dz.parse_line_with_result("hl http://archive.org");

        assert!(
            result.is_err_and(|x| { matches!(x, ReturnCode::NodeNotSelected) }),
            "Did not catch NodeNotSelected"
        );
    }

    #[test]
    fn test_todo() {
        // make sure default behavior works
        let mut dz = DagZet::new();
        dz.parse_line("ns top");
        dz.parse_line("nn aaa");
        dz.parse_line("td todo item");

        assert_eq!(dz.todos.len(), 1, "Expected TODO item");

        let curnode = &dz.curnode.unwrap();

        let todostr = &dz.todos[curnode];

        assert_eq!(todostr, "todo item", "incorrect TODO item found");

        let mut dz = DagZet::new();
        dz.parse_line("ns top");
        let result = dz.parse_line_with_result("td todo item");

        assert!(
            result.is_err_and(|x| { matches!(x, ReturnCode::NodeNotSelected) }),
            "Did not catch NodeNotSelected"
        );
    }
}
