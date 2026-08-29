# NOW -- The ruler read columns, and the check read no fields (2026-08-29)

## Two wrong rulers before one right answer (Refs #2822)

- "four spaces = top level" reported 43 of 650 specs; two were real -- the corpus writes two indentation conventions, so one column count means opposite things in different files
- bracket depth zero reports 2 and handles both, because `module M;` ends in a semicolon and opens no block
- the tell was in the output: a hit list reading `a@828+838+901+914+...` is a loop variable, not forty redefinitions of `a`
- a field-name filter of lowercase-and-underscore drops `pass_at_1` and `pass_at_5` -- every field the check compares -- so it reported zero numeric drift in a file with five, and looked clean doing it
- caught only because the throwaway scan that found the defect already had an answer; keep the exploratory version until the rewrite agrees with it
