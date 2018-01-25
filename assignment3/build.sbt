// give the user a nice default project!
lazy val root = (project in file(".")).
  settings(
    inThisBuild(List(
      organization := "frrobert.jyu.fi",
      scalaVersion := "2.11.8"
    )),
    name := "emailgraphmeasures",
    mainClass := Some("frrobert.jyu.fi.emailgraphmeasures."),
    version := "0.0.1",
    javacOptions ++= Seq("-source", "1.8", "-target", "1.8"),
    parallelExecution in Test := false,
    fork := true,
    javaOptions ++= Seq("-Xms512M", "-Xmx2048M", "-XX:MaxPermSize=2048M", "-XX:+CMSClassUnloadingEnabled"),
    libraryDependencies ++= Seq(
      "org.apache.spark" %% "spark-core" % "2.2.1",
      "org.apache.spark" %% "spark-graphx" % "2.2.1",
      "ml.sparkling" %% "sparkling-graph-operators" % "0.0.8-SNAPSHOT",
      "com.google.guava" % "guava" % "23.6-jre",
      "org.apache.hadoop" % "hadoop-client" % "2.7.2"
    ),
    scalacOptions ++= Seq("-deprecation", "-unchecked"),
    pomIncludeRepository := { x => false },
    resolvers ++= Seq(
      "Spark Package Main Repo" at "https://dl.bintray.com/spark-packages/maven",
      "Sonatype OSS Snapshots" at "https://oss.sonatype.org/content/repositories/snapshots"
    ),
    // publish settings
    publishTo := {
      val nexus = "https://oss.sonatype.org/"
      if (isSnapshot.value)
        Some("snapshots" at nexus + "content/repositories/snapshots")
      else
        Some("releases"  at nexus + "service/local/staging/deploy/maven2")
    }
  )
