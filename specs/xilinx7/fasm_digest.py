#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
# fasm_digest.py -- prjxray-db for one part, as the text `bitwalk --fasm DIGEST ...` reads.
#
#   python3 specs/xilinx7/fasm_digest.py DB_ROOT PART > PART.digest
#   e.g. DB_ROOT = prjxray-db/artix7, PART = xc7a200tfbg484-2
#
# The standard library only: no prjxray, no fasm, no yaml. What it writes is what prjxray's own
# Database, Grid and TileSegbits(Alias) give fasm2frames (prjxray 132342f7):
#   T tile own_type seg_type [site=alias_site ...]   tilegrid.json; seg_type is the alias type
#   K block base frames offset shift words           one per bits block; block 0 CLB_IO_CLK,
#                                                    1 BLOCK_RAM; shift = 32 * alias start_offset
#   S tile IOB_Yn                                    fasm2frames get_iob_sites, IOB33 tiles
#   F block KEY minor_bit|!minor_bit ...             segbits_<type>[.block_ram].db
#   P KEY                                            ppips_<type>.db
#   R line                                           PART/required_features.fasm
#   B tile bank                                      PART/part.json iobanks, then package_pins.csv
# Order matters in three places, and follows prjxray: F lines of a type come CLB_IO_CLK first,
# so an addressed key in both blocks resolves to BLOCK_RAM (TileSegbits.feature_addresses);
# B lines from part.json come before package_pins.csv (fasm2frames run(), last one wins).
import csv
import json
import os
import re
import sys

BLOCKS = ("CLB_IO_CLK", "BLOCK_RAM")


def yaml_map(path):
    # devices.yaml / parts.yaml are two-level maps of plain scalars; enough for name -> fields.
    out, cur = {}, None
    with open(path) as f:
        for line in f:
            line = line.rstrip()
            if not line.strip() or line.lstrip().startswith("#"):
                continue
            m = re.match(r'^"?([^":\s]+)"?:\s*$', line)
            if m:
                cur = out.setdefault(m.group(1), {})
                continue
            m = re.match(r'^\s+([^:\s]+):\s*"?\'?([^"\']*)"?\'?\s*$', line)
            if m and cur is not None:
                cur[m.group(1)] = m.group(2)
    return out


def fabric_of(db_root, part):
    device = yaml_map(os.path.join(db_root, "mapping", "parts.yaml"))[part]["device"]
    return yaml_map(os.path.join(db_root, "mapping", "devices.yaml"))[device]["fabric"]


def digest(db_root, part, w):
    with open(os.path.join(db_root, fabric_of(db_root, part), "tilegrid.json")) as f:
        grid = json.load(f)

    for name, t in grid.items():
        bits = t.get("bits", {})
        seg, sites = t["type"], {}
        for b in BLOCKS:
            alias = bits.get(b, {}).get("alias")
            if alias:
                seg = alias["type"]
                sites.update(alias["sites"])
        w(" ".join(["T", name, t["type"], seg] + ["%s=%s" % kv for kv in sites.items()]))
        for i, b in enumerate(BLOCKS):
            if b not in bits:
                continue
            k = bits[b]
            # In bits, as frames.t27 seg_pos / seg_in_window take it: the alias start_offset is words.
            shift = 32 * k["alias"]["start_offset"] if "alias" in k else 0
            w("K %d %d %d %d %d %d" % (i, int(k["baseaddr"], 0), k["frames"], k["offset"], shift, k["words"]))
        if "IOB33" in name:
            for site in t["sites"]:
                w("S %s IOB_Y%d" % (name, int(site[-1]) % 2))

    types = sorted(f[len("tile_type_"):-len(".json")].lower()
                   for f in os.listdir(db_root) if f.startswith("tile_type_") and f.endswith(".json"))
    for tt in types:
        for i, suffix in enumerate(("", ".block_ram")):
            path = os.path.join(db_root, "segbits_%s%s.db" % (tt, suffix))
            if not os.path.isfile(path):
                continue
            with open(path) as f:
                for line in f:
                    parts = line.split()
                    if parts:
                        w("F %d %s" % (i, " ".join(parts)))
        path = os.path.join(db_root, "ppips_%s.db" % tt)
        if os.path.isfile(path):
            with open(path) as f:
                for line in f:
                    parts = line.split()
                    if parts:
                        w("P " + parts[0])

    path = os.path.join(db_root, part, "required_features.fasm")
    if os.path.isfile(path):
        with open(path) as f:
            for line in f:
                if line.strip():
                    w("R " + line.strip())

    with open(os.path.join(db_root, part, "part.json")) as f:
        for bank, loc in json.load(f)["iobanks"].items():
            w("B HCLK_IOI3_%s %s" % (loc, bank))
    with open(os.path.join(db_root, part, "package_pins.csv")) as f:
        for pin in csv.DictReader(f):
            w("B %s %s" % (pin["tile"], pin["bank"]))


def main():
    if len(sys.argv) != 3:
        sys.stderr.write("usage: fasm_digest.py DB_ROOT PART > PART.digest\n")
        return 2
    out = sys.stdout
    digest(sys.argv[1], sys.argv[2], lambda s: out.write(s + "\n"))
    return 0


if __name__ == "__main__":
    sys.exit(main())
