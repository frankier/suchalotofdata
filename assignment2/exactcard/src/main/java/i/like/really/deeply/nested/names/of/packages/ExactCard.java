package i.like.really.deeply.nested.names.of.packages;
import java.io.IOException;

import org.apache.hadoop.conf.Configuration;
import org.apache.hadoop.fs.Path;
import org.apache.hadoop.io.IntWritable;
import org.apache.hadoop.io.LongWritable;
import org.apache.hadoop.io.NullWritable;
import org.apache.hadoop.io.Text;
import org.apache.hadoop.mapreduce.Job;
import org.apache.hadoop.mapreduce.Mapper;
import org.apache.hadoop.mapreduce.Reducer;
import org.apache.hadoop.mapreduce.lib.input.FileInputFormat;
import org.apache.hadoop.mapreduce.lib.output.FileOutputFormat;

class Unique {
	static class Project extends Mapper<Object, Text, Text, NullWritable> {
		public void map(Object key, Text value, Context context)
				throws IOException, InterruptedException {
			context.write(value, NullWritable.get());
		}
	}
	
	static class Combine extends Reducer<Text, NullWritable, Text, NullWritable> {
		public void reduce(Text key, Iterable<NullWritable> values, Context context)
				throws IOException, InterruptedException {
			context.write(key, NullWritable.get());
		}
	}

	static class ErasingReducer extends Reducer<Text, NullWritable, IntWritable, NullWritable> {
		private IntWritable one = new IntWritable(1);

		public void reduce(Text key, Iterable<NullWritable> values, Context context)
				throws IOException, InterruptedException {
			context.write(one, NullWritable.get());
		}
	}
	
	public static Job getJob(Configuration conf) throws Exception {
		Job job = Job.getInstance(conf, "unique");
		job.setJarByClass(Unique.class);
		job.setMapperClass(Project.class);
		job.setCombinerClass(Combine.class);
		job.setReducerClass(ErasingReducer.class);
		job.setMapOutputKeyClass(Text.class);
		job.setMapOutputValueClass(NullWritable.class);
		job.setOutputKeyClass(IntWritable.class);
		job.setOutputValueClass(NullWritable.class);
		return job;
	}
}

class Count {
	public static class Erase extends Mapper<LongWritable, Text, NullWritable, NullWritable> {
		public void map(LongWritable key, Text value, Context context)
				throws IOException, InterruptedException {
			context.write(NullWritable.get(), NullWritable.get());
		}
	}
	
	static class Sum extends Reducer<NullWritable, NullWritable, Text, LongWritable> {
		private LongWritable result = new LongWritable(32);

		public void reduce(NullWritable key, Iterable<NullWritable> values, Context context)
				throws IOException, InterruptedException {
			long count = 0;
			for (NullWritable blah : values) {
				count++;
			}
			result.set(count);
			context.write(new Text("count"), result);
		}
	}
	
	public static Job getJob(Configuration conf) throws Exception {
		Job job = Job.getInstance(conf, "count");
		job.setJarByClass(Count.class);
		job.setMapperClass(Erase.class);
		job.setReducerClass(Sum.class);
		job.setMapOutputKeyClass(NullWritable.class);
		job.setMapOutputValueClass(NullWritable.class);
		job.setOutputKeyClass(Text.class);
		job.setOutputValueClass(LongWritable.class);
		return job;
	}
}


public class ExactCard {
	public static void main(String[] args) throws Exception {
		Configuration conf = new Configuration();
		Path intermediate_path = new Path(args[1]);
		
		Job unique_job = Unique.getJob(conf);
		FileInputFormat.addInputPath(unique_job, new Path(args[0]));
		FileOutputFormat.setOutputPath(unique_job, intermediate_path);
		unique_job.waitForCompletion(true);
		boolean unique_succeeded = unique_job.waitForCompletion(true);
		
		System.exit(unique_succeeded ? 0 : 1);

		Job count_job = Count.getJob(conf);
		FileInputFormat.addInputPath(count_job, intermediate_path);
		FileOutputFormat.setOutputPath(count_job, new Path(args[2]));
		boolean count_succeeded = count_job.waitForCompletion(true);
		
		System.exit(count_succeeded ? 0 : 1);
	}
}