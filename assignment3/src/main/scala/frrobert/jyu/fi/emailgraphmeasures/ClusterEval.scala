package frrobert.jyu.fi.emailgraphmeasures

import org.apache.spark.{SparkConf, SparkContext}
import org.apache.spark.rdd._
import org.apache.spark.graphx.{Graph, Edge, GraphLoader, VertexRDD, VertexId, GraphOps}
import org.apache.spark.graphx.lib.LabelPropagation
import scala.math.log

import Utils.loadLongPairs

object ClusterEval extends App {
  /* Functions for nmi */

  /* There are two types of situations we could have with a clustering of a
   * graph too big to fit on one machine that affect how the following is done.
   *
   * 1. In one extreme the clustering is one cluster containing all vertices;
   * 2. In the other, each vertex has its own cluster.
   *
   * This implementation of nmi handles the first case but not the second. To
   * work in the second case, we would have to find an alternative way of
   * dealing with predClustSizes, predClustSizes, actualClustSizes without
   * loading them onto a single node as they are now. */

  def entropy(g: Map[Long, Long], N: Long): Double = {
    -g.values.map(
       w_k => {
         val prop = w_k.toDouble / N.toDouble
         prop * log(prop)
       }
    ).sum
  }

  def mi(intersectionSizes: Map[(Long, Long), Long], predClustSizes: Map[Long, Long], actualClustSizes: Map[Long, Long], N: Long): Double = {
    intersectionSizes.map {
      case ((predClust, actualClust), intersectionSize) =>
        intersectionSize.toDouble / N.toDouble * log((N * intersectionSize).toDouble / (predClustSizes(predClust) * actualClustSizes(actualClust)).toDouble)
    } sum
  }

  def clustStats(clustering: RDD[(Long, Long)]): (Long, Map[Long, Long]) = {
    // Get number of clusters and map of cluster id to size of each cluster
    val counts = clustering.map(_.swap).countByKey
    (counts.size, counts.toMap)
  }

  def nmi(pred: RDD[(Long, Long)], actual: RDD[(Long, Long)]): Double = {
    val N = actual.count() // == pred.count()
    println(s"N: $N")
    //val predKeyed = pred.map((_, ()))
    //val actualKeyed = actual.map((_, ()))
    // Maps cluster id to size of intersection of pred and actual
    //val intersectionSizes = predKeyed.join(actualKeyed).map(_._1.swap).countByKey.toMap
    val intersectionSizes = pred.join(actual).map(_.swap).countByKey.toMap
    //println(s"intersectionSizes: $intersectionSizes")
    val (predClusts, predClustSizes) = clustStats(pred)
    val (actualClusts, actualClustSizes) = clustStats(actual)
    //println(s"predClustSizes: $predClustSizes")
    //println(s"actualClustSizes: $actualClustSizes")
    mi(intersectionSizes, predClustSizes, actualClustSizes, N) / ((entropy(predClustSizes, N) + entropy(actualClustSizes, N)) / 2)
  }

  /* Function for precision/recall/f-measure */

  def pairwiseCrosstab(pred: RDD[(Long, Long)], actual: RDD[(Long, Long)]):
      (Long, Long, Long, Long) = {
    val joined = pred.join(actual)
    joined.cartesian(joined) filter {
      case ((vid1, _), (vid2, _)) => vid1 < vid2
    } map {
      case ((vid1, (p1, a1)), (vid2, (p2, a2))) => {
        //println(s"p1, a1, p2, a2: $p1, $a1, $p2, $a2")
        if (p1 == p2) {
          if (a1 == a2) {
            (1L, 0L, 0L, 0L)
          } else {
            (0L, 1L, 0L, 0L)
          }
        } else {
          if (a1 == a2) {
            (0L, 0L, 1L, 0L)
          } else {
            (0L, 0L, 0L, 1L)
          }
        }
      }
    } reduce {
      case ((tp1, fp1, fn1, tn1), (tp2, fp2, fn2, tn2)) => {
        (tp1 + tp2, fp1 + fp2, fn1 + fn2, tn1 + tn2)
      }
    }
  }

  def evaluate(pred: RDD[(Long, Long)], actual: RDD[(Long, Long)]) = {
    //val N = actual.count();
    // Often written as c
    //val predGroups = pred.groupBy(_._2).map(_._1)
    // Often written as gamma
    //val actualGroups = actual.groupBy(_._2).map(_._1)

    val nmiVal = nmi(pred, actual)
    println(s"NMI: $nmiVal")

    val (tp, fp, fn, tn) = pairwiseCrosstab(pred, actual)
    println(s"tp, fp, fn, tn: $tp, $fp, $fn, $tn")

    val (tpf, fpf, fnf, tnf) = (tp.toDouble, fp.toDouble, fn.toDouble, fn.toDouble)

    val precision = tpf / (tpf + fpf)
    println(s"Precision: $precision")

    val recall = tpf / (fpf + fnf)
    println(s"Recall: $recall")

    val f_1 = 2 * 1 / (1 / recall + 1 / precision)
    println(s"F1: $f_1")
  }

  val conf = new SparkConf()
    .setMaster("local[4]")
    .setAppName("ClusterEval")
  val sc = new SparkContext(conf)
  sc.setCheckpointDir("checkpoints")
  sc.setLogLevel("WARN")

  val pred = loadLongPairs(sc, args(0))
  val actual = loadLongPairs(sc, args(1))

  evaluate(pred, actual)
}
