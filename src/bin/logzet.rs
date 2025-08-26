use dagzet::logzet::entity::statements_to_entities;
use dagzet::logzet::rows::SessionRows;
use dagzet::logzet::session_tree::entities_to_map;
use dagzet::logzet::session_tree::SessionNode;
use dagzet::logzet::statement::Statement;
use dagzet::logzet::statement::StatementBuilder;
use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;

fn generate_statements<T>(reader: BufReader<T>) -> Vec<Statement>
where
    T: std::io::Read,
{
    let mut builder = StatementBuilder::new();
    for line in reader.lines().map_while(Result::ok) {
        builder.parse(line);
    }
    builder.build()
}

fn generate(stmts: Vec<Statement>) -> Vec<SessionRows> {
    let entities = statements_to_entities(stmts);
    let session_map = entities_to_map(&entities.entities);
    let sessions: Vec<SessionNode> = session_map.into_iter().map(|s| s.into()).collect();
    sessions.iter().map(|s| (&entities, s).into()).collect()
}

fn rows() -> Vec<SessionRows> {
    let mut stdin = false;

    if env::args().len() < 2 {
        stdin = true;
    }

    if stdin {
        let reader = BufReader::new(io::stdin());
        return generate(generate_statements(reader));
    }

    let filenames = env::args().skip(1);

    let mut rows: Vec<SessionRows> = vec![];
    for filename in filenames {
        let f = File::open(filename).unwrap();
        let reader = BufReader::new(f);
        rows.append(&mut generate(generate_statements(reader)));
    }
    rows
}

fn main() {
    let mut f = io::stdout();
    for row in rows() {
        row.generate(&mut f)
    }
}
