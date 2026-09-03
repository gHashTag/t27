# Re-measuring a figure is not judging the issue that carries it

`tri issues dated` reads every open issue whose title states a figure and reports
which of those figures a second reading can actually decide.

    open issues read              486
    no figure in the title        197
    POPULATION (carries a figure) 289
    pins a revision                27
    says as-of / snapshot          31
    already answered in thread     25
    ANCHORED                       83
    free to re-measure            206

An anchored figure disagreeing with today's tree is not a defect. An issue that
pins a revision states a reading *of that revision*; an issue someone already
answered has been judged by a person. Twenty-nine percent of the figures in this
backlog are in that class, and nothing distinguished them before.

The command exists because one of my own verdicts was wrong in exactly this way:
I called #2160's figure stale by re-measuring it to `0`. It pins a snapshot hash,
its own script refuses to run once the corpus moves, the owner had already
commented the new figures, and the lines I read as missing were settled by a
decision the issue *predicted*. `tri issues dated --list` prints #2160 as
`as-of`.

## The rule that decides a revision, and the two floats that broke it

A hex run of 7..=40 characters with at least one letter and one digit is the
shape of an abbreviated commit id. It is also the shape of a chunk of a float:
#2824 prints `s[0] = -1.7594823e-05`, and #2658 lists `` `5.391247e-44` `` inside
backticks, so quoting does not separate them either.

Two rules reject them -- no decimal point before the run, and no `e` immediately
before a sign. Across all 486 open bodies **both** rejections are caught by
**both** rules, so deleting either one leaves every test green. That is a control
that cannot fail, and it was found by mutation, not by reading. The test
`each_float_rule_decides_a_case_the_other_misses` supplies the two inputs that
separate them: `5391247e-44` has no dot, and `1.2345678e12` has no sign for the
`e` to sit before. Removing either rule now turns it red.

Refs #2983
