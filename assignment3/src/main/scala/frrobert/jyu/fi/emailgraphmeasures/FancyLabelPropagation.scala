/*
 * Modifications by Frankie Robertson 2018. The notice below applies to the
 * original code.
 *
 * Licensed to the Apache Software Foundation (ASF) under one or more
 * contributor license agreements.  See the NOTICE file distributed with this
 * work for additional information regarding copyright ownership.  The ASF
 * licenses this file to You under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *    http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
 * WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.  See the
 * License for the specific language governing permissions and limitations
 * under the License.
 */

package frrobert.jyu.fi.emailgraphmeasures

import scala.reflect.ClassTag

import org.apache.spark.graphx._
import com.google.common.hash.Hashing.murmur3_32

/** Label Propagation algorithm.
 *
 * Modified from version in GraphX so that it:
 *  - Is able to start from a prelabelled graph.
 *  - Supports semi-ordered execution. Rather than running in a totally synchronised
 *    fashion, parition the set of nodes into approximately equal parts and
 *    propagate into nodes in each partition in succession.
 *  - Can run in a simulated annealing inspired fashion where an already
 *    labelled vertex will only be relabelled with a decaying likehood.
 */
object FancyLabelPropagation {
  type Message = Map[VertexId, Long]

  case class GlobalState(
    iteration: Int,
    partition: Int
  )

  type VertexLabel = Option[Long]

  def sustainDecayProbCurve(transition: Int, end: Int)(iteration: Int): Double = {
    //println(s"transition: $transition end: $end iteration: $iteration")
    if (iteration <= transition) {
      1.0
    } else if (transition < iteration && iteration < end) {
      val intermediate = 1.0 - ((iteration.toDouble - transition.toDouble) / (end.toDouble - transition.toDouble))
      //println(s"intermediate: $intermediate")
      intermediate
    } else {
      0.0
    }
  }

  def alwaysAccept(_iteration: Int): Double = {
    1.0
  }

  def deoptionize[ED](graph: Graph[VertexLabel, ED]): Graph[Long, ED] = {
    graph mapVertices {
      case (vid, vlbl) => vlbl.getOrElse(-1)
    }
  }

  /**
   * Run static Label Propagation for detecting communities in networks.
   *
   * Each node in the network is initially assigned to its own community. At every superstep, nodes
   * send their community affiliation to all neighbors and update their state to the mode community
   * affiliation of incoming messages.
   *
   * LPA is a standard community detection algorithm for graphs. It is very inexpensive
   * computationally, although (1) convergence is not guaranteed and (2) one can end up with
   * trivial solutions (all nodes are identified into a single community).
   *
   * @tparam ED the edge attribute type (not used in the computation)
   *
   * @param graph the graph for which to compute the community affiliation
   * @param partitions how many graph partitions to create
   * @param maxSteps the number of supersteps of LPA to be performed. Because this is a static
   * @param seed the seed to use for creating the graph partitions
   * @param reshuffle whether to reshuffle the paritioning each iteration
   * implementation, the algorithm will run for exactly this many supersteps.
   *
   * @return a graph with vertex attributes containing the label of community affiliation
   */
  def run[VD, ED: ClassTag](
      graph: Graph[VD, ED], partitions: Int, maxSteps: Int,
      seed: Int = 0, reshuffle: Boolean = false,
      acceptanceProb: Int => Double = alwaysAccept): Graph[VertexLabel, ED] = {
    val lpaGraph = graph.mapVertices {
      case (vid, _) => Some(vid): Option[Long]
    }

    propagate(lpaGraph, partitions, maxSteps, seed, reshuffle, false, acceptanceProb)
  }

  /**
   * Run LPA seeded with a few authorities, assumed to be serving different
   * communities. Most parameters as for `run'.
   *
   * @param labeledVids a set containing the initial authorities
   */
  def runAuthorities[VD, ED: ClassTag](
      graph: Graph[VD, ED], labeledVids: Set[VertexId], partitions: Int, maxSteps: Int,
      seed: Int = 0, reshuffle: Boolean = false,
      acceptanceProb: Int => Double = alwaysAccept): Graph[VertexLabel, ED] = {
    val lpaGraph = graph.mapVertices {
      case (vid, _) =>
        if (labeledVids contains vid) {
          Some(vid)
        } else {
          None
        }
    };

    val expandedGraph = propagate(lpaGraph, partitions, maxSteps, seed, reshuffle, false, acceptanceProb)
    propagate(expandedGraph, partitions, maxSteps, seed, reshuffle, true, acceptanceProb)
  }

