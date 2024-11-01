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
fn main() {
    if env::args().len() < 2 {
        println!("Please supply a dagzet file\n");
        return;
    }

    let filename: &str = &env::args().last().unwrap();
    let f = File::open(filename).unwrap();
    let reader = BufReader::new(f);
    let mut dz = DagZet::new();

    let lines_iter = reader.lines().map(|l| l.unwrap());

    for str in lines_iter {
        let result = dz.parse_line_with_result(&str);

        match result {
            Ok(_) => {}
            Err(rc) => {
                panic!("error: {}", rc)
            }
        };
    }

    let unknowns = dz.check_unknown_nodes();
    if !unknowns.is_empty() {
        panic!(
            "There were some unknown nodes: {}",
            unknowns.into_iter().collect::<Vec<_>>().join(", ")
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
                    dz.nodelist[e[0] as usize], dz.nodelist[e[1] as usize]
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        panic!("Loops found: {}", loop_str)
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

    let _ = f.write_all(b"COMMIT;\n");
}
