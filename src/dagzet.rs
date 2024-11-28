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

impl fmt::Debug for ReturnCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("TODO: implement debug trait")
    }
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

#[allow(dead_code)]
#[derive(Default)]
pub struct FlashCard {
    pub front: Vec<String>,
    pub back: Vec<String>,
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

    pub tags: HashMap<u32, HashSet<String>>,

    // Any nodes used in the "cx" command get stored here
    // External nodes will be ignored by the check_unknown_nodes
    pub xnodes: HashSet<String>,

    pub flashcards: HashMap<u32, FlashCard>,

    // multimedia: images and audio map nodes to filenames
    pub images: HashMap<u32, String>,
    pub audio: HashMap<u32, String>,
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

fn doubledot(fullpath: &str, path: &str) -> String {
    let fullpath: Vec<&str> = fullpath.split('/').collect();
    let path: Vec<&str> = path.split('/').collect();
    let mut out: Vec<&str> = vec![];

    for name in &fullpath {
        out.push(name);
    }

    for name in &path {
        if *name == ".." {
            out.pop();
        } else {
            out.push(name);
        }
    }
    out.join("/")
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

                let use_left_doubledot = connect_args[0].contains("..");
                let use_right_doubledot = connect_args[1].contains("..");

                let shorthand_used = use_left_shorthand || use_right_shorthand;
                let doubledot_used = use_left_doubledot || use_right_doubledot;

                let curnode = if shorthand_used || doubledot_used {
                    match self.curnode {
                        Some(x) => Some(&self.nodelist[x as usize - 1]),
                        None => return Err(ReturnCode::NodeNotSelected),
                    }
                } else {
                    None
                };

                let process_arg = |arg: &str, use_shorthand: bool, use_doubledot: bool| -> String {
                    if use_doubledot {
                        return doubledot(curnode.unwrap(), arg);
                    }

                    if use_shorthand {
                        curnode.unwrap().to_string()
                    } else {
                        let mut outstr = ns.to_string();
                        outstr.push('/');
                        outstr.push_str(arg);
                        outstr
                    }
                };

                let left = process_arg(connect_args[0], use_left_shorthand, use_left_doubledot);
                let right = process_arg(connect_args[1], use_right_shorthand, use_right_doubledot);

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

            "tg" => {
                let curnode = match &self.curnode {
                    Some(id) => *id,
                    _ => return Err(ReturnCode::NodeNotSelected),
                };

                let tagsmap = &mut self.tags;
                let args: Vec<_> = args.split_whitespace().collect();

                let tags = match tagsmap.get_mut(&curnode) {
                    Some(x) => x,
                    None => {
                        tagsmap.insert(curnode, HashSet::new());
                        tagsmap.get_mut(&curnode).unwrap()
                    }
                };

                for arg in &args {
                    if !tags.insert(arg.to_string()) {
                        // TODO: better error handling
                        return Err(ReturnCode::Error);
                    }
                }
            }

            "sn" => {
                let ns = match &self.namespace {
                    Some(n) => n,
                    None => return Err(ReturnCode::NameSpaceNotSet),
                };

                let args: Vec<_> = args.split_whitespace().collect();
                let nodename = format!("{}/{}", ns, args[0]);
                let node_id = match self.nodes.get(&nodename) {
                    Some(x) => x,

                    // TODO: better error handling
                    None => return Err(ReturnCode::Error),
                };

                self.curnode = Some(*node_id);
            }

            "cx" => {
                let args: Vec<_> = args.split_whitespace().collect();
                if args.len() < 2 {
                    return Err(ReturnCode::NotEnoughArgs);
                }

                let mut left = args[0].to_string();
                let mut right = args[1].to_string();

                if left == "$" || right == "$" {
                    let curnode = match &self.curnode {
                        Some(x) => x,
                        None => return Err(ReturnCode::NodeNotSelected),
                    };

                    let curnode = *curnode as usize;

                    if left == "$" {
                        left = self.nodelist[curnode - 1].to_string();
                    } else {
                        right = self.nodelist[curnode - 1].to_string();
                    }
                }

                // shorthand: substitute with previous connection
                if left == "^" || right == "^" {
                    if self.connections.len() == 0 {
                        return Err(ReturnCode::NoConnections);
                    }
                    let cid: usize = self.connections.len() - 1;
                    if left == "^" {
                        left = self.connections[cid][0].to_string();
                    }

                    if right == "^" {
                        right = self.connections[cid][1].to_string();
                    }
                }

                if left.get(0..1).unwrap() == "@" || right.get(0..1).unwrap() == "@" {
                    todo!("Aliases not yet implemented");
                }

                if self.already_connected(&left, &right) {
                    return Err(ReturnCode::AlreadyConnected);
                }
                self.xnodes.insert(left.clone());
                self.xnodes.insert(right.clone());
                self.connections.push([left, right]);
            }

            "ff" => {
                let curnode = match &self.curnode {
                    Some(id) => *id,
                    _ => return Err(ReturnCode::NodeNotSelected),
                };

                let flashcards = &mut self.flashcards;

                let card = match flashcards.get_mut(&curnode) {
                    Some(x) => x,
                    None => {
                        flashcards.insert(curnode, FlashCard::default());
                        flashcards.get_mut(&curnode).unwrap()
                    }
                };

                card.front.push(args.to_string());
            }

            "fb" => {
                let curnode = match &self.curnode {
                    Some(id) => *id,
                    _ => return Err(ReturnCode::NodeNotSelected),
                };

                let flashcards = &mut self.flashcards;

                let card = match flashcards.get_mut(&curnode) {
                    Some(x) => x,
                    None => {
                        flashcards.insert(curnode, FlashCard::default());
                        flashcards.get_mut(&curnode).unwrap()
                    }
                };

                card.back.push(args.to_string());
            }

            "im" => {
                let curnode = match &self.curnode {
                    Some(id) => *id,
                    _ => return Err(ReturnCode::NodeNotSelected),
                };

                let images = &mut self.images;

                images.insert(curnode, args.to_string());
            }

            "eq" => {
                todo!("eq command not yet implemented");
            }

            "pg" => {
                todo!("pg command not yet implemented");
            }

            "al" => {
                todo!("al command not yet implemented");
            }

            "au" => {
                let curnode = match &self.curnode {
                    Some(id) => *id,
                    _ => return Err(ReturnCode::NodeNotSelected),
                };

                self.audio.insert(curnode, args.to_string());
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

            if !self.nodes.contains_key(left) && !self.xnodes.contains(left) {
                unknown_nodes.insert(left.to_string());
            }

            if !self.nodes.contains_key(right) && !self.xnodes.contains(right) {
                unknown_nodes.insert(right.to_string());
            }
        }
        unknown_nodes
    }

    pub fn generate_edges(&self) -> Vec<[u32; 2]> {
        let mut edges = vec![];

        for co in &self.connections {
            let left_id = self.nodes.get(&co[0]);
            let right_id = self.nodes.get(&co[1]);

            if left_id.is_some() && right_id.is_some() {
                let left_id = left_id.unwrap();
                let right_id = right_id.unwrap();
                edges.push([*left_id, *right_id]);
            }
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
            let mut new_no_incoming = HashSet::new();
            for n in &no_incoming {
                let nodesfrom = nodes_connected_to(*n, &edges);

                for m in &nodesfrom {
                    remove_edge(&mut edges, *n, *m);

                    if no_incoming_nodes(*m, &edges) {
                        new_no_incoming.insert(*m);
                    }
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
#[path = "./dagzet_test.rs"]
mod dagzet_test;