  /**
   * Internal method serving as workhorse for the others. Most parameters as
   * for `run'.
   *
   * @param sweep when true, only propagate to unlabelled nodes
   */
  def propagate[ED: ClassTag](
      graph: Graph[VertexLabel, ED], partitions: Int, maxSteps: Int,
      seed: Int = 0, reshuffle: Boolean = false, sweep: Boolean = false,
      acceptanceProb: Int => Double = alwaysAccept): Graph[VertexLabel, ED] = {

    require(maxSteps > 0, s"Maximum of steps must be greater than 0, but got ${maxSteps}")

    val hashFunc = murmur3_32(seed)

    def sendMessage(state: GlobalState)(e: EdgeTriplet[VertexLabel, ED]): Iterator[(VertexId, Message)] = {
      def getMsg(srcAttr: VertexLabel, dstAttr: VertexLabel, dstId: VertexId) = {
        srcAttr.filter(_ => (!sweep && acceptanceProb(state.iteration) > 0) || dstAttr.nonEmpty).map({
          srcLabel => Iterator((dstId, Map(srcLabel -> 1L)))
        }).getOrElse(Iterator.empty)
      }
      /* TODO: Make use of acceptanceProb
      state.iteration
      var hasher = hashFunc.newHasher()
      hasher.putLong(vid)
      */
      val forwardMsg = getMsg(e.srcAttr, e.dstAttr, e.dstId);
      val backwardsMsg = getMsg(e.dstAttr, e.srcAttr, e.srcId);
      forwardMsg ++ backwardsMsg
    }
    def mergeMessage(count1: Message, count2: Message)
      : Message = {
      (count1.keySet ++ count2.keySet).map { i =>
        val count1Val = count1.getOrElse(i, 0L)
        val count2Val = count2.getOrElse(i, 0L)
        i -> (count1Val + count2Val)
      }(collection.breakOut) // more efficient alternative to [[collection.Traversable.toMap]]
    }
    def vertexProgram(state: GlobalState)(vid: VertexId, attr: VertexLabel, message: Message): VertexLabel = {
      var hasher = hashFunc.newHasher()
      hasher.putLong(vid)
      if (reshuffle) {
        hasher.putInt(state.iteration)
      }
      val h = hasher.hash().asInt() & 0x00000000ffffffffL
      //println(h);
      val lower = (state.partition.toLong << 32L) / partitions.toLong
      //println(lower);
      val upper = ((state.partition + 1).toLong << 32L) / partitions.toLong
      //println(upper);
      val active = lower <= h && h < upper

      def receive(): VertexLabel = {
        Some(message.maxBy {
          case (label, count) => {
            count
          }
        }._1)
      }

      if (!message.isEmpty && active) {
        val accept = acceptanceProb(state.iteration)
        if (accept >= 1.0 || attr.isEmpty) {
          receive()
        } else {
          var hasher = hashFunc.newHasher()
          hasher.putLong(vid)
          hasher.putInt(state.iteration)
          // Uncorrelate
          hasher.putInt(0x11bf6e28)
          val h = hasher.hash().asInt() & 0x00000000ffffffffL
          if (h < accept * 0x0000000100000000L) {
            receive()
          } else {
            attr
          }
        }
      } else {
          attr
      }
    }
    def nextState(state: GlobalState): GlobalState = {
      val nextPartition = state.partition + 1
      if (nextPartition >= partitions)
        GlobalState(
          state.iteration + 1,
          0)
      else
        GlobalState(
          state.iteration,
          nextPartition)

    }
    val initialMessage = Map[VertexId, Long]()
    val initialState = GlobalState(0, 0)
    FancyPregel(graph, initialMessage, initialState, maxIterations = maxSteps * partitions)(
      vprog = vertexProgram,
      sendMsg = sendMessage,
      mergeMsg = mergeMessage,
      nextState = nextState)
  }
}
