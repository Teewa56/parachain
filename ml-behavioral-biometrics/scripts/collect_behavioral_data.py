#!/usr/bin/env python3
"""
Behavioral data collection tool
IMPORTANT: Only collects timing features, NOT keystrokes or content
"""

import json
import time
from datetime import datetime
from pathlib import Path
from pynput import keyboard

class BehavioralDataCollector:
    def __init__(self, user_id: str, session_id: str):
        self.user_id = user_id
        self.session_id = session_id
        self.key_press_times = {}
        self.key_events = []
        self.session_start = time.time()
        
    def on_press(self, key):
        """Record key press time"""
        try:
            key_code = key.char if hasattr(key, 'char') else str(key)
            self.key_press_times[key_code] = time.time()
        except Exception:
            pass
    
    def on_release(self, key):
        """Record key release and calculate features"""
        try:
            key_code = key.char if hasattr(key, 'char') else str(key)
            if key_code in self.key_press_times:
                release_time = time.time()
                press_time = self.key_press_times[key_code]
                hold_time = (release_time - press_time) * 1000  # ms
                
                self.key_events.append({
                    'hold_time_ms': hold_time,
                    'timestamp': release_time,
                })
                
                del self.key_press_times[key_code]
        except Exception:
            pass
        
        # Stop on ESC
        if key == keyboard.Key.esc:
            return False
    
    def calculate_features(self):
        """Calculate behavioral features from collected events"""
        if len(self.key_events) < 10:
            return None
        
        # Calculate typing speed (approximate)
        session_duration_minutes = (time.time() - self.session_start) / 60
        estimated_words = len(self.key_events) / 5  # Avg 5 chars per word
        typing_speed = int(estimated_words / session_duration_minutes) if session_duration_minutes > 0 else 0
        
        # Calculate hold times
        hold_times = [e['hold_time_ms'] for e in self.key_events]
        avg_hold_time = int(sum(hold_times) / len(hold_times))
        
        # Calculate transition times
        transition_times = []
        for i in range(1, len(self.key_events)):
            transition = (self.key_events[i]['timestamp'] - 
                         self.key_events[i-1]['timestamp']) * 1000
            if 0 < transition < 1000:  # Filter outliers
                transition_times.append(transition)
        
        avg_transition = int(sum(transition_times) / len(transition_times)) if transition_times else 0
        
        # Error rate (placeholder - would need actual error detection)
        error_rate = 3  # Default
        
        # Activity hour
        activity_hour = datetime.now().hour
        
        return {
            'user_id': self.user_id,
            'session_id': self.session_id,
            'typing_speed_wpm': typing_speed,
            'avg_key_hold_time_ms': avg_hold_time,
            'avg_transition_time_ms': avg_transition,
            'error_rate_percent': error_rate,
            'activity_hour_preference': activity_hour,
            'timestamp': int(time.time()),
        }
    
    def save_features(self, output_path: str):
        """Save features to JSON"""
        features = self.calculate_features()
        if features:
            Path(output_path).parent.mkdir(parents=True, exist_ok=True)
            with open(output_path, 'a') as f:
                f.write(json.dumps(features) + '\n')
            print(f" Saved features: {features}")

def main():
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument('--user-id', required=True, help='User ID')
    parser.add_argument('--output', default='data/raw/collected_features.jsonl')
    args = parser.parse_args()
    
    session_id = f"session_{int(time.time())}"
    collector = BehavioralDataCollector(args.user_id, session_id)
    
    print("=" * 60)
    print("BEHAVIORAL DATA COLLECTOR")
    print("=" * 60)
    print(f"User ID: {args.user_id}")
    print(f"Session: {session_id}")
    print("\n⌨  Start typing... (Press ESC to stop)")
    print("\nNOTE: Only timing features are collected, NOT keystrokes!")
    print("=" * 60)
    
    # Start listener
    with keyboard.Listener(
        on_press=collector.on_press,
        on_release=collector.on_release
    ) as listener:
        listener.join()
    
    # Save features
    collector.save_features(args.output)
    print(f"\n Collection complete. Data saved to {args.output}")

if __name__ == '__main__':
    main()