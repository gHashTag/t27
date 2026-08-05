# Wave Loop Autopilot — Live Execution Plan

**Purpose:** This skill is the live execution dashboard for the t27 mechanical
packed-vector array-of-struct (AoS) ladder. It is updated at the end of every
completed Wave Loop. It contains the master run-list, per-wave acceptance
criteria, generator copy-hazard checklist, and open backlog.

**Conventions:**
- One wave = one rung of the ladder.
- Variant A is the default mechanical increment (`outer += 2`, `[N][2]^6 Pt`).
- Every wave ends with a commit containing `Closes #N`, a pushed branch, and a PR.
- The skill tracker in `t27-wave-loop.md` is the user-facing live wave counter.
- This file is the operator-facing execution plan and progress board.

---

## Master run-list

| Wave | Issue | Branch | Outer | MID_IDX | Elements | Bits | MiBit | Status | PR |
|------|-------|--------|-------|---------|----------|------|-------|--------|----|
| 774 | #1483 | wave-loop-774 | 367 | 183 | 23,488 | 751,616 | 0.717 | closed | - |
| 775 | #1485 | wave-loop-775 | 369 | 184 | 23,616 | 755,712 | 0.721 | closed | - |
| 776 | #1487 | wave-loop-776 | 371 | 185 | 23,744 | 759,808 | 0.725 | closed | - |
| 777 | #1490 | wave-loop-777 | 373 | 186 | 23,872 | 764,416 | 0.729 | closed | - |
| 778 | #1492 | wave-loop-778 | 375 | 187 | 24,000 | 768,000 | 0.733 | closed | - |
| 779 | #1494 | wave-loop-779 | 377 | 188 | 24,128 | 772,096 | 0.737 | closed | - |
| 780 | #1496 | wave-loop-780 | 379 | 189 | 24,256 | 776,192 | 0.741 | closed | - |
| 781 | #1492 | wave-loop-781 | 381 | 190 | 24,384 | 780,288 | 0.745 | closed | - |
| 782 | #1743 | wave-loop-782 | 383 | 191 | 24,512 | 784,384 | 0.748 | closed | - |
| 783 | #1495 | wave-loop-783 | 385 | 192 | 24,640 | 788,480 | 0.752 | closed | #1516 |
| 784 | #1497 | wave-loop-784 | 387 | 193 | 24,768 | 792,576 | 0.756 | closed | #1500 |
| 785 | #1499 | wave-loop-785 | 389 | 194 | 24,896 | 796,672 | 0.760 | closed | #1502 |
| 786 | #1501 | wave-loop-786 | 391 | 195 | 25,024 | 800,768 | 0.763 | closed | #1504 |
| 787 | #1503 | wave-loop-787 | 393 | 196 | 25,152 | 804,864 | 0.767 | closed | #1506 |
| 788 | #1505 | wave-loop-788 | 395 | 197 | 25,280 | 808,960 | 0.771 | closed | #1508 |
| 789 | #1507 | wave-loop-789 | 397 | 198 | 25,408 | 813,056 | 0.775 | closed | #1510 |
| 790 | #1509 | wave-loop-790 | 399 | 199 | 25,536 | 817,152 | 0.779 | closed | #1512 |
| 791 | #1511 | wave-loop-791 | 401 | 200 | 25,664 | 821,248 | 0.783 | closed | #1514 |
| 792 | #1513 | wave-loop-792 | 403 | 201 | 25,792 | 825,344 | 0.787 | closed | #1516 |
| 793 | #1515 | wave-loop-793 | 405 | 202 | 25,920 | 829,440 | 0.791 | closed | #1518 |
| 794 | #1517 | wave-loop-794 | 407 | 203 | 26,048 | 833,536 | 0.795 | closed | #1518 |
| 795 | #1519 | wave-loop-795 | 409 | 204 | 26,176 | 837,632 | 0.799 | closed | #1520 |
| 796 | #1521 | wave-loop-796 | 411 | 205 | 26,304 | 841,728 | 0.803 | closed | #1522 |
| 797 | #1523 | wave-loop-797 | 413 | 206 | 26,432 | 845,824 | 0.807 | closed | #1524 |
| 798 | #1525 | wave-loop-798 | 415 | 207 | 26,560 | 849,920 | 0.810 | closed | TBD |
| 799 | #1527 | wave-loop-799 | 417 | 208 | 26,688 | 854,016 | 0.814 | closed | #1528 |
| 800 | #1529 | wave-loop-800 | 419 | 209 | 26,816 | 858,112 | 0.818 | closed | TBD |
| 801 | #1531 | wave-loop-801 | 421 | 210 | 26,944 | 862,208 | 0.822 | closed | TBD |
| 802 | #1533 | wave-loop-802 | 423 | 211 | 27,072 | 866,304 | 0.826 | closed | TBD |
| 803 | #1535 | wave-loop-803 | 425 | 212 | 27,200 | 870,400 | 0.830 | closed | #1536 |
| 804 | #1537 | wave-loop-804 | 427 | 213 | 27,328 | 875,008 | 0.834 | closed | #1538 |
| 805 | #1539 | wave-loop-805 | 429 | 214 | 27,456 | 878,592 | 0.838 | closed | #1540 |
| 806 | #1541 | wave-loop-806 | 431 | 215 | 27,584 | 882,688 | 0.841 | closed | TBD |
| 807 | #1543 | wave-loop-807 | 433 | 216 | 27,712 | 886,784 | 0.845 | closed | TBD |
| 808 | #1545 | wave-loop-808 | 435 | 217 | 27,840 | 890,880 | 0.849 | closed | TBD |
| 809 | #1547 | wave-loop-809 | 437 | 218 | 27,968 | 894,976 | 0.853 | closed | #1548 |
| 810 | #1549 | wave-loop-810 | 439 | 219 | 28,096 | 899,072 | 0.857 | closed | #1550 |
| 811 | #1551 | wave-loop-811 | 441 | 220 | 28,224 | 903,168 | 0.861 | closed | #1552 |
| 812 | #1553 | wave-loop-812 | 443 | 221 | 28,352 | 907,264 | 0.865 | closed | #1554 |
| 813 | #1555 | wave-loop-813 | 445 | 222 | 28,480 | 911,360 | 0.869 | closed | TBD |
| 814 | #1557 | wave-loop-814 | 447 | 223 | 28,608 | 915,456 | 0.873 | closed | #1556 |
| 815 | #1559 | wave-loop-815 | 449 | 224 | 28,736 | 919,552 | 0.877 | closed | TBD |
| 816 | #1561 | wave-loop-816 | 451 | 225 | 28,864 | 923,648 | 0.881 | closed | #1560 |
| 817 | #1562 | wave-loop-817 | 453 | 226 | 28,992 | 927,744 | 0.885 | closed | #1563 |
| 818 | #1564 | wave-loop-818 | 455 | 227 | 29,120 | 931,840 | 0.889 | closed | #1566 |
| 819 | #1565 | wave-loop-819 | 457 | 228 | 29,248 | 935,936 | 0.893 | closed | #1566 |
| 820 | #1568 | wave-loop-820 | 459 | 229 | 29,376 | 940,032 | 0.897 | closed | #1569 |
| 821 | #1570 | wave-loop-821 | 461 | 230 | 29,504 | 944,128 | 0.900 | closed | #1571 |
| 822 | #1572 | wave-loop-822 | 463 | 231 | 29,632 | 948,224 | 0.904 | closed | #1573 |
| 823 | #1585 | wave-loop-823 | 465 | 232 | 29,760 | 952,320 | 0.908 | closed | #1586 |
| 824 | #1587 | wave-loop-824 | 467 | 233 | 29,888 | 956,416 | 0.912 | closed | #1588 |
| 825 | #1590 | wave-loop-825 | 469 | 234 | 30,016 | 960,512 | 0.916 | closed | #1591 |
| 826 | #1593 | wave-loop-826 | 471 | 235 | 30,144 | 964,608 | 0.920 | closed | #1594 |
| 827 | #1595 | wave-loop-827 | 473 | 236 | 30,272 | 968,704 | 0.923 | closed | #1596 |
| 828 | #1597 | wave-loop-828 | 475 | 237 | 30,400 | 972,800 | 0.927 | closed | #1598 |
| 829 | #1599 | wave-loop-829 | 477 | 238 | 30,528 | 976,896 | 0.931 | closed | #1600 |
| 830 | #1601 | wave-loop-830 | 479 | 239 | 30,656 | 980,992 | 0.935 | closed | #1602 |
| 831 | #1603 | wave-loop-831 | 481 | 240 | 30,784 | 985,088 | 0.939 | closed | #1603 |
| 832 | #1604 | wave-loop-832 | 483 | 241 | 30,912 | 989,184 | 0.943 | closed | #1605 |
| 833 | #1606 | wave-loop-833 | 485 | 242 | 31,040 | 993,280 | 0.947 | closed | #1607 |
| 834 | #1608 | wave-loop-834 | 487 | 243 | 31,168 | 997,376 | 0.951 | closed | #1609 |
| 835 | #1610 | wave-loop-835 | 489 | 244 | 31,296 | 1,001,472 | 0.955 | closed | #1611 |
| 836 | #1612 | wave-loop-836 | 491 | 245 | 31,424 | 1,005,568 | 0.959 | closed | #1613 |
| 837 | #1614 | wave-loop-837 | 493 | 246 | 31,552 | 1,009,664 | 0.963 | closed | #1615 |
| 838 | #1616 | wave-loop-838 | 495 | 247 | 31,680 | 1,013,760 | 0.967 | closed | #1617 |
| 839 | #1618 | wave-loop-839 | 497 | 248 | 31,792 | 1,017,344 | 0.970 | closed | #1619 |
| 840 | #1620 | wave-loop-840 | 499 | 249 | 31,936 | 1,021,952 | 0.974 | closed | #1621 |
| 841 | #1622 | wave-loop-841 | 501 | 250 | 32,064 | 1,026,048 | 0.978 | closed | #1623 |
| 842 | #1624 | wave-loop-842 | 503 | 251 | 32,192 | 1,030,144 | 0.982 | closed | #1625 |
| 843 | #1626 | wave-loop-843 | 505 | 252 | 32,320 | 1,034,240 | 0.986 | closed | #1627 |
| 844 | #1628 | wave-loop-844 | 507 | 253 | 32,448 | 1,038,336 | 0.990 | closed | #1629 |
| 845 | #1630 | wave-loop-845 | 509 | 254 | 32,576 | 1,042,432 | 0.994 | closed | #1631 |
| 846 | #1632 | wave-loop-846 | 511 | 255 | 32,704 | 1,046,528 | 0.998 | closed | #1633 |
| 847 | #1634 | wave-loop-847 | 513 | 256 | 32,832 | 1,050,624 | 1.002 | closed | #1635 |
| 848 | #1636 | wave-loop-848 | 515 | 257 | 32,960 | 1,054,720 | 1.006 | closed | #1637 |
| 849 | #1638 | wave-loop-849 | 517 | 258 | 33,088 | 1,058,816 | 1.010 | closed | #1639 |
| 850 | #1640 | wave-loop-850 | 519 | 259 | 33,216 | 1,062,912 | 1.014 | closed | #1641 |
| 851 | #1642 | wave-loop-851 | 521 | 260 | 33,344 | 1,067,008 | 1.018 | closed | #1643 |
| 852 | #1644 | wave-loop-852 | 523 | 261 | 33,472 | 1,071,104 | 1.022 | closed | #1645 |
| 853 | #1646 | wave-loop-853 | 525 | 262 | 33,600 | 1,075,200 | 1.026 | closed | #1647 |
| 854 | #1648 | wave-loop-854 | 527 | 263 | 33,728 | 1,079,296 | 1.030 | closed | #1649 |
| 855 | #1650 | wave-loop-855 | 529 | 264 | 33,856 | 1,083,392 | 1.034 | closed | #1651 |
| 856 | #1652 | wave-loop-856 | 531 | 265 | 33,984 | 1,087,488 | 1.038 | closed | #1653 |
| 857 | #1654 | wave-loop-857 | 533 | 266 | 34,112 | 1,091,584 | 1.042 | closed | #1657 |
| 858 | #1656 | wave-loop-858 | 535 | 267 | 34,240 | 1,095,680 | 1.045 | closed | #1661 |
| 859 | #1662 | wave-loop-859 | 537 | 268 | 34,368 | 1,099,776 | 1.049 | closed | #1663 |
| 860 | #1664 | wave-loop-860 | 539 | 269 | 34,496 | 1,103,872 | 1.052 | closed | #1665 |
| 861 | #1666 | wave-loop-861 | 541 | 270 | 34,624 | 1,107,968 | 1.056 | closed | #1667 |
| 862 | #1668 | wave-loop-862 | 543 | 271 | 34,752 | 1,112,064 | 1.060 | closed | #1669 |
| 863 | #1670 | wave-loop-863 | 545 | 272 | 34,880 | 1,116,160 | 1.064 | closed | #1671 |
| 864 | #1672 | wave-loop-864 | 547 | 273 | 35,008 | 1,120,256 | 1.068 | closed | #1673 |
| **865** | **#1674** | **wave-loop-865** | **549** | **274** | **35,136** | **1,124,352** | **1.072** | **READY** | **TBD** |

