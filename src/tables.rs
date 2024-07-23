use crate::sqlite::SQLize;
use crate::DagZet;
use crate::FileRange;
use crate::Param;
use crate::{ParamType, Row, Table};
use std::io;

pub trait Generate {
    fn generate(&self, dz: &DagZet, f: &mut impl io::Write);
}

pub struct NodesTable;

pub struct NodesRow {
    name: String,
    position: u32,
}

impl<NodesTable> Row<NodesTable> for NodesRow {
    fn sqlize_values(&self) -> String {
        format!("'{}', {}", self.name, self.position)
    }
}

impl Default for Table<NodesTable> {
    fn default() -> Self {
        let mut nodes: Table<NodesTable> = Table::new("dz_nodes");
        nodes.add_column(&Param::new("name", ParamType::TextUnique));
        nodes.add_column(&Param::new("id", ParamType::IntegerPrimaryKey));
        nodes.add_column(&Param::new("position", ParamType::Integer));
        nodes
    }
}

impl Generate for Table<NodesTable> {
    fn generate(&self, dz: &DagZet, f: &mut impl io::Write) {
        let _ = f.write_all(&self.sqlize().into_bytes());

        for (name, id) in dz.nodes.iter() {
            let row = NodesRow {
                name: name.to_string(),
                position: *id,
            };
            let str = self.sqlize_insert(&row).to_string();
            let _ = f.write_all(&str.into_bytes());
        }
    }
}

pub struct ConnectionsTable;

pub struct ConnectionsRow {
    left: u32,
    right: u32,
}

impl<ConnectionsTable> Row<ConnectionsTable> for ConnectionsRow {
    fn sqlize_values(&self) -> String {
        format!("{}, {}", self.left, self.right)
    }
}

impl Default for Table<ConnectionsTable> {
    fn default() -> Self {
        let mut con: Table<ConnectionsTable> = Table::new("dz_connections");
        con.add_column(&Param::new("left", ParamType::Integer));
        con.add_column(&Param::new("right", ParamType::Integer));
        con
    }
}

impl Generate for Table<ConnectionsTable> {
    fn generate(&self, dz: &DagZet, f: &mut impl io::Write) {
        let _ = f.write_all(&self.sqlize().into_bytes());

        // TODO: this was computed already. Reuse instead of
        // generating again.
        let edges = dz.generate_edges();
        for edge in edges {
            let row = ConnectionsRow {
                left: edge[0],
                right: edge[1],
            };
            let str = self.sqlize_insert(&row).to_string();
            let _ = f.write_all(&str.into_bytes());
        }
    }
}

fn name_lookup(name: &String) -> String {
    format!("(SELECT id from dz_nodes WHERE name IS '{name}' LIMIT 1)")
}

fn lines_to_json(lines: &[String]) -> String {
    let mut jsonstr = "[".to_string();

    let vals = lines
        .iter()
        .map(|x| {
            let mut s = "\"".to_string();
            s.push_str(x);
            s.push('"');
            s
        })
        .collect::<Vec<String>>()
        .join(",");
    jsonstr.push_str(&vals);
    jsonstr.push(']');

    jsonstr
}

pub struct LinesTable;

pub struct LinesRow<'a> {
    node: String,
    lines: &'a Vec<String>,
}

impl<LinesTable> Row<LinesTable> for LinesRow<'_> {
    fn sqlize_values(&self) -> String {
        format!(
            "{}, '{}'",
            name_lookup(&self.node),
            lines_to_json(self.lines)
        )
    }
}

impl Default for Table<LinesTable> {
    fn default() -> Self {
        let mut con: Table<LinesTable> = Table::new("dz_lines");
        con.add_column(&Param::new("left", ParamType::Integer));
        con.add_column(&Param::new("right", ParamType::Integer));
        con
    }
}

