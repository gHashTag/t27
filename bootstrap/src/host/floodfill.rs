use std::collections::VecDeque;

pub struct FloodFill {
    grid: Vec<Vec<u8>>,
    rows: usize,
    cols: usize,
    total_fills: u64,
    total_cells_filled: u64,
}

impl FloodFill {
    pub fn new(rows: usize, cols: usize) -> Self {
        Self { grid: vec![vec![0u8; cols]; rows], rows, cols, total_fills: 0, total_cells_filled: 0 }
    }

    pub fn set(&mut self, r: usize, c: usize, val: u8) { if r < self.rows && c < self.cols { self.grid[r][c] = val; } }
    pub fn get(&self, r: usize, c: usize) -> u8 { if r < self.rows && c < self.cols { self.grid[r][c] } else { 0 } }

    pub fn fill(&mut self, sr: usize, sc: usize, new_val: u8) -> usize {
        self.total_fills += 1;
        if sr >= self.rows || sc >= self.cols { return 0; }
        let old_val = self.grid[sr][sc];
        if old_val == new_val { return 0; }
        let mut count = 0usize;
        let mut queue = VecDeque::new();
        queue.push_back((sr, sc));
        while let Some((r, c)) = queue.pop_front() {
            if r >= self.rows || c >= self.cols { continue; }
            if self.grid[r][c] != old_val { continue; }
            self.grid[r][c] = new_val;
            count += 1;
            queue.push_back((r.wrapping_sub(1), c));
            queue.push_back((r + 1, c));
            queue.push_back((r, c.wrapping_sub(1)));
            queue.push_back((r, c + 1));
        }
        self.total_cells_filled += count as u64;
        count
    }

    pub fn count_val(&self, val: u8) -> usize { self.grid.iter().flat_map(|r| r.iter()).filter(|&&v| v == val).count() }
    pub fn rows(&self) -> usize { self.rows }
    pub fn cols(&self) -> usize { self.cols }
    pub fn total_fills(&self) -> u64 { self.total_fills }
    pub fn total_cells_filled(&self) -> u64 { self.total_cells_filled }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_all() {
        let mut f = FloodFill::new(3, 3);
        assert_eq!(f.fill(0, 0, 1), 9);
        assert_eq!(f.count_val(1), 9);
    }

    #[test]
    fn partial() {
        let mut f = FloodFill::new(3, 3);
        for r in 0..3 { for c in 0..3 { f.set(r, c, 1); } }
        f.set(1, 1, 0);
        assert_eq!(f.fill(1, 1, 2), 1);
        assert_eq!(f.count_val(1), 8);
    }

    #[test]
    fn boundary() {
        let mut f = FloodFill::new(5, 5);
        for i in 0..5 { f.set(0, i, 1); f.set(4, i, 1); f.set(i, 0, 1); f.set(i, 4, 1); }
        assert_eq!(f.fill(2, 2, 5), 9);
        assert_eq!(f.count_val(5), 9);
    }

    #[test]
    fn same_val() {
        let mut f = FloodFill::new(3, 3);
        assert_eq!(f.fill(0, 0, 0), 0);
    }

    #[test]
    fn out_of_bounds() { assert_eq!(FloodFill::new(3, 3).fill(10, 10, 1), 0); }

    #[test]
    fn stats() {
        let mut f = FloodFill::new(3, 3);
        f.fill(0, 0, 1);
        assert_eq!(f.total_fills(), 1);
        assert_eq!(f.total_cells_filled(), 9);
    }
}
