#!/usr/bin/env python3
"""
AMD SMU PM Table Temperature Analyzer

This script analyzes temperature values in the PM table to help identify
correct offsets for Tctl, SoC temp, and per-core temperatures.

Usage:
    sudo python3 analyze_temps.py [options]

Options:
    --tctl          Search for Tctl/junction temperature (40-95°C)
    --soc           Search for SoC temperature (30-70°C)
    --cores         Search for per-core temperature arrays
    --all           Run all temperature analyses
    --num-cores N   Number of cores to search for (default: 16)
    --watch         Continuously monitor temperatures
"""

import struct
import argparse
import sys
import time
from pathlib import Path

SYSFS_PATH = "/sys/kernel/ryzen_smu_drv"
PM_TABLE_PATH = f"{SYSFS_PATH}/pm_table"
PM_VERSION_PATH = f"{SYSFS_PATH}/pm_table_version"
CODENAME_PATH = f"{SYSFS_PATH}/codename"

CODENAME_MAP = {
    0: "Unsupported",
    1: "Colfax",
    2: "Renoir",
    3: "Picasso",
    4: "Matisse",
    5: "Threadripper",
    6: "Castle Peak",
    7: "Raven",
    8: "Raven 2",
    9: "Summit Ridge",
    10: "Pinnacle Ridge",
    11: "Rembrandt",
    12: "Vermeer",
    13: "Van Gogh",
    14: "Cezanne",
    15: "Milan",
    16: "Dali",
    17: "Lucienne",
    18: "Naples",
    19: "Chagall",
    20: "Raphael",
    21: "Phoenix",
    22: "Hawk Point",
    23: "Granite Ridge",
    24: "Strix Point",
    25: "Storm Peak",
}


def read_pm_table():
    """Read PM table binary data"""
    try:
        with open(PM_TABLE_PATH, 'rb') as f:
            return f.read()
    except PermissionError:
        print("Error: Permission denied. Run with sudo.", file=sys.stderr)
        sys.exit(1)
    except FileNotFoundError:
        print(f"Error: {PM_TABLE_PATH} not found. Is ryzen_smu module loaded?", file=sys.stderr)
        sys.exit(1)


def read_pm_version():
    """Read PM table version as little-endian u32"""
    try:
        with open(PM_VERSION_PATH, 'rb') as f:
            data = f.read(4)
            return struct.unpack('<I', data)[0]
    except:
        return 0


def read_codename():
    """Read processor codename"""
    try:
        with open(CODENAME_PATH, 'r') as f:
            return int(f.read().strip())
    except:
        return 0


def read_f32(data, offset):
    """Read little-endian float at offset"""
    if offset + 4 > len(data):
        return None
    return struct.unpack('<f', data[offset:offset+4])[0]


def print_header():
    """Print system info header"""
    version = read_pm_version()
    codename_id = read_codename()
    codename = CODENAME_MAP.get(codename_id, f"Unknown ({codename_id})")

    print("=" * 60)
    print("AMD SMU PM Table Temperature Analyzer")
    print("=" * 60)
    print(f"Codename:         {codename} (ID: {codename_id})")
    print(f"PM Table Version: 0x{version:08X}")
    print("=" * 60)
    print()


def search_tctl(data):
    """Search for Tctl/junction temperature (typically 40-95°C under load)"""
    print("=== Tctl/Junction Temperature Candidates (40-95°C) ===\n")

    candidates = []
    for i in range(0, len(data) - 4, 4):
        value = read_f32(data, i)
        if value and 40 < value < 95:
            candidates.append((i, value))

    # Sort by value (highest temps are likely Tctl)
    candidates.sort(key=lambda x: x[1], reverse=True)

    print("Top candidates (sorted by temperature, highest first):")
    for offset, value in candidates[:20]:
        print(f"  0x{offset:04X}: {value:6.2f}°C")

    return candidates


def search_soc_temp(data):
    """Search for SoC temperature (typically 30-70°C)"""
    print("=== SoC Temperature Candidates (30-70°C) ===\n")

    candidates = []
    for i in range(0, len(data) - 4, 4):
        value = read_f32(data, i)
        if value and 30 < value < 70:
            candidates.append((i, value))

    print(f"Found {len(candidates)} candidates in range 30-70°C:")
    for offset, value in candidates[:30]:
        print(f"  0x{offset:04X}: {value:6.2f}°C")

    return candidates


