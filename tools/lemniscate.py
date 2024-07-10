import numpy as np
import matplotlib.pyplot as plt

# Parameters for the lemniscate of Gerono
t = np.linspace(0, 2 * np.pi, 1000)
x = 100 * np.cos(t) + 300
y = 100*  np.sin(t) * np.cos(t) + 200

# Plotting the lemniscate of Gerono
plt.figure(figsize=(6,6))
plt.plot(x, y)
plt.axhline(0, color='black', linewidth=0.5)
plt.axvline(0, color='black', linewidth=0.5)
plt.grid(color='gray', linestyle='--', linewidth=0.5)
plt.title('Lemniscate of Gerono')
plt.xlabel('x')
plt.ylabel('y')
plt.gca().set_aspect('equal', adjustable='box')
plt.show()