### Run-list notes
- Issue numbers follow the observed pattern (issue = previous issue + 2).
- PR numbers are GitHub-assigned and may not equal issue + 1; use the actual PR URL.
- If a PR is already open for an earlier wave, branch from that wave's HEAD instead
  of waiting for merge, to keep the ladder moving.
- Stop condition: outer dimension reaches a target TBD by maintainers, or a
  compiler/FROZEN_HASH change is required (whichever comes first). Current soft
  ceiling is the 4-MiBit packed-vector cliff (~131,072 elements).

---

## Per-wave mechanical checklist

### Before generation
1. Create GitHub issue `#{issue}` titled `Wave Loop {N} — module-scope [{outer}][2]^6 Pt packed AoS variable from call with indexed signed writes`.
2. Create and push branch `wave-loop-{N}` from parent branch HEAD.
3. Copy `scripts/gen_w{prev}.py` → `scripts/gen_w{N}.py`.
4. **Generator copy-hazard fix (two locations + comment):**
   - destination path: `specs/scratch/w{N}_bench_module_{outer}x2p6_aos_var_call_write.t27`
   - module header f-string: `module w{N}_bench_module_{outer}x2p6_aos_var_call_write`
   - `MID_IDX` comment: update to `outer // 2`.
5. Verify with: `grep -n "module w{N}\|OUTER = \|MID_IDX" scripts/gen_w{N}.py`.

