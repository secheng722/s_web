#[derive(Default,Debug)]
pub struct Node {
    pub pattern: String,
    pub part: String,
    pub children: Vec<Node>,
    pub iswild: bool,
}

impl Node {
    pub fn new() -> Self {
        Node {
            pattern: String::new(),
            part: String::new(),
            children: Vec::new(),
            iswild: false,
        }
    }

    fn match_child(&mut self, path: &str) -> Option<&mut Node> {
        self.children
            .iter_mut()
            .find(|child| child.part == path || child.iswild)
    }

    fn match_children(&self, path: &str) -> Vec<&Node> {
        self.children
            .iter()
            .filter(|&child| child.part == path || child.iswild)
            .collect()
    }

    pub fn insert(&mut self, pattern: &str, parts: Vec<&str>, height: usize) {
        if height == parts.len() {
            self.pattern = pattern.to_string();
            return;
        }

        let part = &parts[height];
        if let Some(child) = self.match_child(part) {
            child.insert(pattern, parts, height + 1);
        } else {
            let mut new_node = Node {
                pattern: String::new(),
                part: part.to_string(),
                children: Vec::new(),
                iswild: part.starts_with(':') || part.starts_with('*'),
            };
            new_node.insert(pattern, parts, height + 1);
            self.children.push(new_node);
        }
    }

    pub fn search(&self, parts: &Vec<&str>, height: usize) -> Option<&Node> {
        if height == parts.len() || self.part.starts_with("*") {
            return if self.pattern.is_empty() {
                None
            } else {
                Some(self)
            };
        }

        let part = &parts[height];
        for child in self.match_children(part) {
            if let Some(result) = child.search(parts, height + 1) {
                return Some(result);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert() {
        let mut root = Node {
            pattern: String::new(),
            part: String::new(),
            children: Vec::new(),
            iswild: false,
        };

        root.insert("/p/:lang/doc", vec!["p", ":lang", "doc"], 0);
        assert_eq!(root.children.len(), 1);
        assert_eq!(root.children[0].part, "p");
        assert!(!root.children[0].iswild);
        assert_eq!(root.children[0].children.len(), 1);
        assert_eq!(root.children[0].children[0].part, ":lang");
        assert!(root.children[0].children[0].iswild);
        println!("{:?}", root);
    }
}