impl Generate for Table<LinesTable> {
    fn generate(&self, dz: &DagZet, f: &mut impl io::Write) {
        let _ = f.write_all(&self.sqlize().into_bytes());

        for (key, val) in &dz.lines {
            let row = LinesRow {
                node: dz.nodelist[*key as usize - 1].to_string(),
                lines: val,
            };
            let str = self.sqlize_insert(&row).to_string();
            let _ = f.write_all(&str.into_bytes());
        }
    }
}

pub struct GraphRemarksTable;

pub struct GraphRemarksRow<'a> {
    namespace: String,
    remarks: &'a Vec<String>,
}

impl<GraphRemarksTable> Row<GraphRemarksTable> for GraphRemarksRow<'_> {
    fn sqlize_values(&self) -> String {
        format!("'{}', '{}'", self.namespace, lines_to_json(self.remarks))
    }
}

impl Default for Table<GraphRemarksTable> {
    fn default() -> Self {
        let mut con: Table<GraphRemarksTable> = Table::new("dz_graph_remarks");
        con.add_column(&Param::new("namespace", ParamType::Text));
        con.add_column(&Param::new("remarks", ParamType::Text));
        con
    }
}

impl Generate for Table<GraphRemarksTable> {
    fn generate(&self, dz: &DagZet, f: &mut impl io::Write) {
        let _ = f.write_all(&self.sqlize().into_bytes());

        for (key, val) in &dz.graph_remarks {
            let row = GraphRemarksRow {
                namespace: key.to_string(),
                remarks: val,
            };
            let str = self.sqlize_insert(&row).to_string();
            let _ = f.write_all(&str.into_bytes());
        }
    }
}

pub struct ConnectionRemarksTable;

pub struct ConnectionRemarksRow<'a> {
    left: &'a String,
    right: &'a String,
    remarks: &'a Vec<String>,
}

impl<ConnectionRemarksTable> Row<ConnectionRemarksTable> for ConnectionRemarksRow<'_> {
    fn sqlize_values(&self) -> String {
        format!(
            "{}, {}, '{}'",
            name_lookup(self.left),
            name_lookup(self.right),
            lines_to_json(self.remarks)
        )
    }
}

impl Default for Table<ConnectionRemarksTable> {
    fn default() -> Self {
        let mut con: Table<ConnectionRemarksTable> = Table::new("dz_connection_remarks");
        con.add_column(&Param::new("left", ParamType::Integer));
        con.add_column(&Param::new("right", ParamType::Integer));
        con.add_column(&Param::new("remarks", ParamType::Text));
        con
    }
}

impl Generate for Table<ConnectionRemarksTable> {
    fn generate(&self, dz: &DagZet, f: &mut impl io::Write) {
        let _ = f.write_all(&self.sqlize().into_bytes());

        for (key, val) in &dz.connection_remarks {
            let co = &dz.connections[*key];
            let row = ConnectionRemarksRow {
                left: &co[0],
                right: &co[1],
                remarks: val,
            };
            let str = self.sqlize_insert(&row).to_string();
            let _ = f.write_all(&str.into_bytes());
        }
    }
}

pub struct NodeRemarksTable;

pub struct NodeRemarksRow<'a> {
    node: &'a String,
    remarks: &'a Vec<String>,
}

impl<NodeRemarksTable> Row<NodeRemarksTable> for NodeRemarksRow<'_> {
    fn sqlize_values(&self) -> String {
        format!(
            "{}, '{}'",
            name_lookup(self.node),
            lines_to_json(self.remarks)
        )
    }
}

impl Default for Table<NodeRemarksTable> {
    fn default() -> Self {
        let mut con: Table<NodeRemarksTable> = Table::new("dz_remarks");
        con.add_column(&Param::new("node", ParamType::Integer));
        con.add_column(&Param::new("remarks", ParamType::Text));
        con
    }
}

