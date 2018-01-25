package frrobert.jyu.fi.emailgraphmeasures

import ml.sparkling.graph.operators.OperatorsDSL._
import ml.sparkling.graph.operators.measures.vertex.betweenness.edmonds.EdmondsBC
import ml.sparkling.graph.operators.measures.vertex.betweenness.hua.HuaBC
import org.apache.spark.rdd._
import org.apache.spark.{SparkConf, SparkContext}
import org.apache.spark.graphx.{Graph, Edge, GraphLoader, VertexRDD, VertexId, GraphOps}

object Connectedness extends App {
  def time[R](block: => R): R = {  
      val t0 = System.nanoTime()
      val result = block    // call-by-name
      val t1 = System.nanoTime()
      val int = (t1 - t0)
      val secs = int / 1000000000
      val mins = secs / 60
      val remain_secs = secs % 60
      val nanos = "%09d".format(int)
      println(s"Elapsed time: $mins m, $remain_secs.$nanos s")
      result
  }

  def vertex_ordering[T: Numeric]: Ordering[(VertexId, T)] = {
    Ordering.by[(VertexId, T), T](_._2)
  }

  def top5[VD: Numeric](ranks: RDD[(VertexId, VD)]) = {
    implicit val ord = vertex_ordering[VD];
    for ((vid, rank) <- ranks.top(5)) {
      println(s"val: $rank\tid: $vid")
    }
  }

  val conf = new SparkConf()
    .setMaster("local[8]")
    .setAppName("Connectedness")
  val sc = new SparkContext(conf)
  sc.setCheckpointDir("checkpoints")
  sc.setLogLevel("WARN")
  val graph = GraphLoader.edgeListFile(sc, "email-Eu-core.txt")
  // = Basic metrics =
  val v = graph.vertices.count()
  val e = graph.edges.count()
  println(s"Vertices: $v\tEdges: $e")
  // = Local =
  println("= Local measures =");
  // InDegree
  time {
    println("InDegree");
    top5[Int](graph.inDegrees)
  }
  // Betweenness
  time {
    println("Betweenness (Hua's method)");
    top5[Double](HuaBC.computeBC(graph))
  }
  time {
    println("Betweenness (Edmond's method)");
    top5[Double](EdmondsBC.computeBC(graph))
  }
  // KPP
  time {
    println("KPP");
    val kpp = graph.closenessCentrality().vertices.map {
      case (k, w) => (k, w / (graph.vertices.count() - 1))
    }
    top5[Double](kpp)
  }
  // HITS
  time {
    val hits = graph.hits().vertices
    println("HITS auth");
    val hits_auth = hits.map {
      case (vid, (auth, hub)) => (vid, auth)
    }
    top5[Double](hits_auth)
    println("HITS hub");
    val hits_hub = hits.map {
      case (vid, (auth, hub)) => (vid, hub)
    }
    top5[Double](hits_hub)
  }
  // PageRank
  time {
    println("PageRank (alpha = 0.05)");
    top5[Double](graph.pageRank(0.001, 0.05).vertices)
  }
  time {
    println("PageRank (alpha = 0.15)");
    top5[Double](graph.pageRank(0.001, 0.15).vertices)
  }
  time {
    println("PageRank (alpha = 0.3)");
    top5[Double](graph.pageRank(0.001, 0.3).vertices)
  }
  // Maxflow
  println("Maxflow not implemented");
  // = Global =
  println("Global measures not implemented");
  // Compactness
  // GraphEntropy
  // EdgeDense
}
