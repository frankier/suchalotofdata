Prerequisites
=============

If you have sbt, it should pull everything in -- I think.

Task 3
======

Since this task seems to be about running existing stuff, I decided I'd do it
for a few connectedness measures and compare.

The choice of measures is a subset (those which I could find existing
implementations for) of those in *Graph Connectivity Measures for Unsupervised
Word Sense Disambiguation*.
http://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.77.919&rep=rep1&type=pdf

To run::

  $ sbt "runMain frrobert.jyu.fi.emailgraphmeasures.Connectedness"

The output for me was::

  [info] Vertices: 1005	Edges: 25571
  [info] = Local measures =
  [info] InDegree
  [info] val: 212	id: 160
  [info] val: 179	id: 62
  [info] val: 169	id: 107
  [info] val: 157	id: 121
  [info] val: 154	id: 86
  [info] Elapsed time: 0 m, 0.138452252 s
  [info] Betweenness (Hua's method)
  [info] val: 0.08968015075036585	id: 160
  [info] val: 0.039802101319044034	id: 86
  [info] val: 0.03177242681631218	id: 5
  [info] val: 0.02949145458081336	id: 121
  [info] val: 0.02621800352897648	id: 82
  [info] Elapsed time: 2 m, 59.179510336350 s
  [info] Betweenness (Edmond's method)
  * Many warnings about inefficient joins/bad indexes *
  [info] val: 0.0896801507503658	id: 160
  [info] val: 0.03980210131904402	id: 86
  [info] val: 0.03177242681631214	id: 5
  [info] val: 0.02949145458081338	id: 121
  [info] val: 0.026218003528976425	id: 82
  [info] Elapsed time: 6 m, 2.362271811299 s
  [info] KPP
  [info] val: 9.9601593625498E-4	id: 846
  [info] val: 9.9601593625498E-4	id: 995
  [info] val: 6.000096001536025E-7	id: 160
  [info] val: 5.601889405258607E-7	id: 82
  [info] val: 5.536497700138855E-7	id: 121
  [info] Elapsed time: 0 m, 9.9339151701 s
  * Many warnings about inefficient joins/bad indexes *
  [info] HITS auth
  [info] val: 0.010628804107475636	id: 160
  [info] val: 0.009616666409673472	id: 82
  [info] val: 0.009530347763394299	id: 121
  [info] val: 0.008788064958166695	id: 107
  [info] val: 0.008232597031024696	id: 62
  [info] HITS hub
  [info] val: 0.00722048089228075	id: 160
  [info] val: 0.006898168378816666	id: 107
  [info] val: 0.006695882620596148	id: 62
  [info] val: 0.0064850917182638764	id: 434
  [info] val: 0.006471581103839901	id: 121
  [info] Elapsed time: 0 m, 2.2237295653 s
  [info] PageRank (alpha = 0.05)
  [info] val: 27.87939013626457	id: 1
  [info] val: 19.30754261099934	id: 130
  [info] val: 11.376204578435551	id: 532
  [info] val: 8.96864941610939	id: 227
  [info] val: 6.779544749873288	id: 160
  [info] Elapsed time: 0 m, 17.17270190372 s
  [info] PageRank (alpha = 0.15)
  [info] val: 10.009752972702055	id: 1
  [info] val: 7.314026833644806	id: 130
  [info] val: 6.752994243134354	id: 160
  [info] val: 5.319997390372312	id: 62
  [info] val: 5.12941888144711	id: 86
  [info] Elapsed time: 0 m, 2.2532973861 s
  [info] PageRank (alpha = 0.3)
  [info] val: 5.881909678985159	id: 160
  [info] val: 4.895596272016485	id: 1
  [info] val: 4.5466711944743645	id: 62
  [info] val: 4.463434495483432	id: 86
  [info] val: 4.261242126612235	id: 5
  [info] Elapsed time: 0 m, 2.2392438115 s
  [info] Maxflow not implemented
  [info] Global measures not implemented
  [success] Total time: 628 s, completed 25-Dec-2017 21:28:34

As we increase alpha, we seem to make PageRank more like the other centrality
measures. Vertex 160 has a high in-degree and betweenness (it is on many
shortest paths), but random walks tend to end up on vertex 1 instead. As we
increase the teleportation factor, we end up with something more comparable to
other connectedness measures.

Task 4
======

I experimented with a few changes to simple label propagation (usually referred
to as the Chinese whispers algorithm in NLP for some reason)  and also made
some baselines.

Make baselines::

  $ SBT_OPTS="-Xmx4G" sbt "runMain frrobert.jyu.fi.emailgraphmeasures.Cluster --algorithm random --partitions 1 --output random1.txt"
  $ SBT_OPTS="-Xmx4G" sbt "runMain frrobert.jyu.fi.emailgraphmeasures.Cluster --algorithm random --partitions 42 --output random42.txt"
  $ SBT_OPTS="-Xmx4G" sbt "runMain frrobert.jyu.fi.emailgraphmeasures.Cluster --algorithm stronglyconnectedcomponents --output stronglyconnectedcomponents.txt"
  $ SBT_OPTS="-Xmx4G" sbt "runMain frrobert.jyu.fi.emailgraphmeasures.Cluster --algorithm connectedcomponents --output connectedcomponents.txt"

For evaluation I included normalise mutual information and precision, recall
and F1 measure based on pairwise comparisons. The former is attractive because
it is 0 both in the case that all vertices have their own cluster and in the
case that all vertices share the same cluster. The latter is attractive since
it is standard for other tasks and makes clear an important trade off.

Evaluation::

  $ SBT_OPTS="-Xmx4G" sbt "runMain frrobert.jyu.fi.emailgraphmeasures.ClusterEval random1.txt email-Eu-core-department-labels.txt" 
  [...]
  [info] N: 1005
  [info] NMI: 0.0
  [info] tp, fp, fn, tn: 23544, 480966, 0, 0
  [info] Precision: 0.046667063090919905
  [info] Recall: 0.04895148513616347
  [info] F1: 0.047781985558248
  [...]
  $ SBT_OPTS="-Xmx4G" sbt "runMain frrobert.jyu.fi.emailgraphmeasures.ClusterEval random42.txt email-Eu-core-department-labels.txt" 
  [...]
  [info] N: 1005
  [info] NMI: 0.20930605224279097
  [info] tp, fp, fn, tn: 591, 11329, 22953, 469637
  [info] Precision: 0.04958053691275168
  [info] Recall: 0.017239367598156467
  [info] F1: 0.025583308081901217
  [...]
  $ SBT_OPTS="-Xmx4G" sbt "runMain frrobert.jyu.fi.emailgraphmeasures.ClusterEval stronglyconnectedcomponents.txt email-Eu-core-department-labels.txt" 
  [...]
  [info] N: 1005
  [info] NMI: 0.27372229863428776
  [info] tp, fp, fn, tn: 14982, 307021, 8562, 173945
  [info] Precision: 0.04652751682437741
  [info] Recall: 0.04747404010989185
  [info] F1: 0.04699601308686201
  [...]
  $ SBT_OPTS="-Xmx4G" sbt "runMain frrobert.jyu.fi.emailgraphmeasures.ClusterEval connectedcomponents.txt email-Eu-core-department-labels.txt" 
  [...]
  [info] N: 1005
  [info] NMI: 0.03291866226028445
  [info] tp, fp, fn, tn: 22492, 463113, 1052, 17853
  [info] Precision: 0.04631748025658714
  [info] Recall: 0.04845690648799457
  [info] F1: 0.04736304579003338
  [...]

GraphX's included label propagation::
  $ SBT_OPTS="-Xmx4G" sbt "runMain frrobert.jyu.fi.emailgraphmeasures.Cluster --algorithm labelpropagation --iterations 50 --output labelpropagation.txt"
  $ SBT_OPTS="-Xmx4G" sbt "runMain frrobert.jyu.fi.emailgraphmeasures.ClusterEval labelpropagation.txt email-Eu-core-department-labels.txt"
  [...]
  [info] N: 1005
  [info] NMI: 0.17410002073381123
  [info] tp, fp, fn, tn: 20951, 382935, 2593, 98031
  [info] Precision: 0.05187354847655031
  [info] Recall: 0.05434365337926169
  [info] F1: 0.053079879505557294
  [...]

This outperforms all baselines (marginly) on F1 but not NMI.

Best score for the included label propagation was obtained by running it for
only 2 iterations, this prevents one cluster from totally dominating::

  $ SBT_OPTS="-Xmx4G" sbt "runMain frrobert.jyu.fi.emailgraphmeasures.Cluster --algorithm labelpropagation --iterations 2 --output labelpropagation2.txt"
  $ SBT_OPTS="-Xmx4G" sbt "runMain frrobert.jyu.fi.emailgraphmeasures.ClusterEval labelpropagation2.txt email-Eu-core-department-labels.txt"
  [...]
  [info] N: 1005
  [info] NMI: 0.5251593079954481
  [info] tp, fp, fn, tn: 9058, 40891, 14486, 440075
  [info] Precision: 0.18134497187130874
  [info] Recall: 0.16356971305776766
  [info] F1: 0.17199931640810437
  [...]

The variations I came up with were:

 * Some descriptions of label propagation talk about modifying nodes one at
   a time in an asynchronous fashion. Perhaps a rather than running in
   a totally synchronised like GraphX's built in implementation does, the set
   of nodes could be partitioned into approximately equal parts each partition
   could be modified in turn.
 * Rather than starting by labelling all nodes, start from some set of
   important nodes and expand outwards.
    - In this case, after some number of iterations, labels can be flood filled
      into not yet labelled vertices.
 * Relatedly, run in a simulated annealing inspired fashion where an already
   labelled vertex will only be relabelled with a decaying likelihood.

Semi-asynchronous::

  $ SBT_OPTS="-Xmx4G" sbt "runMain frrobert.jyu.fi.emailgraphmeasures.Cluster --algorithm fancylabelpropagation --partitions 2 --iterations 2 --output labelpropagation2-p2.txt"
  $ SBT_OPTS="-Xmx4G" sbt "runMain frrobert.jyu.fi.emailgraphmeasures.Cluster --algorithm fancylabelpropagation --partitions 10 --iterations 2 --output labelpropagation2-p10.txt"
  $ SBT_OPTS="-Xmx4G" sbt "runMain frrobert.jyu.fi.emailgraphmeasures.Cluster --algorithm fancylabelpropagation --partitions 100 --iterations 2 --output labelpropagation2-p100.txt"
  $ SBT_OPTS="-Xmx4G" sbt "runMain frrobert.jyu.fi.emailgraphmeasures.ClusterEval labelpropagation2-p2.txt email-Eu-core-department-labels.txt"
  [...]
  [info] N: 1005
  [info] NMI: 0.5133627771410734
  [info] tp, fp, fn, tn: 12287, 75143, 11257, 405823
  [info] Precision: 0.14053528537115406
  [info] Recall: 0.14221064814814816
  [info] F1: 0.14136800322153828
  [...]
  $ SBT_OPTS="-Xmx4G" sbt "runMain frrobert.jyu.fi.emailgraphmeasures.ClusterEval labelpropagation2-p10.txt email-Eu-core-department-labels.txt"
  [...]
  [info] N: 1005
  [info] NMI: 0.3291118680856808
  [info] tp, fp, fn, tn: 16749, 236804, 6795, 244162
  [info] Precision: 0.0660571951426329
  [info] Recall: 0.06875643988686325
  [info] F1: 0.06737979531410918
  [...]
  $ SBT_OPTS="-Xmx4G" sbt "runMain frrobert.jyu.fi.emailgraphmeasures.ClusterEval labelpropagation2-p100.txt email-Eu-core-department-labels.txt"
  [...]
  [info] N: 1005
  [info] NMI: 0.09307490541564746
  [info] tp, fp, fn, tn: 21578, 434023, 1966, 46943
  [info] Precision: 0.04736161685334317
  [info] Recall: 0.049492074341325125
  [info] F1: 0.04840341412532667
  [...]

Now repeating the partitions=10 experiments with different seeds::

  $ SBT_OPTS="-Xmx4G" sbt "runMain frrobert.jyu.fi.emailgraphmeasures.Cluster --algorithm fancylabelpropagation --partitions 10 --iterations 2 --seed 1 --output labelpropagation2-p10-s1.txt"
  $ SBT_OPTS="-Xmx4G" sbt "runMain frrobert.jyu.fi.emailgraphmeasures.Cluster --algorithm fancylabelpropagation --partitions 10 --iterations 2 --seed 2 --output labelpropagation2-p10-s2.txt"
  $ SBT_OPTS="-Xmx4G" sbt "runMain frrobert.jyu.fi.emailgraphmeasures.Cluster --algorithm fancylabelpropagation --partitions 10 --iterations 2 --seed 3 --output labelpropagation2-p10-s3.txt"
  $ SBT_OPTS="-Xmx4G" sbt "runMain frrobert.jyu.fi.emailgraphmeasures.ClusterEval labelpropagation2-p10-s1.txt email-Eu-core-department-labels.txt"
  [...]
  [info] N: 1005
  [info] NMI: 0.4612912587635428
  [info] tp, fp, fn, tn: 16978, 139038, 6566, 341928
  [info] Precision: 0.10882217208491436
  [info] Recall: 0.11660393945221285
  [info] F1: 0.11257874146276772
  [...]
  $ SBT_OPTS="-Xmx4G" sbt "runMain frrobert.jyu.fi.emailgraphmeasures.ClusterEval labelpropagation2-p10-s2.txt email-Eu-core-department-labels.txt"
  [...]
  [info] N: 1005
  [info] NMI: 0.19172967468336366
  [info] tp, fp, fn, tn: 20139, 370535, 3405, 110431
  [info] Precision: 0.05154937364656977
  [info] Recall: 0.05385623362036691
  [info] F1: 0.05267756018069248
  [...]
  $ SBT_OPTS="-Xmx4G" sbt "runMain frrobert.jyu.fi.emailgraphmeasures.ClusterEval labelpropagation2-p10-s3.txt email-Eu-core-department-labels.txt"
  [...]
  [info] N: 1005
  [info] NMI: 0.38556461026825484
  [info] tp, fp, fn, tn: 18034, 220644, 5510, 260322
  [info] Precision: 0.07555786457067681
  [info] Recall: 0.07974212262440637
  [info] F1: 0.07759362522373674
  [...]

From this small sample, the results appears as if they are probably usually
worse with this approach. The number of partitions and number of iterations
might be confounding variables.

The "centrality seeded" label propagation did not work very well, failing to
outperform the original simple label propagation. For completeness here is
a couple of example commands::

  $ SBT_OPTS="-Xmx4G" sbt "runMain frrobert.jyu.fi.emailgraphmeasures.Cluster --algorithm centralitylabelpropagation --iterations 2 --output hubsprop-50-2.txt --numcentral 50 --centrality hubs"
  $ SBT_OPTS="-Xmx4G" sbt "runMain frrobert.jyu.fi.emailgraphmeasures.Cluster --algorithm centralitylabelpropagation --iterations 2 --output authsprop-50-2.txt --numcentral 50 --centrality auths"

One possibility of the simulated annealing inspired parameters is to
essentially try and essentially run fractional iterations::

  $ SBT_OPTS="-Xmx4G" sbt "runMain frrobert.jyu.fi.emailgraphmeasures.Cluster --algorithm fancylabelpropagation --iterations 2 --decay-iterations 2 --output labelpropagation2-2d.txt"
  $ SBT_OPTS="-Xmx4G" sbt "runMain frrobert.jyu.fi.emailgraphmeasures.Cluster --algorithm fancylabelpropagation --iterations 1 --decay-iterations 2 --output labelpropagation1-2d.txt"
  $ SBT_OPTS="-Xmx4G" sbt "runMain frrobert.jyu.fi.emailgraphmeasures.Cluster --algorithm fancylabelpropagation --iterations 1 --decay-iterations 3 --output labelpropagation1-3d.txt"
  $ SBT_OPTS="-Xmx4G" sbt "runMain frrobert.jyu.fi.emailgraphmeasures.Cluster --algorithm fancylabelpropagation --iterations 1 --decay-iterations 4 --output labelpropagation1-4d.txt"
  $ SBT_OPTS="-Xmx4G" sbt "runMain frrobert.jyu.fi.emailgraphmeasures.ClusterEval labelpropagation2-2d.txt email-Eu-core-department-labels.txt"
  [...]
  [info] N: 1005
  [info] NMI: 0.5148148884132672
  [info] tp, fp, fn, tn: 10830, 56555, 12714, 424411
  [info] Precision: 0.1607182607405209
  [info] Recall: 0.15634699504829
  [info] F1: 0.15850249535322786
  [...]
  $ SBT_OPTS="-Xmx4G" sbt "runMain frrobert.jyu.fi.emailgraphmeasures.ClusterEval labelpropagation1-2d.txt email-Eu-core-department-labels.txt"
  [...]
  [info] N: 1005
  [info] NMI: 0.5401895089434393
  [info] tp, fp, fn, tn: 4162, 18279, 19382, 462687
  [info] Precision: 0.18546410587763468
  [info] Recall: 0.11051220095058549
  [info] F1: 0.1384978869255599
  [...]
  $ SBT_OPTS="-Xmx4G" sbt "runMain frrobert.jyu.fi.emailgraphmeasures.ClusterEval labelpropagation1-3d.txt email-Eu-core-department-labels.txt"
  [...]
  [info] N: 1005
  [info] NMI: 0.5372224373163769
  [info] tp, fp, fn, tn: 7951, 37473, 15593, 443493
  [info] Precision: 0.17503962662909475
  [info] Recall: 0.14983228432517998
  [info] F1: 0.1614580160422378
  [...]
  $ SBT_OPTS="-Xmx4G" sbt "runMain frrobert.jyu.fi.emailgraphmeasures.ClusterEval labelpropagation1-4d.txt email-Eu-core-department-labels.txt"
  [...]
  [info] N: 1005
  [info] NMI: 0.5160492990137722
  [info] tp, fp, fn, tn: 10316, 57958, 13228, 423008
  [info] Precision: 0.15109705012156896
  [info] Recall: 0.14491613519512264
  [info] F1: 0.14794206224006884
  [...]

So... that didn't seem to significantly improve upon simple label propagation
for 2 iterations either. However I would say I'm now fairly comfortable with
Spark and GraphX, as required.
