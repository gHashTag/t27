# NOW -- the frame address walk as a spec, checked against Vivado (2026-10-03)

## specs/xilinx7/far.t27 -- which frame a configuration bit lands in (Refs #5608)

- A multi-frame FDRI write carries no addresses. The device steps the frame
  address register itself, and a writer (or a reader that wants to say which
  tile a bit belongs to) has to know the order. `far.t27` writes that order as a
  spec, citing prjxray c9f02d857 by file and line. Valid addresses come in
  increasing numeric order: minors of a column, then columns of a row, then rows
  (top half before bottom), then block type. After every (block, half, row)
  group there are two pad frames that belong to no address.
- The FAR layout (`far`, `far_block` ... `far_minor`) moved here from
  `packets.t27`, because t27c has no cross-module `use` yet. It moved rather
  than being copied, so the rule has one home.
- The part table (6 groups, 134 columns, 5408 frames for xc7a35t) sits between
  generated markers. It is rendered from prjxray-db `part.json` by `tri x7-part`
  and checked for drift on every audit, not typed by hand.
- Oracle 1, frame count. All six xc7a35t Vivado bitstreams write FDRI as one
  packet of 5420 frames. The walk produces 5420: 5408 addressed frames plus
  2 pads after each of the 6 groups. The last two pads are written too. prjxray's
  reader stops at the end of the packet, so it never shows them. All 72 pad
  frames (12 per file) are empty in every file.
- Oracle 2, constrained pins. For each package pin a design constrains, take its
  IOB tile from `package_pins.csv` and `tilegrid.json`. The tile must carry data
  in the frames the walk assigns it. 82/82 pins over the six files.
- Oracle 3, segbits. Every set bit of every frame of the six files must be a bit
  that some tile's segbits name (prjxray-db `segbits_*.db`), at the frame the
  walk gives that tile. `*_SING` IO tiles have no segbits file of their own;
  tilegrid's `alias` entry maps them onto `LIOB33`/`LIOI3` with a word shift.
  Result: 4224/4224 set bits named, none outside every tile. Before the alias
  was handled, the same check reported 13 unnamed bits per file, all in `_SING`
  tiles. That is the check working, not noise.
- Mutation check: five deliberate defects, each run through the spec tests and
  all three oracles:
  - one pad frame instead of two;
  - bottom half before top;
  - columns starting at minor 1;
  - an inclusive minor bound;
  - an index that forgets the pads.

  Every one fails both gates. The last needed oracle 3. Forgetting the pads
  only in the index keeps the frame count at 5420 and all 82 pins on frames
  with data, because an IOB tile spans 42 frames and a shift of 2 to 12 frames
  stays inside it. Oracle 3 leaves 501 of 4224 bits unnamed under that defect.
  The inclusive-bound defect also makes the generated Rust index past its table
  in the plain report mode. The other modes still catch it by measurement
  (pins 44/82, segbits 3330/4224).
- Not done here: writing a .bit. With the walk, the packets, the CRC and the
  ECC all stated as specs, the writer (frames -> .bit, compared with
  xc7frames2bit) is next.
