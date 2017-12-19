Prerequisites
=============

The only prerequisites should be Rust and its package manager Cargo, which
usually come together. Cargo will pull in other requirements. The simplest way
to get started is with the [rustup script](https://rustup.rs/).

Producing the plot additionally requires R and the ggplot and reshape packages. The packages can be installed by running:

    > install.packages("ggplot2")
    > install.packages("reshape")

within R.

All command lines assume you have the directories enron1-6 under the current directory.

Task 0
======

Task 0 can be tested using the following:

    $ cargo run make enron1 exact.dat

A good starting point to read the code would be:

    fn make_knn_model...

Task 1
======

Task 1 can be tested using:

    $ cargo run make enron1 exact.dat
    $ cargo run match_exact enron2/spam/0134.2002-05-06.SA_and_HP.spam.txt exact.dat

A good starting point to read the code would be:

    fn match_exact...

Task 2
======

Task 2 can be tested using:

    $ cargo run make_approx enron1 exact.dat
    $ cargo run match_approx enron2/spam/0134.2002-05-06.SA_and_HP.spam.txt exact.dat

A good starting point to read the code would be:

    fn make_approx_knn_model...
    fn match_approx...

Task 3
======

Task 3 can be tested using:

    $ cargo run task3 .
    $ Rscript chart.R

A place to start reading the code is:

    fn task3...

Task 4
======

You can run this task using:

    $ cargo run task4 .

A place to start reading the code is:

    fn task4...

The nearest 1 strategy is used. I also collected numbers for which no
prediction was made because there was no result in the 4 bands, 16 rows setup.

The results are:

true_ham: 256 false_ham: 386 true_spam: 3668 false_spam: 1241 no_pred: 14