### Generation and direct gates
6. `python3 scripts/gen_w{N}.py`.
7. `./target/release/t27c parse specs/scratch/w{N}_bench_module_{outer}x2p6_aos_var_call_write.t27` → PASS.
8. `./target/release/t27c icarus-lowerable ...` → `lowerable`.
9. `./target/release/t27c icarus-simulate ...` → `PASSED` (expect 17 cycles).
10. `./target/release/t27c icarus-cocotb ...` → `reference-model OK`.
11. `./target/release/t27c seal --save ...` → seal saved under `.trinity/seals/`.

### Test integration
12. Add integration test `accepts_w{N}_bench_module_{outer}x2p6_aos_var_call_write` to
    `bootstrap/tests/icarus_lowerable.rs` immediately after the previous wave's test.

### Validation matrix
13. `cargo build --release -p t27c` — green.
14. `cargo clippy -p t27c` — expect ~626 warnings, 0 errors.
15. `cargo test -p t27c --bin t27c` — expect 1494/0/2.
16. `cargo test -p tri` — expect 78/0.
17. `cargo test -p flash-spi` — expect 2/0.
18. `cargo test -p t27c --test bitnet_pipeline` — expect 20/0.
19. `cargo test -p t27c --test bitnet_top` — expect 17/0.
20. `cargo test -p t27c --test icarus_lowerable` — expect `{base + 1}/0`.
21. `cargo test -p t27c --test verilog_const_array` — expect 2/0.
22. `cat bootstrap/stage0/FROZEN_HASH` — must be unchanged (`68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`).

