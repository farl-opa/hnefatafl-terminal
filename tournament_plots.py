import pandas as pd
import matplotlib.pyplot as plt

# Create a DataFrame manually (or read from CSV if saved)
data = {
    'Result': ['Attacker wins', 'Defender wins', 'Draws'],
    'Algorithm 1': [93, 218, 27],
    'Algorithm 2': [224, 299, 27],
    'Algorithm 3.1': [33, 136, 0],
    'Algorithm 3.2': [26, 144, 0]
}

df = pd.DataFrame(data)
df.set_index('Result', inplace=True)

# Plotting
df.T.plot(kind='bar', figsize=(10, 6))
plt.title('Outcomes by Algorithm')
plt.ylabel('Number of Games')
plt.xlabel('Algorithm')
plt.xticks(rotation=0)
plt.legend(title='Result')
plt.tight_layout()
plt.show()
