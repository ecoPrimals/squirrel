#!/usr/bin/env python3
"""
Check Phase 4 async_trait migration progress
"""
import subprocess
import os
from datetime import datetime
from pathlib import Path

BASELINE = 317  # Initial count
TARGET = 10     # Target remaining (legitimate uses)

def count_async_trait():
    """Count current async_trait usage"""
    # Get the squirrel root directory
    script_dir = Path(__file__).parent
    squirrel_root = script_dir.parent
    
    try:
        result = subprocess.run(
            ['grep', '-r', '#[async_trait]', 'crates', '--include=*.rs'],
            capture_output=True,
            text=True,
            cwd=str(squirrel_root)
        )
        lines = [line for line in result.stdout.split('\n') if line.strip() and '#[async_trait]' in line]
        return len(lines)
    except Exception as e:
        print(f"Error counting: {e}")
        return None

def main():
    current = count_async_trait()
    
    if current is None:
        print("Error: Could not count async_trait usage")
        return
    
    migrated = BASELINE - current
    remaining = current - TARGET
    progress_pct = (migrated / (BASELINE - TARGET)) * 100 if BASELINE > TARGET else 0
    
    print("=" * 60)
    print("PHASE 4: ASYNC_TRAIT MIGRATION PROGRESS")
    print("=" * 60)
    print(f"Date:              {datetime.now().strftime('%Y-%m-%d %H:%M')}")
    print()
    print(f"Baseline:          {BASELINE} instances")
    print(f"Target:            {TARGET} instances")
    print(f"Current:           {current} instances")
    print()
    print(f"Migrated:          {migrated} instances")
    print(f"Remaining:         {remaining} instances (to target)")
    print(f"Progress:          {progress_pct:.1f}%")
    print()
    
    # Status
    if current <= TARGET:
        print("✅ STATUS: COMPLETE!")
        print("   Migration target achieved!")
    elif progress_pct >= 80:
        print("🟢 STATUS: NEARLY COMPLETE")
        print(f"   Only {remaining} instances remaining")
    elif progress_pct >= 50:
        print("🟡 STATUS: ON TRACK")
        print(f"   Halfway there! Keep going!")
    elif progress_pct >= 20:
        print("🟡 STATUS: IN PROGRESS")
        print(f"   Good start! Continue with hot paths")
    elif progress_pct > 0:
        print("🟡 STATUS: STARTED")
        print(f"   Beginning migration, {migrated} done")
    else:
        print("🔴 STATUS: NOT STARTED")
        print("   Ready to begin! Start with Core MCP")
    
    print()
    
    # Timeline estimate
    if current > TARGET:
        weeks_remaining = remaining / 50  # ~50 per week estimate
        print(f"Estimated remaining: {weeks_remaining:.1f} weeks")
        print(f"  (at ~50 migrations/week pace)")
    
    print("=" * 60)

if __name__ == "__main__":
    main()
