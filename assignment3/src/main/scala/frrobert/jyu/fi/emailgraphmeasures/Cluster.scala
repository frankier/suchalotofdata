package frrobert.jyu.fi.emailgraphmeasures

import ml.sparkling.graph.operators.OperatorsDSL._
import org.apache.spark.{SparkConf, SparkContext}
import org.apache.spark.rdd._
import org.apache.spark.graphx.{Graph, Edge, GraphLoader, VertexRDD, VertexId, GraphOps}
import org.apache.spark.graphx.lib.{StronglyConnectedComponents, ConnectedComponents, LabelPropagation}
import scala.math.log
import scala.reflect.ClassTag
import com.google.common.hash.Hashing.murmur3_32

import Utils.saveLongPairs

/*def plp(graph: Graph): Graph {
  graph.mapVertices
}*/

/*
def plp(graph: Graph): Graph {
  graph.pregel([], activeDir=EdgeDirection.Either)(
    (id, label, newLabel) => ,
}*/

object Cluster extends App {

  /*val predictionAndLabels = test.map {
    case LabeledPoint(label, features) =>
      val prediction = model.predict(features)
      (prediction, label)
  }*/

  // 
  def centralVids(graph: Graph[Int, Int], centrality: String, labels: Int): Set[VertexId] = {
    def vertexOrdering[T: Numeric]: Ordering[(VertexId, T)] = {
      Ordering.by[(VertexId, T), T](_._2)
    }
    def topSet[N: Numeric](central: RDD[(VertexId, N)]): Set[VertexId] = {
      implicit val ord = vertexOrdering[N];
      central.top(labels).map {
        case (vid, _) => vid
      }.toSet
    }
    centrality match {
      case "hubs" => {
        val hits = graph.hits().vertices
        topSet(hits.map {
          case (vid, (_auth, hub)) => (vid, hub)
        })
      }
      case "auths" => {
        val hits = graph.hits().vertices
        topSet(hits.map {
          case (vid, (auth, _hub)) => (vid, auth)
        })
      }
      case "outdeg" => topSet(graph.outDegrees)
      case "indeg" => topSet(graph.inDegrees)
    }
  }

  // Parse arguments
  val arglist = args.toList
  type OptionMap = Map[Symbol, Any]

  def nextOption(map : OptionMap, list: List[String]) : OptionMap = {
    list match {
      case Nil => map
      case "--algorithm" :: value :: tail =>
                             nextOption(map ++ Map('algorithm -> value), tail)
      case "--centrality" :: value :: tail =>
                             nextOption(map ++ Map('centrality -> value), tail)
      case "--partitions" :: value :: tail =>
                             nextOption(map ++ Map('partitions -> value.toInt), tail)
      case "--iterations" :: value :: tail =>
                             nextOption(map ++ Map('iterations -> value.toInt), tail)
      case "--decay-iterations" :: value :: tail =>
                             nextOption(map ++ Map('decayiterations -> value.toInt), tail)
      case "--seed" :: value :: tail =>
                             nextOption(map ++ Map('seed -> value.toInt), tail)
      case "--numcentral" :: value :: tail =>
                             nextOption(map ++ Map('numcentral -> value.toInt), tail)
      case "--output" :: value :: tail =>
                             nextOption(map ++ Map('output -> value), tail)
      case option :: tail => println("Unknown option "+option)
                             sys.exit(1)
    }
  }
  val options = nextOption(Map('seed -> 0, 'partitions -> 1, 'iterations -> Int.MaxValue), arglist)

  // Configure Spark
  System.setProperty("spark.executor.memory", "1g")
  System.setProperty("spark.worker.memory", "2g")

  val conf = new SparkConf()
    .setMaster("local[4]")
    .setAppName("Cluster")
  val sc = new SparkContext(conf)
  sc.setCheckpointDir("checkpoints")
  sc.setLogLevel("WARN")
  implicit val ctx: SparkContext = sc

  // Run graph clustering
  val labeledGraph = {
    val graph = GraphLoader.edgeListFile(sc, "email-Eu-core.txt")

    def acceptanceProb() = {
      val iterations = options('iterations).asInstanceOf[Int]
      val decayiterations = 
        if (options contains 'decayiterations) {
          options('decayiterations).asInstanceOf[Int]
        } else {
          0
        }
      val totaliterations = iterations + decayiterations
      if (options contains 'decayiterations) {
        (
          FancyLabelPropagation.sustainDecayProbCurve(iterations, totaliterations) _,
          totaliterations
        )
      } else {
        (FancyLabelPropagation.alwaysAccept _, totaliterations)
      }
    }

    options('algorithm) match {
      case "labelpropagation" => {
        println("Running label propagation")
        LabelPropagation.run(graph, options('iterations).asInstanceOf[Int])
      }
      case "fancylabelpropagation" => {
        println("Running fancy label propagation")
        val (acceptance, totaliterations) = acceptanceProb()
        FancyLabelPropagation.deoptionize(FancyLabelPropagation.run(
          graph,
          options('partitions).asInstanceOf[Int],
          totaliterations,
          options('seed).asInstanceOf[Int],
          acceptanceProb=acceptance
        ))
      }
      case "centralitylabelpropagation" => {
        val labeledVids = centralVids(graph, options('centrality).asInstanceOf[String], options('numcentral).asInstanceOf[Int]);
        val (acceptance, totaliterations) = acceptanceProb()
        FancyLabelPropagation.deoptionize(FancyLabelPropagation.runAuthorities(
          graph,
          labeledVids,
          options('partitions).asInstanceOf[Int],
          totaliterations,
          options('seed).asInstanceOf[Int],
          acceptanceProb=acceptance
        ))
      }
      case "random" => {
        graph mapVertices {
          case (vid, _vlbl) => {
            val seed = options('seed).asInstanceOf[Int]
            val hashFunc = murmur3_32(seed)
            var hasher = hashFunc.newHasher()
            hasher.putLong(vid)
            val h = hasher.hash().asInt() & 0x00000000ffffffffL
            val partitions = options('partitions).asInstanceOf[Int]
            (h * partitions) >> 32L
          }
        }
      }
      /*
      case "betweenness-ish" => {
        val components = ConnectedComponents.run(graph).vertices;
        val biggestComponent = components.map(_._2).countByValue.maxBy(_._2)._1;
        val subgraph = graph.joinVertices(
          components.filter(_._2 == biggestComponent)
        )((vid, attr, _) => (vid, attr));
        val labeledVids = centralVids(subgraph, options('centrality), options('numcentral));
        ShortestPaths.run(subgraph, labeledVids);
      }*/
      case "connectedcomponents" => {
        println("Running connected components")
        ConnectedComponents.run(graph, options('iterations).asInstanceOf[Int])
      }
      case "stronglyconnectedcomponents" => {
        println("Running strongly connected components")
        StronglyConnectedComponents.run(graph, options('iterations).asInstanceOf[Int])
      }
    }
  }

  println("Saving")
  saveLongPairs(sc, options('output).asInstanceOf[String], labeledGraph.vertices)
}