def search_core_temps(data, num_cores=16):
    """Search for per-core temperature arrays"""
    print(f"=== Per-Core Temperature Array Candidates ({num_cores} cores) ===\n")

    candidates = []

    for base in range(0, len(data) - num_cores * 4, 4):
        values = [read_f32(data, base + i * 4) for i in range(num_cores)]

        # Check if all values are in reasonable temperature range
        if all(v is not None and 25 < v < 100 for v in values):
            avg = sum(values) / len(values)
            spread = max(values) - min(values)

            # Good candidates have reasonable average and small spread
            if 35 < avg < 85 and spread < 30:
                candidates.append((base, values, avg, spread))

    # Sort by average temperature (descending)
    candidates.sort(key=lambda x: x[2], reverse=True)

    print(f"Found {len(candidates)} candidate arrays:")
    for base, values, avg, spread in candidates[:5]:
        print(f"\n  0x{base:04X}: avg={avg:.1f}°C, spread={spread:.1f}°C")
        for i, v in enumerate(values):
            ccd = i // 8
            core_in_ccd = i % 8
            print(f"    CCD{ccd} Core {core_in_ccd}: {v:5.1f}°C")

    return candidates


def compare_with_sensors(data):
    """Compare PM table temps with lm_sensors output"""
    print("=== Comparison with System Sensors ===\n")

    try:
        import subprocess
        result = subprocess.run(['sensors'], capture_output=True, text=True)
        if result.returncode == 0:
            print("lm_sensors output:")
            for line in result.stdout.split('\n'):
                if 'temp' in line.lower() or 'tctl' in line.lower() or 'tdie' in line.lower():
                    print(f"  {line.strip()}")
            print()
    except FileNotFoundError:
        print("  lm_sensors not installed or 'sensors' command not found\n")

    # Also try reading from hwmon
    print("hwmon k10temp readings:")
    try:
        hwmon_path = Path("/sys/class/hwmon")
        for hwmon in hwmon_path.iterdir():
            name_file = hwmon / "name"
            if name_file.exists():
                name = name_file.read_text().strip()
                if name == "k10temp":
                    for temp_file in sorted(hwmon.glob("temp*_input")):
                        label_file = temp_file.parent / temp_file.name.replace("_input", "_label")
                        label = label_file.read_text().strip() if label_file.exists() else temp_file.name
                        temp = int(temp_file.read_text().strip()) / 1000
                        print(f"  {label}: {temp:.1f}°C")
    except Exception as e:
        print(f"  Error reading hwmon: {e}")


def watch_temps(data_getter, offsets, interval=1.0):
    """Continuously monitor temperatures at specific offsets"""
    print("=== Temperature Monitor (Ctrl+C to stop) ===\n")
    print("Monitoring offsets:", [f"0x{o:04X}" for o in offsets])
    print()

    try:
        while True:
            data = data_getter()
            values = [read_f32(data, o) for o in offsets]

            timestamp = time.strftime("%H:%M:%S")
            temp_str = " | ".join(f"0x{o:04X}:{v:5.1f}°C" for o, v in zip(offsets, values) if v)
            print(f"[{timestamp}] {temp_str}")

            time.sleep(interval)
    except KeyboardInterrupt:
        print("\nMonitoring stopped.")


def main():
    parser = argparse.ArgumentParser(description='Analyze AMD SMU PM Table temperatures')
    parser.add_argument('--tctl', action='store_true', help='Search for Tctl temperature')
    parser.add_argument('--soc', action='store_true', help='Search for SoC temperature')
    parser.add_argument('--cores', action='store_true', help='Search for per-core temps')
    parser.add_argument('--compare', action='store_true', help='Compare with system sensors')
    parser.add_argument('--all', action='store_true', help='Run all analyses')
    parser.add_argument('--num-cores', type=int, default=16, help='Number of cores (default: 16)')
    parser.add_argument('--watch', nargs='*', type=str, metavar='OFFSET',
                        help='Monitor specific offsets (hex, e.g., 0x00C 0x534)')

    args = parser.parse_args()

    # Handle watch mode
    if args.watch is not None:
        if not args.watch:
            print("Error: --watch requires offset arguments (e.g., --watch 0x00C 0x534)")
            sys.exit(1)
        offsets = [int(o, 16) for o in args.watch]
        watch_temps(read_pm_table, offsets)
        return

    # If no options specified, show help
    if not any([args.tctl, args.soc, args.cores, args.compare, args.all]):
        parser.print_help()
        print("\nExamples:")
        print("  sudo python3 analyze_temps.py --all")
        print("  sudo python3 analyze_temps.py --cores --num-cores 16")
        print("  sudo python3 analyze_temps.py --watch 0x00C 0x534 0x53C")
        sys.exit(0)

    print_header()
    data = read_pm_table()
    print(f"PM Table Size: {len(data)} bytes\n")

    if args.tctl or args.all:
        search_tctl(data)
        print()

    if args.soc or args.all:
        search_soc_temp(data)
        print()

    if args.cores or args.all:
        search_core_temps(data, args.num_cores)
        print()

    if args.compare or args.all:
        compare_with_sensors(data)
        print()


if __name__ == '__main__':
    main()
