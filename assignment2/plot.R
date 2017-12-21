library(ggplot2)
library(reshape2)
library(plyr)

# Preprocess into a dataframe
s <- paste(readLines("task1.dat"), sep='\n')
df <- read.delim(
  textConnection(s),
  header=FALSE,
  sep=" ",
  strip.white=TRUE)
df <- unstack(df, V2 ~ V1)

# Add element count
df$Elems <- seq(from=0, by=1, length.out=dim(df)[1])

# Change units
df$Mem <- df$Mem / 1000000000

# Make df suitable fo faceting
df <- melt(subset(df, select=c(Mem, Time, Elems)), id.var="Elems")

# Make plot
pdf('setperf.pdf')
ggplot(df, aes(x = Elems, y = value)) +
  geom_point(aes(color = variable)) +
  facet_grid(variable ~ ., scales = "free_y", labeller=labeller(variable = c(Mem="Memory usage (GiB)", Time="Time to add a million more (seconds)"))) +
  theme_bw() +
  theme(legend.position = "none", axis.title.y = element_blank()) +
  labs(x="Millions of elements already in set", y="response", size=1)