impl Generate for Table<NodeRemarksTable> {
    fn generate(&self, dz: &DagZet, f: &mut impl io::Write) {
        let _ = f.write_all(&self.sqlize().into_bytes());

        for (key, val) in &dz.node_remarks {
            let row = NodeRemarksRow {
                node: &dz.nodelist[*key as usize - 1],
                remarks: val,
            };
            let str = self.sqlize_insert(&row).to_string();
            let _ = f.write_all(&str.into_bytes());
        }
    }
}

pub struct FileRangesTable;

pub struct FileRangesRow<'a> {
    node: &'a String,
    file_range: &'a FileRange,
}

impl<FileRangesTable> Row<FileRangesTable> for FileRangesRow<'_> {
    fn sqlize_values(&self) -> String {
        format!(
            "{}, '{}', {}, {}",
            name_lookup(self.node),
            self.file_range.filename,
            self.file_range.start,
            self.file_range.end
        )
    }
}

impl Default for Table<FileRangesTable> {
    fn default() -> Self {
        let mut con: Table<FileRangesTable> = Table::new("dz_file_ranges");
        con.add_column(&Param::new("node", ParamType::Integer));
        con.add_column(&Param::new("filename", ParamType::Text));
        con.add_column(&Param::new("start", ParamType::Integer));
        con.add_column(&Param::new("end", ParamType::Integer));
        con
    }
}

impl Generate for Table<FileRangesTable> {
    fn generate(&self, dz: &DagZet, f: &mut impl io::Write) {
        let _ = f.write_all(&self.sqlize().into_bytes());

        for (key, val) in &dz.file_ranges {
            let row = FileRangesRow {
                node: &dz.nodelist[*key as usize - 1],
                file_range: val,
            };
            let str = self.sqlize_insert(&row).to_string();
            let _ = f.write_all(&str.into_bytes());
        }
    }
}

pub struct HyperlinksTable;

pub struct HyperlinksRow<'a> {
    node: &'a String,
    hyperlink: &'a String,
}

impl<HyperlinksTable> Row<HyperlinksTable> for HyperlinksRow<'_> {
    fn sqlize_values(&self) -> String {
        format!("{}, '{}'", name_lookup(self.node), self.hyperlink)
    }
}

impl Default for Table<HyperlinksTable> {
    fn default() -> Self {
        let mut con: Table<HyperlinksTable> = Table::new("dz_hyperlinks");
        con.add_column(&Param::new("node", ParamType::Integer));
        con.add_column(&Param::new("hyperlink", ParamType::Text));
        con
    }
}

impl Generate for Table<HyperlinksTable> {
    fn generate(&self, dz: &DagZet, f: &mut impl io::Write) {
        let _ = f.write_all(&self.sqlize().into_bytes());

        for (key, val) in &dz.hyperlinks {
            let row = HyperlinksRow {
                node: &dz.nodelist[*key as usize - 1],
                hyperlink: val,
            };
            let str = self.sqlize_insert(&row).to_string();
            let _ = f.write_all(&str.into_bytes());
        }
    }
}

pub struct TODOTable;

pub struct TODORow<'a> {
    node: &'a String,
    todo_item: &'a String,
}

impl<TODOTable> Row<TODOTable> for TODORow<'_> {
    fn sqlize_values(&self) -> String {
        format!("{}, '{}'", name_lookup(self.node), self.todo_item)
    }
}

impl Default for Table<TODOTable> {
    fn default() -> Self {
        let mut con: Table<TODOTable> = Table::new("dz_todo");
        con.add_column(&Param::new("node", ParamType::Integer));
        con.add_column(&Param::new("hyperlink", ParamType::Text));
        con
    }
}

impl Generate for Table<TODOTable> {
    fn generate(&self, dz: &DagZet, f: &mut impl io::Write) {
        let _ = f.write_all(&self.sqlize().into_bytes());

        for (key, val) in &dz.todos {
            let row = TODORow {
                node: &dz.nodelist[*key as usize - 1],
                todo_item: val,
            };
            let str = self.sqlize_insert(&row).to_string();
            let _ = f.write_all(&str.into_bytes());
        }
    }
}
