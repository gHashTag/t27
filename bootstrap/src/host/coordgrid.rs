use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coord { pub x: i64, pub y: i64 }

#[derive(Debug, Clone, PartialEq)]
pub enum GridError {
    EntityExists { id: u64 },
    EntityNotFound { id: u64 },
}

impl std::fmt::Display for GridError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GridError::EntityExists { id } => write!(f, "entity {id} exists"),
            GridError::EntityNotFound { id } => write!(f, "entity {id} not found"),
        }
    }
}

impl std::error::Error for GridError {}

struct Entity {
    id: u64,
    pos: Coord,
    cell: (i64, i64),
}

pub struct CoordGrid {
    entities: BTreeMap<u64, Entity>,
    cells: BTreeMap<(i64, i64), Vec<u64>>,
    cell_size: u64,
    total_inserts: u64,
    total_moves: u64,
    total_queries: u64,
}

impl CoordGrid {
    pub fn new(cell_size: u64) -> Self { Self { entities: BTreeMap::new(), cells: BTreeMap::new(), cell_size, total_inserts: 0, total_moves: 0, total_queries: 0 } }

    fn to_cell(&self, pos: &Coord) -> (i64, i64) {
        let cs = self.cell_size as i64;
        (pos.x.div_euclid(cs), pos.y.div_euclid(cs))
    }

    pub fn insert(&mut self, id: u64, pos: Coord) -> Result<(), GridError> {
        if self.entities.contains_key(&id) { return Err(GridError::EntityExists { id }); }
        let cell = self.to_cell(&pos);
        self.entities.insert(id, Entity { id, pos, cell });
        self.cells.entry(cell).or_default().push(id);
        self.total_inserts += 1;
        Ok(())
    }

    pub fn remove(&mut self, id: u64) -> Result<Coord, GridError> {
        let e = self.entities.remove(&id).ok_or(GridError::EntityNotFound { id })?;
        if let Some(cell) = self.cells.get_mut(&e.cell) {
            cell.retain(|&x| x != id);
            if cell.is_empty() { self.cells.remove(&e.cell); }
        }
        Ok(e.pos)
    }

    pub fn move_to(&mut self, id: u64, new_pos: Coord) -> Result<Coord, GridError> {
        let cs = self.cell_size as i64;
        let new_cell = (new_pos.x.div_euclid(cs), new_pos.y.div_euclid(cs));
        let e = self.entities.get_mut(&id).ok_or(GridError::EntityNotFound { id })?;
        let old_pos = e.pos;
        let old_cell = e.cell;
        if old_cell != new_cell {
            drop(e);
            if let Some(cell) = self.cells.get_mut(&old_cell) {
                cell.retain(|&x| x != id);
                if cell.is_empty() { self.cells.remove(&old_cell); }
            }
            self.cells.entry(new_cell).or_default().push(id);
            let e = self.entities.get_mut(&id).unwrap();
            e.pos = new_pos;
            e.cell = new_cell;
        } else {
            e.pos = new_pos;
        }
        self.total_moves += 1;
        Ok(old_pos)
    }

    pub fn pos(&self, id: u64) -> Option<Coord> { self.entities.get(&id).map(|e| e.pos) }

    pub fn query_rect(&mut self, top_left: &Coord, bot_right: &Coord) -> Vec<u64> {
        self.total_queries += 1;
        let cs = self.cell_size as i64;
        let tl = (top_left.x.div_euclid(cs), top_left.y.div_euclid(cs));
        let br = (bot_right.x.div_euclid(cs), bot_right.y.div_euclid(cs));
        let mut result = Vec::new();
        for cx in tl.0..=br.0 {
            for cy in tl.1..=br.1 {
                if let Some(ids) = self.cells.get(&(cx, cy)) {
                    for &id in ids {
                        if let Some(e) = self.entities.get(&id) {
                            if e.pos.x >= top_left.x && e.pos.x <= bot_right.x && e.pos.y >= top_left.y && e.pos.y <= bot_right.y {
                                result.push(id);
                            }
                        }
                    }
                }
            }
        }
        result
    }

