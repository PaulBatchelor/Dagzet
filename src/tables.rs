use crate::escape_quotes;
use crate::sqlite::SQLize;
use crate::DagZet;
use crate::FileRange;
use crate::FlashCard;
use crate::Param;
use crate::{ParamType, Row, Table};
use std::io;
use std::ops::Not;

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

pub struct ConnectionsRow<'a> {
    left: &'a String,
    right: &'a String,
}

impl<ConnectionsTable> Row<ConnectionsTable> for ConnectionsRow<'_> {
    fn sqlize_values(&self) -> String {
        format!("{}, {}", name_lookup(self.left), name_lookup(self.right))
    }
}

impl Default for Table<ConnectionsTable> {
    fn default() -> Self {
        let mut con: Table<ConnectionsTable> = Table::new("dz_connections");
        con.add_column(&Param::new("left", ParamType::IntegerNotNull));
        con.add_column(&Param::new("right", ParamType::IntegerNotNull));
        con
    }
}

impl Generate for Table<ConnectionsTable> {
    fn generate(&self, dz: &DagZet, f: &mut impl io::Write) {
        let _ = f.write_all(&self.sqlize().into_bytes());

        for con in &dz.connections {
            let row = ConnectionsRow {
                left: &con[0],
                right: &con[1],
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
            s.push_str(
                &x.replace('\'', "''")
                    .replace('\\', "\\\\")
                    .replace('\"', "\\\""),
            );
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
        con.add_column(&Param::new("node", ParamType::Integer));
        con.add_column(&Param::new("lines", ParamType::Text));
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
        format!(
            "{}, '{}'",
            name_lookup(self.node),
            escape_quotes(&self.hyperlink)
        )
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
        format!(
            "{}, '{}'",
            name_lookup(self.node),
            escape_quotes(self.todo_item)
        )
    }
}

impl Default for Table<TODOTable> {
    fn default() -> Self {
        let mut con: Table<TODOTable> = Table::new("dz_todo");
        con.add_column(&Param::new("node", ParamType::Integer));
        con.add_column(&Param::new("task", ParamType::Text));
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

pub struct TagsTable;

pub struct TagsRow<'a> {
    node: &'a String,
    tag: &'a String,
}

impl<TagsTable> Row<TagsTable> for TagsRow<'_> {
    fn sqlize_values(&self) -> String {
        format!("{}, '{}'", name_lookup(self.node), self.tag)
    }
}

impl Default for Table<TagsTable> {
    fn default() -> Self {
        let mut con: Table<TagsTable> = Table::new("dz_tags");
        con.add_column(&Param::new("node", ParamType::Integer));
        con.add_column(&Param::new("tag", ParamType::Text));
        con
    }
}

impl Generate for Table<TagsTable> {
    fn generate(&self, dz: &DagZet, f: &mut impl io::Write) {
        let _ = f.write_all(&self.sqlize().into_bytes());

        for (nodeid, tags) in &dz.tags {
            // Insert tags as (node,tag) pairs
            for tag in tags {
                let row = TagsRow {
                    node: &dz.nodelist[*nodeid as usize - 1],
                    tag: &tag,
                };
                let str = self.sqlize_insert(&row).to_string();
                let _ = f.write_all(&str.into_bytes());
            }
        }
    }
}

pub struct FlashCardsTable;

pub struct FlashCardsRow<'a> {
    node: String,
    card: &'a FlashCard,
}

impl<FlashCardsTable> Row<FlashCardsTable> for FlashCardsRow<'_> {
    fn sqlize_values(&self) -> String {
        format!(
            "{}, '{}', '{}'",
            name_lookup(&self.node),
            lines_to_json(&self.card.front),
            lines_to_json(&self.card.back),
        )
    }
}

impl Default for Table<FlashCardsTable> {
    fn default() -> Self {
        let mut con: Table<FlashCardsTable> = Table::new("dz_flashcards");
        con.add_column(&Param::new("node", ParamType::Integer));
        con.add_column(&Param::new("front", ParamType::Text));
        con.add_column(&Param::new("back", ParamType::Text));
        con
    }
}

