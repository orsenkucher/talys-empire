import matplotlib
import matplotlib.pyplot as plt
import numpy as np

# Data for plotting
# t = np.arange(0.0, 2.0, 0.01)
# s = 1 + np.sin(2 * np.pi * t)

xs = []
ys = []
with open("emp.txt") as fl:
  for line in fl.readlines():
    for i in range(0,10):
      line=line.replace('  ', ' ')
    l = line.strip().replace('  ', ' ').split(' ')
    print(l)
    if (float(l[2])>180000.0):
      continue
    xs.append(float(l[0])/1000)
    ys.append(float(l[2]))


xs2 = []
ys2 = []
with open("tal.txt") as fl:
  for line in fl.readlines():
    for i in range(0,10):
      line=line.replace('  ', ' ')
    l = line.strip().replace('  ', ' ').split(' ')
    print(l)
    # if (float(l[2])>180000.0):
    #   continue
    xs2.append(float(l[0])/1000)
    ys2.append(float(l[2]))

xs0 = []
ys10 = []
ys20 = []
with open("data.txt") as fl:
  for line in fl.readlines():
    l = line.split('\t')
    # print(l)
    xs0.append(float(l[0]))
    ys10.append(float(l[1]))
    ys20.append(float(l[2]))

print(len(xs), len(ys))

fig, ax = plt.subplots()
ax.plot(xs, ys,'.', ls='-', label='Exp Empire')
ax.plot(xs2, ys2,'.', ls='-', label='Exp Talys')

ax.plot(xs0, ys10,'.', ls='-', label='Theor Empire')
ax.plot(xs0, ys20,'.', ls='-', color="r",label='Theor Talys')

# ax.plot(xs, ys2,'.', ls='-', color="r",label='Talys')
# ax.scatter([1,3],[2,5])
# ax.plot(t, s)

ax.set(xlabel='Energy, MeV', ylabel='Intensity, lo^4 normalized',
       title='Experimental normed Talys/Empire')
ax.grid()

plt.legend(loc='upper right')
fig.savefig("talys_plus.png")
plt.show()
