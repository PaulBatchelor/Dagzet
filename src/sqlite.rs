use std::marker::PhantomData;

#[derive(Clone)]
pub enum ParamType {
    TextUnique,
    IntegerPrimaryKey,
    Integer,
}

pub trait SQLize {
    fn sqlize(&self) -> String;
}

pub trait Row<T> {
    fn sqlize_values(&self) -> String;
}

#[derive(Clone)]
pub struct Param {
    name: String,
    ptype: ParamType,
}

pub struct Table<T> {
    name: String,
    columns: Vec<Param>,
    phantom: PhantomData<T>,
}

impl SQLize for ParamType {
    fn sqlize(&self) -> String {
        match self {
            ParamType::TextUnique => "TEXT UNIQUE".to_string(),
            ParamType::IntegerPrimaryKey => "INTEGER PRIMARY KEY".to_string(),
            ParamType::Integer => "INTEGER".to_string(),
        }
    }
}

impl SQLize for Param {
    fn sqlize(&self) -> String {
        format!("{} {}", self.name, self.ptype.sqlize())
    }
}

impl<T> SQLize for Table<T> {
    fn sqlize(&self) -> String {
        let mut sql = format!("CREATE TABLE IF NOT EXISTS {}(\n", self.name);
        let mut params: Vec<String> = vec![];

        for col in &self.columns {
            params.push(format!("    {}", col.sqlize()));
        }

        sql.push_str(&params.join(",\n"));
        sql.push_str("\n);\n");
        sql
    }
}

impl Param {
    pub fn new(name: &str, ptype: ParamType) -> Self {
        Param {
            name: name.to_string(),
            ptype,
        }
    }
}

impl<T> Table<T> {
    pub fn new(name: &str) -> Self {
        Table::<T> {
            name: name.to_string(),
            columns: vec![],
            phantom: PhantomData,
        }
    }

    pub fn add_column(&mut self, param: &Param) {
        self.columns.push(param.clone());
    }

    pub fn sqlize_insert(&self, row: &impl Row<T>) -> String {
        let mut sql = "".to_string();

        sql.push_str(&format!("INSERT INTO {}(", self.name));

        let mut params: Vec<String> = vec![];

        for col in &self.columns {
            if !matches!(col.ptype, ParamType::IntegerPrimaryKey) {
                params.push(col.name.to_string());
            }
        }

        sql.push_str(&params.join(", "));
        sql.push_str(")\nVALUES(");
        sql.push_str(&row.sqlize_values());
        sql.push_str(");");
        sql
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestTable;

    struct TestRow {
        name: String,
        position: u32,
    }

    impl<TestTable> Row<TestTable> for TestRow {
        fn sqlize_values(&self) -> String {
            format!("'{}', {}", self.name, self.position)
        }
    }

    #[test]
    fn sqlize_param_type() {
        assert_eq!(ParamType::TextUnique.sqlize(), "TEXT UNIQUE".to_string());
    }

    #[test]
    fn sqlize_param() {
        let p = Param {
            ptype: ParamType::TextUnique,
            name: "name".to_string(),
        };

        assert_eq!(p.sqlize(), "name TEXT UNIQUE", "unexpected SQLite code");
    }

    fn generate_test_table() -> Table<TestTable> {
        let mut tab = Table::new("dz_nodes");

        tab.add_column(&Param::new("name", ParamType::TextUnique));
        tab.add_column(&Param::new("id", ParamType::IntegerPrimaryKey));
        tab.add_column(&Param::new("position", ParamType::Integer));

        tab
    }

    #[test]
    fn sqlize_table() {
        let tab = generate_test_table();

        let expected = concat!(
            "CREATE TABLE IF NOT EXISTS dz_nodes(\n",
            "    name TEXT UNIQUE,\n",
            "    id INTEGER PRIMARY KEY,\n",
            "    position INTEGER\n",
            ");\n"
        );

        assert_eq!(tab.sqlize(), expected);
    }

    #[test]
    #[allow(unused)]
    fn sqlize_insert() {
        let tab = generate_test_table();

        let row = TestRow {
            name: "test".to_string(),
            position: 1,
        };

        let expected = concat!(
            "INSERT INTO dz_nodes(name, position)\n",
            "VALUES('test', 1);"
        );

        assert_eq!(
            tab.sqlize_insert(&row),
            expected,
            "Did not generate expected INSERT statement"
        );
    }
}
