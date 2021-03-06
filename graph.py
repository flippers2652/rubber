import numpy as np
from matplotlib import pyplot as plt

import time
print("Opening")
file = open("file.csv")
print("Reading")
con = file.readlines()
x, y = [], []
c=0
print("Parsing")
for i in con:
    s = i.split(",")
    x.append(float(s[0]))
    y.append(float(s[1]))
print("Parsed")
corr_matrix = np.corrcoef(x, y)
corr = corr_matrix
R_sq = corr
x=x[:1000]
y=y[:1000]
 
print(R_sq)
      
fig, ax = plt.subplots()
y_max = 10 ** int(np.log10(max(y)))
By = int(max(y)) // y_max + 1
y_max = By * y_max
x_max = 10 ** int(np.log10(max(max(x), -min(x))))
Bx = int(max(x)) // x_max + 1
x_max = Bx * x_max
ax.set_xticks(np.linspace(-x_max, x_max, 1 + Bx * 4), minor=True)

ax.set(xlim=(-x_max, x_max), xticks=np.linspace(-x_max, x_max, 2 * Bx + 1),
       ylim=(0, y_max), yticks=np.linspace(0, y_max, 2 * By + 1))

ax.set_yticks(np.linspace(0, y_max, 1 + 4 * By), minor=True)
ax.set_axisbelow(True)
ax.grid(True, which='both')
ax.grid(True, which='major', color=(0., 0., 0.))
ax.grid(True, which='minor', color=(0., 0., 0.), alpha=0.2)
ax.scatter(x, y, marker="x")
ax.set_ylabel('Lengths')
ax.set_xlabel('n')
plt.show()
#plt.savefig("file.jpg")
