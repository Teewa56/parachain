import json
import pandas as pd

features_list = []
with open('data/raw/collected_features.jsonl') as f:
    for line in f:
        features_list.append(json.loads(line))

df = pd.DataFrame(features_list)

# Add labels (1 = legitimate, 0 = impostor)
# You'd need to label impostor attempts manually or simulate them
df['is_legitimate'] = 1  # Default to legitimate

df.to_csv('data/raw/real_data.csv', index=False)
print(f"Converted {len(df)} samples to CSV")