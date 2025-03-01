use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;

mod dagzet;
use dagzet::*;

mod sqlite;
use sqlite::*;

mod tables;
use tables::*;

mod trie;

fn parse_file(filename: &str, dz: &mut DagZet) {
    let f = File::open(filename).unwrap();
    let reader = BufReader::new(f);

    let lines_iter = reader.lines().map(|l| l.unwrap());

    let mut linum = 1;

    for str in lines_iter {
        dz.linum = linum;
        let result = dz.parse_line_with_result(&str);

        match result {
            Ok(_) => {}
            Err(rc) => {
                panic!("Error on line {}: {}\nContext:'{}'", linum, rc, &str)
            }
        };
        linum += 1;
    }
}

fn parse_stdin(dz: &mut DagZet) {
    let reader = BufReader::new(io::stdin());

    let lines_iter = reader.lines().map(|l| l.unwrap());

    let mut linum = 1;

    for str in lines_iter {
        dz.linum = linum;
        let result = dz.parse_line_with_result(&str);

        match result {
            Ok(_) => {}
            Err(rc) => {
                panic!("Error on line {}: {}\nContext:'{}'", linum, rc, &str)
            }
        };
        linum += 1;
    }
}

struct FileMapper {
    start: usize,
    end: usize,
}

fn main() {
    let mut stdin = false;

    if env::args().len() < 2 {
        stdin = true;
    }

    let mut dz = DagZet::new();
    let mut file_mappings: Vec<FileMapper> = vec![];

    let mut start = 0;
    let mut end;
    let filenames = env::args().skip(1);

    if stdin {
        parse_stdin(&mut dz);
        end = dz.nodelist.len();
        file_mappings.push(FileMapper { start, end });
    } else {
        for filename in env::args().skip(1) {
            parse_file(&filename, &mut dz);
            end = dz.nodelist.len();
            file_mappings.push(FileMapper { start, end });
            start = end;
        }
    }

    dz.resolve_connections();
    let unknowns = dz.check_unknown_nodes();
    if !unknowns.is_empty() {
        panic!(
            "There were some unknown nodes:\n{}",
            unknowns.into_iter().collect::<Vec<_>>().join("\n")
        );
    }

    let result = dz.check_for_loops(&dz.generate_edges());

    if result.is_err() {
        let found_loops = result.unwrap_err();
        let loop_str = found_loops
            .iter()
            .map(|e| {
                format!(
                    "{} -> {}",
                    dz.nodelist[(e[0] - 1) as usize],
                    dz.nodelist[(e[1] - 1) as usize]
                )
            })
            .collect::<Vec<_>>()
            .join(",\n");
        panic!("Loops found:\n{}", loop_str)
    }

    // Generate nodes table
    let mut f = io::stdout();

    let _ = f.write_all(b"BEGIN;\n");
    let nodes: Table<NodesTable> = Table::default();
    nodes.generate(&dz, &mut f);
    let _ = f.write_all(b"COMMIT;\n");

    let _ = f.write_all(b"BEGIN;\n");

    let connections: Table<ConnectionsTable> = Table::default();
    connections.generate(&dz, &mut f);

    let lines: Table<LinesTable> = Table::default();
    lines.generate(&dz, &mut f);

    let graph_remarks: Table<GraphRemarksTable> = Table::default();
    graph_remarks.generate(&dz, &mut f);

    let connection_remarks: Table<ConnectionRemarksTable> = Table::default();
    connection_remarks.generate(&dz, &mut f);

    let node_remarks: Table<NodeRemarksTable> = Table::default();
    node_remarks.generate(&dz, &mut f);

    let file_ranges: Table<FileRangesTable> = Table::default();
    file_ranges.generate(&dz, &mut f);

    let hyperlinks: Table<HyperlinksTable> = Table::default();
    hyperlinks.generate(&dz, &mut f);

    let todos: Table<TODOTable> = Table::default();
    todos.generate(&dz, &mut f);

    let tags: Table<TagsTable> = Table::default();
    tags.generate(&dz, &mut f);

    let flashcards: Table<FlashCardsTable> = Table::default();
    flashcards.generate(&dz, &mut f);

    let images: Table<ImagesTable> = Table::default();
    images.generate(&dz, &mut f);

    let audio: Table<AudioTable> = Table::default();
    audio.generate(&dz, &mut f);

    let noderefs: Table<NodeRefsTable> = Table::default();
    for (idx, filename) in filenames.enumerate() {
        let mapping = &file_mappings[idx];
        noderefs.generate_with_filename(
            &dz,
            &mut f,
            Some(&filename.to_string()),
            mapping.start + 1,
            mapping.end + 1,
        );
    }

    let attributes: Table<AttributesTable> = Table::default();
    attributes.generate(&dz, &mut f);

    let _ = f.write_all(b"COMMIT;\n");
}
