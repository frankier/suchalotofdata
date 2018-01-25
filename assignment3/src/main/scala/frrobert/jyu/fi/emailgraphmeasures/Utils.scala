package frrobert.jyu.fi.emailgraphmeasures

import org.apache.spark.{SparkConf, SparkContext}
import org.apache.spark.rdd._
import org.apache.spark.graphx.{Graph, Edge, GraphLoader, VertexRDD, VertexId, GraphOps}
import org.apache.spark.graphx.lib.LabelPropagation
import org.apache.hadoop.fs.{FileSystem, Path}
import java.io.BufferedOutputStream
import scala.math.log

object Utils {
  def loadLongPairs(sc: SparkContext, fn: String): RDD[(Long, Long)] = {
    sc.textFile(fn).map {
      line => {
        val bits = line.split("\\s+")
        assert(bits.size == 2)
        (bits(0).toLong, bits(1).toLong)
      }
    }
  }

  def saveLongPairs(sc: SparkContext, fn: String, pairs: RDD[(Long, Long)]) = {
    val fs = FileSystem.get(sc.hadoopConfiguration)
    val output = fs.create(new Path(fn))
    val os = new BufferedOutputStream(output)
    pairs.collect.foreach {
      case (x1, x2) => os.write(s"$x1 $x2\n".getBytes("UTF-8"))
    }
    os.close()
  }
}
