use std::collections::BTreeMap;

pub struct AhoCorasick {
    go: Vec<[usize; 256]>,
    fail: Vec<usize>,
    output: Vec<Vec<usize>>,
}

impl AhoCorasick {
    pub fn new(patterns: &[&[u8]]) -> Self {
        let mut ac = Self { go: vec![[0usize; 256]], fail: vec![0], output: vec![Vec::new()] };
        for (pid, pat) in patterns.iter().enumerate() {
            let mut node = 0;
            for &b in *pat {
                let next = ac.go[node][b as usize];
                if next == 0 {
                    ac.go.push([0; 256]);
                    ac.fail.push(0);
                    ac.output.push(Vec::new());
                    let idx = ac.go.len() - 1;
                    ac.go[node][b as usize] = idx;
                    node = idx;
                } else { node = next; }
            }
            ac.output[node].push(pid);
        }
        let mut queue = std::collections::VecDeque::new();
        for b in 0..256u16 {
            let next = ac.go[0][b as usize];
            if next != 0 { queue.push_back(next); }
        }
        while let Some(u) = queue.pop_front() {
            for b in 0..256u16 {
                let v = ac.go[u][b as usize];
                if v != 0 {
                    let mut f = ac.fail[u];
                    while f != 0 && ac.go[f][b as usize] == 0 { f = ac.fail[f]; }
                    ac.fail[v] = if ac.go[f][b as usize] != v { ac.go[f][b as usize] } else { 0 };
                    let fv = ac.fail[v];
                    let mut merged = ac.output[fv].clone();
                    ac.output[v].append(&mut merged);
                    queue.push_back(v);
                } else {
                    ac.go[u][b as usize] = ac.go[ac.fail[u]][b as usize];
                }
            }
        }
        ac
    }

    pub fn search(&self, text: &[u8]) -> Vec<(usize, usize)> {
        let mut result = Vec::new();
        let mut state = 0;
        for (i, &b) in text.iter().enumerate() {
            state = self.go[state][b as usize];
            for &pid in &self.output[state] {
                result.push((pid, i));
            }
        }
        result
    }

    pub fn count(&self, text: &[u8]) -> BTreeMap<usize, usize> {
        let mut counts = BTreeMap::new();
        for (pid, _) in self.search(text) { *counts.entry(pid).or_insert(0) += 1; }
        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_pattern() {
        let ac = AhoCorasick::new(&[b"abc"]);
        let r = ac.search(b"xxabcyyabc");
        assert_eq!(r.len(), 2);
        assert_eq!(r[0], (0, 4));
        assert_eq!(r[1], (0, 9));
    }

    #[test]
    fn multi_pattern() {
        let ac = AhoCorasick::new(&[b"he", b"she", b"his", b"hers"]);
        let r = ac.search(b"ahishers");
        assert!(r.len() >= 4);
    }

    #[test]
    fn overlapping() {
        let ac = AhoCorasick::new(&[b"ab", b"bc", b"abc"]);
        let r = ac.search(b"abc");
        assert!(r.len() >= 2);
    }

    #[test]
    fn no_match() {
        let ac = AhoCorasick::new(&[b"xyz"]);
        assert!(ac.search(b"abcdef").is_empty());
    }

    #[test]
    fn count() {
        let ac = AhoCorasick::new(&[b"aa", b"aaa"]);
        let c = ac.count(b"aaaa");
        assert!(c[&0] >= 2);
    }
}