impl Generate for Table<FlashCardsTable> {
    fn generate(&self, dz: &DagZet, f: &mut impl io::Write) {
        let _ = f.write_all(&self.sqlize().into_bytes());

        for (key, val) in &dz.flashcards {
            let row = FlashCardsRow {
                node: dz.nodelist[*key as usize - 1].to_string(),
                card: val,
            };
            let str = self.sqlize_insert(&row).to_string();
            let _ = f.write_all(&str.into_bytes());
        }
    }
}

pub struct ImagesTable;

pub struct ImagesRow<'a> {
    node: &'a String,
    filename: &'a String,
}

impl<ImagesTable> Row<ImagesTable> for ImagesRow<'_> {
    fn sqlize_values(&self) -> String {
        format!("{}, '{}'", name_lookup(self.node), self.filename)
    }
}

impl Default for Table<ImagesTable> {
    fn default() -> Self {
        let mut con: Table<ImagesTable> = Table::new("dz_images");
        con.add_column(&Param::new("node", ParamType::Integer));
        con.add_column(&Param::new("image", ParamType::Text));
        con
    }
}

impl Generate for Table<ImagesTable> {
    fn generate(&self, dz: &DagZet, f: &mut impl io::Write) {
        let _ = f.write_all(&self.sqlize().into_bytes());
        for (key, val) in &dz.images {
            let row = ImagesRow {
                node: &dz.nodelist[*key as usize - 1].to_string(),
                filename: val,
            };
            let str = self.sqlize_insert(&row).to_string();
            let _ = f.write_all(&str.into_bytes());
        }
    }
}

pub struct AudioTable;

pub struct AudioRow<'a> {
    node: &'a String,
    filename: &'a String,
}

impl<AudioTable> Row<AudioTable> for AudioRow<'_> {
    fn sqlize_values(&self) -> String {
        format!("{}, '{}'", name_lookup(self.node), self.filename)
    }
}

impl Default for Table<AudioTable> {
    fn default() -> Self {
        let mut con: Table<AudioTable> = Table::new("dz_audio");
        con.add_column(&Param::new("node", ParamType::Integer));
        con.add_column(&Param::new("audio", ParamType::Text));
        con
    }
}

impl Generate for Table<AudioTable> {
    fn generate(&self, dz: &DagZet, f: &mut impl io::Write) {
        let _ = f.write_all(&self.sqlize().into_bytes());
        for (key, val) in &dz.images {
            let row = AudioRow {
                node: &dz.nodelist[*key as usize - 1].to_string(),
                filename: val,
            };
            let str = self.sqlize_insert(&row).to_string();
            let _ = f.write_all(&str.into_bytes());
        }
    }
}

pub struct NodeRefsTable;

pub struct NodeRefsRow<'a> {
    node: &'a String,
    filename: &'a String,
    linum: u32,
}

impl<NodeRefsTable> Row<NodeRefsTable> for NodeRefsRow<'_> {
    fn sqlize_values(&self) -> String {
        format!(
            "{}, '{}', {}",
            name_lookup(self.node),
            self.filename,
            self.linum
        )
    }
}

impl Default for Table<NodeRefsTable> {
    fn default() -> Self {
        let mut con: Table<NodeRefsTable> = Table::new("dz_noderefs");
        con.add_column(&Param::new("node", ParamType::Integer));
        con.add_column(&Param::new("filename", ParamType::Text));
        con.add_column(&Param::new("linum", ParamType::Integer));
        con
    }
}

impl Generate for Table<NodeRefsTable> {
    fn generate(&self, dz: &DagZet, f: &mut impl io::Write) {
        self.generate_with_filename(dz, f, None, 0, 0);
    }
}

impl Table<NodeRefsTable> {
    pub fn generate_with_filename(
        &self,
        dz: &DagZet,
        f: &mut impl io::Write,
        filename: Option<&String>,
        start: usize,
        end: usize,
    ) {
        let _ = f.write_all(&self.sqlize().into_bytes());
        let emptystring = "".to_string();
        let filename = match filename {
            Some(x) => x,
            None => &emptystring,
        };

        for (key, val) in &dz.noderefs {
            // TODO: this could be handled better
            if (*key >= start as u32 && *key < end as u32).not() {
                continue;
            }
            let row = NodeRefsRow {
                node: &dz.nodelist[*key as usize - 1].to_string(),
                filename,
                linum: *val,
            };
            let str = self.sqlize_insert(&row).to_string();
            let _ = f.write_all(&str.into_bytes());
        }
    }
}
