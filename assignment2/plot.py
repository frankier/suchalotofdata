import matplotlib
import matplotlib.pyplot as plt
import pandas as pd
import seaborn as sns
import json

SPACE = [8, 16, 32, 64, 128, 256, 512, 1024]
dat = json.load(open('rmse.dat'))
df = pd.DataFrame()
df['val'] = [x / 10000000 for x in (dat['hll'] + dat['kmv'])]
df['space'] = SPACE + SPACE
df['Algorithm'] = ['HyperLogLog'] * len(dat['hll']) + \
        ['K min values'] * len(dat['kmv'])
print(df)

sns.set()
sns_plot = sns.lmplot(
   'space',
   'val',
   data=df,
   fit_reg=False,
   hue="Algorithm")

ax = plt.gca()
ax.set_xscale('log')
ax.set_xlabel('Space usage (64-bit words)')
ax.set_ylabel('Root mean square error')
ax.set_xticks(SPACE)
ax.get_xaxis().set_major_formatter(matplotlib.ticker.ScalarFormatter())

plt.savefig('rmse.pdf')
