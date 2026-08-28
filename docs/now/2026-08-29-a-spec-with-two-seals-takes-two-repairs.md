# NOW -- A spec with two seals takes two repairs (2026-08-29)

## A spec with two seals takes two repairs (Refs #2767)

- seal --save writes .trinity/seals/<path-derived>.json and 547 specs also carry an older module-name seal naming the same spec_path
- 31 of those pairs already disagree about the five hashes they both claim to describe
- tri seals twins reports them; which of two is the truth is a decision, so the command does not write
