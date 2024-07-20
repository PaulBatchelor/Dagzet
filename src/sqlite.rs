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

#[allow(dead_code)]
#[derive(Clone)]
pub struct Param {
    name: String,
    ptype: ParamType,
}

#[allow(dead_code)]
pub struct Table {
    name: String,
    columns: Vec<Param>,
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

impl SQLize for Table {
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

impl Table {
    pub fn new(name: &str) -> Self {
        Table {
            name: name.to_string(),
            columns: vec![],
        }
    }

    pub fn add_column(&mut self, param: &Param) {
        self.columns.push(param.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn sqlize_table() {
        let mut tab = Table::new("dz_nodes");

        tab.add_column(&Param::new("name", ParamType::TextUnique));
        tab.add_column(&Param::new("id", ParamType::IntegerPrimaryKey));
        tab.add_column(&Param::new("position", ParamType::Integer));

        let expected = concat!(
            "CREATE TABLE IF NOT EXISTS dz_nodes(\n",
            "    name TEXT UNIQUE,\n",
            "    id INTEGER PRIMARY KEY,\n",
            "    position INTEGER\n",
            ")\n"
        );

        assert_eq!(tab.sqlize(), expected);
    }
}
