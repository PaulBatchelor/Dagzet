use std::collections::BTreeMap;

#[derive(Default)]
pub struct TrieNode {
    pub is_end: bool,
    pub children: BTreeMap<String, TrieNode>,
    pub traversed: u32,
}

#[derive(Default)]
pub struct Trie {
    root: TrieNode,
}

#[allow(dead_code)]
impl Trie {
    pub fn add_path(&mut self, path: &str) {
        let mut node = &mut self.root;

        // append in reverse order for suffix lookup

        for word in path.split('/').rev() {
            node.traversed += 1;
            node = node.children.entry(word.to_string()).or_default();
        }

        node.is_end = true;
    }

    pub fn search(&self, suffix: &str) -> Result<String, &'static str> {
        let mut path: Vec<String> = vec![];

        let mut node = &self.root;

        // search in reversed order for suffix search

        for word in suffix.split('/').rev() {
            match node.children.get(word) {
                Some(n) => {
                    node = n;
                    path.push(word.to_string());
                }
                None => return Err("Could not suffix"),
            }
        }

        if node.traversed > 1 {
            return Err("Ambiguous suffix");
        }

        while !node.is_end {
            let (key, value) = node.children.first_key_value().unwrap();
            path.push(key.to_string());
            node = value;
        }

        Ok(path.into_iter().rev().collect::<Vec<_>>().join("/"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic() {
        let mut root = Trie::default();

        root.add_path("a/b/c");

        let res = root.search("b/c").unwrap();

        assert_eq!(&res, "a/b/c");
    }

    #[test]
    fn invalid_key() {
        let mut root = Trie::default();
        root.add_path("a/b/c");
        let res = root.search("d/e");
        assert!(res.is_err());
    }

    #[test]
    fn amibguous() {
        // match
        let mut root = Trie::default();
        root.add_path("a/b/c");
        root.add_path("f/a/b/c");

        root.add_path("d/e");
        root.add_path("f/a/d/e");

        // Ambiguous pattern
        let res = root.search("b/c");
        assert!(res.is_err());

        // Unambiguous pattern
        let res = root.search("a/d/e").unwrap();
        assert_eq!(res, "f/a/d/e");
    }
}
