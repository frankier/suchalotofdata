#chart <- read.csv('chart.csv');
#chart$acc_secs <- chart$secs + chart$nanos / 1000000000;
#
#par(mar = c(5,5,2,5))
#plot(abs_err_sum ~ num_hyperplanes, log = "x", data=chart, xaxt='n');
#par(new = T);
#plot(acc_secs ~ num_hyperplanes, log = "x", data=chart, axes=F, xlab=NA, ylab=NA);
#axis(side = 4);
#mtext(side = 4, line = 3, 'acc_secs');
#legend("topleft",
#       legend=c(expression(-log[10](italic(p))), "N genes"),
#       lty=c(1,0), pch=c(NA, 16), col=c("red3", "black"));
#
#axis(1, at=c(0.5, 1, 2, 4, 8, 16, 32, 64), labels=c(0, 1, 2, 4, 8, 16, 32, 64));


library(ggplot2)
library(reshape2)

chart <- read.csv('chart.csv');
chart$acc_secs <- chart$secs + chart$nanos / 1000000000;
chart$num_hyperplanes[1] = 0.5

mm <- melt(subset(chart, select=c(num_hyperplanes, acc_secs, abs_err_sum)), id.var="num_hyperplanes")
ggplot(mm, aes(x = num_hyperplanes, y = value)) +
  geom_point(aes(color = variable)) +
  facet_grid(variable ~ ., scales = "free_y", labeller=labeller(variable = c(acc_secs="Time to complete (secs)", abs_err_sum="Sum of absolute errors (rads)"))) +
  theme_bw() +
  theme(legend.position = "none", axis.title = element_blank()) +
  scale_x_continuous(trans="log2", breaks = c(0.5, 1, 2, 4, 8, 16, 32, 64),
                labels = c(0, 1, 2, 4, 8, 16, 32, 64)) +
  labs(x="Number of hyperplanes", y="response", size=1)
dev.copy(pdf, 'hyperplanes.pdf')
dev.off()
