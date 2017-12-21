Prerequisites
=============

See assignment 1.

Task 1
======

Task 1 can be tested using:

    $ cargo run --bin task1 > task1.dat

Plots can be made using:

    $ Rscript plot.R

> the amount of memory you have set,

I didn't set a memory limit, but since I traced memory usage I just stopped
when it hit 10 GiB.

> the language and set type you used,

Rust and std::collections::BTreeSet

> a chart of the timings, and

To be found in setperf.pdf

> how many elements you could store.

The memory usage goes over 10 GiB between 468 and 469 million elements.

As expected, the time plot is approximately a noisy logarithm, since it's
proportional to the depth of the B-tree. It would probably have significantly
less noise if CPU time given to the process was measured rather than wall clock
time.

Task 2
======
