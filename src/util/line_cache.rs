use std::collections::LinkedList;

#[derive(Debug)]
pub struct LineCache {
    pub data: LinkedList<String>,
    size: usize,
}

impl LineCache {
    pub fn new(size: usize) -> Self {
        Self {
            data: LinkedList::new(),
            size: size,
        }
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn insert(&mut self, string: String) {
        self.data.push_back(string);
        self.trim();
    }

    fn trim(&mut self) {
        while self.data.len() > self.size {
            self.data.pop_front();
        }
    }
}
