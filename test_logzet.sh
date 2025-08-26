LOG=logs/logzet.txt
sqlite3 test.db <<EOM
DROP TABLE IF EXISTS lz_sessions;
DROP TABLE IF EXISTS lz_entities;
DROP TABLE IF EXISTS lz_entries;
DROP TABLE IF EXISTS lz_blocks;
DROP TABLE IF EXISTS lz_connections;
DROP TABLE IF EXISTS lz_tags;
EOM
cargo run --bin logzet $LOG | sqlite3 test.db
