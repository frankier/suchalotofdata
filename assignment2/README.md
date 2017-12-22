Prerequisites
=============

See assignment 1. In addition Python is required to run experiments and make
plots. Use pipenv to pull in extra requirements. After obtained pipenv, you can
run for example:

 $ pipenv install --three

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

I chose to implement K-min-values and HyperLogLog.

The can be run like:
 $ cargo run --release --bin task2 kmv 3

and:
 $ cargo run --release --bin task2 hll 3

respectively, where the arguments are k and p respectively.

You can run the RMSE experiment using 

 $ pipenv run python rmse.py > rmse.dat

rmse.dat now contains JSON with RMSE values corresponding to space usages of
2^n 64-bit words where n = 3..10

Now you can generate a plot by running

 $ pipenv run python plot.py

The plot is now in rmse.pdf

Some values are, for a space usage of 8 64-bit words, with a stream length of
10 million, kmv has a RMSE of about 3 million, and hll has a RMSE of about half
a million.

Bonus: There is a Jupyter notebook using the SageMath kernel: alpha_m.ipynb to
calculate the bias values for HyperLogLog. The main reason I did this was to
see if it would have been possible to avoid part at least part of the integral
analysis towards the end of the HyperLogLog paper by "just calculating it
instead". It turns out it is, at least this time.
