import json
from plumbum.cmd import java, cargo

CARDS = [
    3753742,
    3755459,
    3753884,
    3755369,
    3752845,
    3753219,
    3754504,
    3755127,
    3753823,
    3754738,
]

SETUPS = [
    ('hll', 3),
    ('hll', 4),
    ('hll', 5),
    ('hll', 6),
    ('hll', 7),
    ('hll', 8),
    ('hll', 9),
    ('hll', 10),
    ('kmv', 8),
    ('kmv', 16),
    ('kmv', 32),
    ('kmv', 64),
    ('kmv', 128),
    ('kmv', 256),
    ('kmv', 512),
    ('kmv', 1024)]

mse = {'hll': [], 'kmv': []}

for alg, p in SETUPS:
    sum = 0
    for seed, card in enumerate(CARDS):
        seed = seed + 1
        generator = java['-jar', 'generator.jar', '10000000']
        cardapprox = cargo['run', '--release', '--bin', 'task2',
                           alg, str(p), seed]

        pipeline = generator | cardapprox
        est = int(pipeline().strip())
        sum += (est - card) ** 2
    mse[alg].append((sum / 10) ** 0.5)

print(json.dumps(mse))
