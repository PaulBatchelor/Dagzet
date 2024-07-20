use std::marker::PhantomData;

#[allow(dead_code)]
#[derive(Clone)]
pub enum ParamType {
    None,
    TextUnique,
    IntegerPrimaryKey,
    Integer,
}

pub trait SQLize {
    fn sqlize(&self) -> String;
}

pub trait Row<T> {
    fn sqlize_values(&self) {}
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct Param {
    name: String,
    ptype: ParamType,
}

#[allow(dead_code)]
pub struct Table<T> {
    name: String,
    columns: Vec<Param>,
    phantom: PhantomData<T>,
}

#[allow(dead_code)]
impl SQLize for ParamType {
    fn sqlize(&self) -> String {
        match self {
            ParamType::TextUnique => "TEXT UNIQUE".to_string(),
            ParamType::IntegerPrimaryKey => "INTEGER PRIMARY KEY".to_string(),
            ParamType::Integer => "INTEGER".to_string(),
            _ => "".to_string(),
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
        sql.push_str("\n)\n");
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

    pub fn sqlize_insert(&self, _row: &impl Row<T>) -> String {
        "".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestTable;

    #[allow(dead_code)]
    struct TestRow {
        name: String,
        id: u32,
        position: u32,
    }

    impl<TestTable> Row<TestTable> for TestRow {}

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
            ")\n"
        );

        assert_eq!(tab.sqlize(), expected);
    }

    #[test]
    #[allow(unused)]
    fn sqlize_insert() {
        let tab = generate_test_table();

        let row = TestRow {
            name: "test".to_string(),
            id: 0,
            position: 0,
        };

        let expected = concat!(
            "INSERT INTO dz_nodes(name, id, position)\n",
            "VALUES('test', 0, 0);"
        );

        assert_eq!(
            tab.sqlize_insert(&row),
            expected,
            "Did not generate expected INSERT statement"
        );
    }
}
