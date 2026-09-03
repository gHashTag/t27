# NOW -- Every gate CI runs, against a tree with nothing in it (2026-09-03)

## Every gate CI runs, against a tree with nothing in it (Refs #2994)

- tri gates empty extracts the exact command lines the workflows write, runs each in a tree holding the scripts and no data, and reports what still exits 0.
- Of 36 runnable invocations, 31 refuse the empty tree and 5 pass -- four are self-contained self-tests and the fifth prints 'tracked files read 0'. This axis is healthy, and a clean audit is a result.
- The population is the COMMAND LINE, not the script: by script name it reads 12 of 38, because seven gates are invoked with --require, which is what turns their SKIP branch into a failure.