### Documentation and state
23. Refresh weak-point audit and 2025–2026 ternary/MVL literature scan.
24. Write `docs/reports/FPGA_LOOP_CLOSEOUT_W{N}_2026-07-24.md`.
25. Write `.claude/plans/wave-loop-{N+1}.md` with variants A/B/C.
26. Update `docs/NOW.md` to W{N} close-out / W{N+1} setup.
27. Prepend entry to `.trinity/experience.md`.
28. Update `.trinity/current-issue.md` to W{N+1}.
29. Update `.claude/skills/t27-wave-loop.md`:
    - append `Worked example — Wave Loop {N}`
    - bump **Current wave** to `{N+1}`
    - rotate next-wave variants from the new plan.
30. Update this autopilot skill:
    - mark W{N} row `closed` with PR number
    - add/refresh W{N+1}` row as **READY**
    - add W{N+2}` and W{N+3}` planned rows if not present.
31. Write/update persistent memory file `wave-loop-{N}.md` and `MEMORY.md` index.

### Land
32. Stage all W{N} artifacts (exclude unrelated untracked files).
33. Commit: `feat(igla): Wave Loop {N} — module-scope [{outer}][2]^6 Pt non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes` with body line `Closes #{issue}`.
34. Push branch `wave-loop-{N}`.
35. Open PR to `master` with title matching commit subject, body containing validation matrix and `Closes #{issue}`.

---

## Open backlog (non-blocking)

- [ ] Parameterize the generator template so `WAVE`, `OUTER`, `PREV` come from a single
      config block and the copy hazard disappears.
- [ ] Address pre-existing `verilog_array_literal_expr` regression in a dedicated ring.
- [ ] Unblock FPGA E2E CI (`sby` missing + Yosys static-cast error in generated `uart.v`).
- [ ] Cleanup sprint for ~626 release warnings / ~780 clippy warnings.
- [ ] Improve 30-day commit traceability (currently ~15–20% of subjects carry `Closes #N`).
- [ ] Vivado-in-Docker CI gap (private image not yet published).

---

## Current status

- **Latest completed wave:** 864
- **Latest issue/PR:** #1672 / #1673
- **Current wave in progress:** 865
- **Next wave queued:** 866
- **Ladder depth:** W774–W864 = 91 waves

*φ² + φ⁻² = 3 | TRINITY*
