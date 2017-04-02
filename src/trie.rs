struct Node {
    letter: char,
    children: [Option<Box<Node>>; 26],
    end_of_word: bool,
}

struct Trie {
    root: [Option<Box<Node>>; 26],
}
const empty_array: [Option<Box<Node>>; 26] = [None; 26];

impl Node {
    fn new(l: char, eow: bool) -> Node {
        Node { letter: l, children: [None; 26], end_of_word: eow, }
    }

    fn insert(&self, word: &str) {

    }
}

// consists of wrappers around functions on Nodes to hide internal structure of trie
impl Trie {
    fn new() -> Trie {
        Trie { root: [None; 26], }
    }

    // this is so not idiomatic
    fn insert(&self, word: &str) {
        if word == "" {
            return;
        }

        let index = word.chars().nth(0).unwrap() as usize - ('a' as usize);
        let eow = if word.len() == 4 { true } else { false };
        if let Some(n) = self.root[index] {
            n.end_of_word = eow;
        } else {
            self.root[index] = Some(Box::new(Node::new(word.chars().nth(0).unwrap(), eow)));
        }

        if !eow {
            self.root[index].unwrap().insert(&word[4..]);
        }
    }
/*
    fn delete(&self, word: &str) -> bool {
        self.root.delete(word)
    }

    fn words_with_prefix(&self, prefix: &str) -> Vec<String> {
        vec![]
    }*/
}
