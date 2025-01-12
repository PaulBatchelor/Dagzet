function table_count() {
    TABLE=$1
    COUNT=$2
RC=$(
sqlite3 a.db <<EOM
SELECT count(*) from $TABLE;
EOM
)
    if [ ! "$RC" -eq "$COUNT" ]
    then
        echo "Table $TABLE: expected $COUNT rows, got $RC"
        exit 1
    fi
}

> a.db
cargo run test.dz | sqlite3 a.db

table_count dz_images 1
table_count dz_audio 1
table_count dz_attributes 2
