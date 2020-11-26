import matplotlib
import matplotlib.pyplot as plt
import numpy as np

# Data for plotting
# t = np.arange(0.0, 2.0, 0.01)
# s = 1 + np.sin(2 * np.pi * t)

xs = []
ys1 = []
ys2 = []
with open("data.txt") as fl:
  for line in fl.readlines():
    l = line.split('\t')
    # print(l)
    xs.append(float(l[0]))
    ys1.append(float(l[1]))
    ys2.append(float(l[2]))

print(len(xs), len(ys1), len(ys2))

fig, ax = plt.subplots()
ax.plot(xs, ys1,'.', ls='-', label='Empire')
ax.plot(xs, ys2,'.', ls='-', color="r",label='Talys')
# ax.scatter([1,3],[2,5])
# ax.plot(t, s)

ax.set(xlabel='Energy, MeV', ylabel='Intensity',
       title='Theoretical Talys/Empire')
ax.grid()

plt.legend(loc='upper right')
fig.savefig("theoretical.png")
plt.show()
