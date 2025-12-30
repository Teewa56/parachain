"""
Generate synthetic behavioral biometric data for training
"""

import argparse
import pandas as pd
import numpy as np
from pathlib import Path


def generate_legitimate_user(user_id: int, n_samples: int) -> pd.DataFrame:
    """
    Generate data for a legitimate user with consistent patterns
    
    Args:
        user_id: Unique user identifier
        n_samples: Number of samples to generate
        
    Returns:
        DataFrame with behavioral features
    """
    # Base characteristics (consistent per user)
    base_typing_speed = np.random.randint(40, 100)  # WPM
    base_key_hold = np.random.randint(80, 150)  # ms
    base_transition = np.random.randint(60, 120)  # ms
    base_error_rate = np.random.randint(1, 8)  # %
    preferred_hour = np.random.randint(8, 22)  # Hour of day
    
    # Generate samples with natural variation
    samples = []
    for _ in range(n_samples):
        sample = {
            'user_id': user_id,
            'typing_speed_wpm': int(np.clip(
                np.random.normal(base_typing_speed, 5), 
                20, 150
            )),
            'avg_key_hold_time_ms': int(np.clip(
                np.random.normal(base_key_hold, 10), 
                40, 300
            )),
            'avg_transition_time_ms': int(np.clip(
                np.random.normal(base_transition, 8), 
                30, 250
            )),
            'error_rate_percent': int(np.clip(
                np.random.normal(base_error_rate, 1), 
                0, 30
            )),
            'activity_hour_preference': int(np.clip(
                np.random.normal(preferred_hour, 2), 
                0, 23
            )),
            'is_legitimate': 1,
        }
        samples.append(sample)
    
    return pd.DataFrame(samples)


def generate_impostor(target_user_id: int, n_samples: int) -> pd.DataFrame:
    """
    Generate impostor data (different patterns from legitimate user)
    
    Args:
        target_user_id: User being impersonated
        n_samples: Number of samples to generate
        
    Returns:
        DataFrame with impostor features
    """
    # Impostor has different characteristics
    impostor_speed = np.random.randint(30, 90)
    impostor_hold = np.random.randint(70, 180)
    impostor_transition = np.random.randint(50, 140)
    impostor_error = np.random.randint(3, 15)
    impostor_hour = np.random.randint(6, 23)
    
    samples = []
    for _ in range(n_samples):
        sample = {
            'user_id': target_user_id,
            'typing_speed_wpm': int(np.clip(
                np.random.normal(impostor_speed, 8), 
                20, 150
            )),
            'avg_key_hold_time_ms': int(np.clip(
                np.random.normal(impostor_hold, 15), 
                40, 300
            )),
            'avg_transition_time_ms': int(np.clip(
                np.random.normal(impostor_transition, 12), 
                30, 250
            )),
            'error_rate_percent': int(np.clip(
                np.random.normal(impostor_error, 2), 
                0, 30
            )),
            'activity_hour_preference': int(np.clip(
                np.random.normal(impostor_hour, 3), 
                0, 23
            )),
            'is_legitimate': 0,
        }
        samples.append(sample)
    
    return pd.DataFrame(samples)


def generate_dataset(
    n_users: int,
    samples_per_user: int,
    impostor_ratio: float = 0.2,
) -> pd.DataFrame:
    """
    Generate complete synthetic dataset
    
    Args:
        n_users: Number of unique users
        samples_per_user: Samples per legitimate user
        impostor_ratio: Ratio of impostor samples (0.0 to 1.0)
        
    Returns:
        Complete dataset DataFrame
    """
    all_data = []
    
    print(f"Generating data for {n_users} users...")
    
    for user_id in range(n_users):
        if (user_id + 1) % 100 == 0:
            print(f"  Generated {user_id + 1}/{n_users} users")
        
        # Generate legitimate samples
        legitimate_data = generate_legitimate_user(user_id, samples_per_user)
        all_data.append(legitimate_data)
        
        # Generate impostor samples
        n_impostor_samples = int(samples_per_user * impostor_ratio)
        if n_impostor_samples > 0:
            impostor_data = generate_impostor(user_id, n_impostor_samples)
            all_data.append(impostor_data)
    
    # Combine all data
    df = pd.concat(all_data, ignore_index=True)
    
    # Shuffle
    df = df.sample(frac=1, random_state=42).reset_index(drop=True)
    
    return df


def main():
    parser = argparse.ArgumentParser(
        description='Generate synthetic behavioral biometric data'
    )
    parser.add_argument(
        '--samples',
        type=int,
        default=10000,
        help='Total number of samples'
    )
    parser.add_argument(
        '--output',
        type=str,
        default='data/raw/synthetic_data.csv',
        help='Output CSV path'
    )
    parser.add_argument(
        '--legitimate-ratio',
        type=float,
        default=0.8,
        help='Ratio of legitimate samples (0.0-1.0)'
    )
    
    args = parser.parse_args()
    
    # Calculate users and samples
    impostor_ratio = 1.0 - args.legitimate_ratio
    samples_per_user = 100  # Fixed
    n_users = args.samples // int(samples_per_user * (1 + impostor_ratio))
    
    print(f"Configuration:")
    print(f"  Total samples: {args.samples}")
    print(f"  Number of users: {n_users}")
    print(f"  Samples per user: {samples_per_user}")
    print(f"  Legitimate ratio: {args.legitimate_ratio:.1%}")
    print(f"  Impostor ratio: {impostor_ratio:.1%}")
    print()
    
    # Generate dataset
    df = generate_dataset(
        n_users=n_users,
        samples_per_user=samples_per_user,
        impostor_ratio=impostor_ratio,
    )
    
    # Create output directory
    output_path = Path(args.output)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    
    # Save to CSV
    df.to_csv(output_path, index=False)
    
    print(f"\nDataset statistics:")
    print(f"  Total samples: {len(df)}")
    print(f"  Legitimate: {df['is_legitimate'].sum()} ({df['is_legitimate'].mean():.1%})")
    print(f"  Impostor: {(~df['is_legitimate'].astype(bool)).sum()} ({1 - df['is_legitimate'].mean():.1%})")
    print(f"\nSaved to: {output_path}")


if __name__ == '__main__':
    main()