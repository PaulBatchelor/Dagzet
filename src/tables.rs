use std::io;

use crate::sqlite::SQLize;
use crate::DagZet;
use crate::Param;
use crate::{ParamType, Row, Table};

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

pub struct LinesTable;

pub struct LinesRow<'a> {
    node: String,
    lines: &'a Vec<String>,
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
        .join(", ");
    jsonstr.push_str(&vals);
    jsonstr.push(']');

    jsonstr
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
