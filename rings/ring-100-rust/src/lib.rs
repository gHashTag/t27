//! ring-100 — **Multi-Chip Mesh**
//!
//! Wave 12 / Track C scaffolding. Implements the *control plane* (routing,
//! topology, hop cost) for a triad of Trinity tiles (`Phi`, `Euler`, `Gamma`)
//! connected over an N×M mesh. No silicon, no FPGA — pure software model.
//!
//! ## Status (honest, mirrors `README.md → Wave 11 / Wave 12`)
//! * **Written** — yes (this file).
//! * **`cargo check`** — not run in authoring sandbox (no toolchain).
//! * **`cargo test`** — must be run by reader / Wave 12 Track D Docker image.
//!
//! ## Identity
//! Anchor: `phi^2 + 1/phi^2 = 3`. Verified in [`Mesh::identity_witness`].

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use core::fmt;

/// Trinity tile role inside the multi-chip mesh.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TileRole {
    /// `tt-trinity-phi` — 1×1 φ-anchor.
    Phi,
    /// `tt-trinity-euler` — 8×2 e-engine, safety & control.
    Euler,
    /// `tt-trinity-gamma` — 8×4 γ-surface, 32-PE ternary mesh.
    Gamma,
}

impl TileRole {
    /// Returns the 3-letter mnemonic used in the on-chip ID register.
    pub const fn mnemonic(self) -> &'static str {
        match self {
            TileRole::Phi => "PHI",
            TileRole::Euler => "EUL",
            TileRole::Gamma => "GAM",
        }
    }
}

/// 2-D lattice coordinate of a tile inside the mesh.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Coord {
    /// Column (X).
    pub x: u16,
    /// Row (Y).
    pub y: u16,
}

impl Coord {
    /// Manhattan distance between two coordinates (hop count in a mesh).
    pub fn manhattan(self, other: Self) -> u32 {
        let dx = (self.x as i32 - other.x as i32).unsigned_abs();
        let dy = (self.y as i32 - other.y as i32).unsigned_abs();
        dx + dy
    }
}

/// A populated cell of the mesh.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tile {
    /// Tile role.
    pub role: TileRole,
    /// Mesh position.
    pub at: Coord,
}

/// Sparse multi-chip mesh.
///
/// Bounds are kept in `width × height` cells. Tiles can be sparse —
/// not every cell needs a tile.
#[derive(Debug, Clone)]
pub struct Mesh {
    /// Mesh width in tile slots.
    pub width: u16,
    /// Mesh height in tile slots.
    pub height: u16,
    tiles: alloc_vec_polyfill::Vec<Tile>,
}

impl Mesh {
    /// Construct an empty mesh of the given size.
    pub fn new(width: u16, height: u16) -> Self {
        Self { width, height, tiles: alloc_vec_polyfill::Vec::new() }
    }

    /// Place a tile. Returns `Err` if the coordinate is out of bounds or already
    /// occupied.
    pub fn place(&mut self, tile: Tile) -> Result<(), MeshError> {
        if tile.at.x >= self.width || tile.at.y >= self.height {
            return Err(MeshError::OutOfBounds);
        }
        if self.tiles.iter().any(|t| t.at == tile.at) {
            return Err(MeshError::Occupied);
        }
        self.tiles.push(tile);
        Ok(())
    }

    /// Tile at the given coordinate, if any.
    pub fn at(&self, c: Coord) -> Option<Tile> {
        self.tiles.iter().copied().find(|t| t.at == c)
    }

    /// All tiles, in placement order.
    pub fn tiles(&self) -> &[Tile] {
        &self.tiles
    }

    /// Hop cost between two tiles assuming XY-routing on the mesh.
    pub fn hop_cost(&self, a: Coord, b: Coord) -> u32 {
        a.manhattan(b)
    }

    /// Identity witness: `phi^2 + 1/phi^2 == 3` (exact in f64 within 1e-15).
    ///
    /// This is the *only* gate every Trinity component shares; failing it
    /// means the build is corrupted and the mesh must refuse to route.
    pub fn identity_witness() -> bool {
        // phi = (1 + sqrt(5)) / 2
        let phi = (1.0_f64 + 5.0_f64.sqrt()) / 2.0;
        let lhs = phi * phi + 1.0 / (phi * phi);
        (lhs - 3.0).abs() < 1e-15
    }

    /// Returns `true` iff the mesh contains at least one of each role
    /// (Phi + Euler + Gamma) — i.e. a valid triad is reachable.
    pub fn is_triad_complete(&self) -> bool {
        let has = |r: TileRole| self.tiles.iter().any(|t| t.role == r);
        has(TileRole::Phi) && has(TileRole::Euler) && has(TileRole::Gamma)
    }
}

/// Mesh placement / routing errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshError {
    /// Coordinate exceeds mesh bounds.
    OutOfBounds,
    /// Cell is already occupied by another tile.
    Occupied,
}

impl fmt::Display for MeshError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MeshError::OutOfBounds => f.write_str("mesh coordinate out of bounds"),
            MeshError::Occupied => f.write_str("mesh cell already occupied"),
        }
    }
}

/// Tiny std-free `Vec` polyfill so this crate stays portable.
mod alloc_vec_polyfill {
    extern crate alloc;
    pub use alloc::vec::Vec;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_witness_holds() {
        // The most important invariant in the whole repo.
        assert!(Mesh::identity_witness(), "phi^2 + 1/phi^2 must equal 3");
    }

    #[test]
    fn place_and_route_triad() {
        let mut mesh = Mesh::new(4, 4);
        mesh.place(Tile { role: TileRole::Phi, at: Coord { x: 0, y: 0 } }).unwrap();
        mesh.place(Tile { role: TileRole::Euler, at: Coord { x: 1, y: 0 } }).unwrap();
        mesh.place(Tile { role: TileRole::Gamma, at: Coord { x: 3, y: 2 } }).unwrap();

        assert!(mesh.is_triad_complete());
        assert_eq!(mesh.tiles().len(), 3);
        // phi -> gamma: |3-0| + |2-0| = 5 hops via XY routing.
        assert_eq!(
            mesh.hop_cost(Coord { x: 0, y: 0 }, Coord { x: 3, y: 2 }),
            5
        );
    }

    #[test]
    fn rejects_out_of_bounds_and_duplicate() {
        let mut mesh = Mesh::new(2, 2);
        let t = Tile { role: TileRole::Phi, at: Coord { x: 5, y: 5 } };
        assert_eq!(mesh.place(t), Err(MeshError::OutOfBounds));

        let ok = Tile { role: TileRole::Phi, at: Coord { x: 0, y: 0 } };
        mesh.place(ok).unwrap();
        let dup = Tile { role: TileRole::Euler, at: Coord { x: 0, y: 0 } };
        assert_eq!(mesh.place(dup), Err(MeshError::Occupied));
    }

    #[test]
    fn mnemonics_are_3_chars_uppercase() {
        for r in [TileRole::Phi, TileRole::Euler, TileRole::Gamma] {
            let m = r.mnemonic();
            assert_eq!(m.len(), 3);
            assert!(m.chars().all(|c| c.is_ascii_uppercase()));
        }
    }
}