    pub fn query_radius(&mut self, center: &Coord, radius: u64) -> Vec<u64> {
        self.total_queries += 1;
        let r = radius as i64;
        let tl = Coord { x: center.x - r, y: center.y - r };
        let br = Coord { x: center.x + r, y: center.y + r };
        let cs = self.cell_size as i64;
        let tl_cell = (tl.x.div_euclid(cs), tl.y.div_euclid(cs));
        let br_cell = (br.x.div_euclid(cs), br.y.div_euclid(cs));
        let r2 = (radius * radius) as i64;
        let mut result = Vec::new();
        for cx in tl_cell.0..=br_cell.0 {
            for cy in tl_cell.1..=br_cell.1 {
                if let Some(ids) = self.cells.get(&(cx, cy)) {
                    for &id in ids {
                        if let Some(e) = self.entities.get(&id) {
                            let dx = e.pos.x - center.x;
                            let dy = e.pos.y - center.y;
                            if dx * dx + dy * dy <= r2 { result.push(id); }
                        }
                    }
                }
            }
        }
        result
    }

    pub fn cell_count(&self) -> usize { self.cells.len() }
    pub fn entity_count(&self) -> usize { self.entities.len() }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_moves(&self) -> u64 { self.total_moves }
    pub fn total_queries(&self) -> u64 { self.total_queries }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_grid() { assert_eq!(CoordGrid::new(100).entity_count(), 0); }

    #[test]
    fn insert_pos() {
        let mut g = CoordGrid::new(100);
        g.insert(1, Coord { x: 50, y: 50 }).unwrap();
        assert_eq!(g.pos(1), Some(Coord { x: 50, y: 50 }));
    }

    #[test]
    fn remove() {
        let mut g = CoordGrid::new(100);
        g.insert(1, Coord { x: 50, y: 50 }).unwrap();
        let pos = g.remove(1).unwrap();
        assert_eq!(pos, Coord { x: 50, y: 50 });
        assert_eq!(g.entity_count(), 0);
    }

    #[test]
    fn move_entity() {
        let mut g = CoordGrid::new(100);
        g.insert(1, Coord { x: 50, y: 50 }).unwrap();
        let old = g.move_to(1, Coord { x: 200, y: 200 }).unwrap();
        assert_eq!(old, Coord { x: 50, y: 50 });
        assert_eq!(g.pos(1), Some(Coord { x: 200, y: 200 }));
    }

    #[test]
    fn query_rect() {
        let mut g = CoordGrid::new(100);
        g.insert(1, Coord { x: 50, y: 50 }).unwrap();
        g.insert(2, Coord { x: 150, y: 150 }).unwrap();
        g.insert(3, Coord { x: 500, y: 500 }).unwrap();
        let found = g.query_rect(&Coord { x: 0, y: 0 }, &Coord { x: 200, y: 200 });
        assert_eq!(found.len(), 2);
    }

    #[test]
    fn query_radius() {
        let mut g = CoordGrid::new(100);
        g.insert(1, Coord { x: 10, y: 10 }).unwrap();
        g.insert(2, Coord { x: 1000, y: 1000 }).unwrap();
        let found = g.query_radius(&Coord { x: 0, y: 0 }, 50);
        assert!(found.contains(&1));
        assert!(!found.contains(&2));
    }

    #[test]
    fn duplicate() {
        let mut g = CoordGrid::new(100);
        g.insert(1, Coord { x: 0, y: 0 }).unwrap();
        let err = g.insert(1, Coord { x: 0, y: 0 }).unwrap_err();
        assert!(matches!(err, GridError::EntityExists { .. }));
    }

    #[test]
    fn not_found() {
        let mut g = CoordGrid::new(100);
        let err = g.remove(99).unwrap_err();
        assert!(matches!(err, GridError::EntityNotFound { .. }));
    }

    #[test]
    fn cell_partitioning() {
        let mut g = CoordGrid::new(100);
        g.insert(1, Coord { x: 50, y: 50 }).unwrap();
        g.insert(2, Coord { x: 150, y: 150 }).unwrap();
        assert_eq!(g.cell_count(), 2);
    }

    #[test]
    fn stats() {
        let mut g = CoordGrid::new(100);
        g.insert(1, Coord { x: 0, y: 0 }).unwrap();
        g.move_to(1, Coord { x: 50, y: 50 }).unwrap();
        g.query_rect(&Coord { x: 0, y: 0 }, &Coord { x: 100, y: 100 });
        assert_eq!(g.total_inserts(), 1);
        assert_eq!(g.total_moves(), 1);
        assert_eq!(g.total_queries(), 1);
    }

    #[test]
    fn error_display() { assert!(GridError::EntityNotFound { id: 3 }.to_string().contains("3")); }
}
