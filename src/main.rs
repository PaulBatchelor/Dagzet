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
        // TODO: handle error
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
        panic!("There were some unknown nodes");
    }

    let result = dz.check_for_loops(&dz.generate_edges());

    if result.is_err() {
        panic!("Loops found")
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

    let _ = f.write_all(b"COMMIT;\n");
}
