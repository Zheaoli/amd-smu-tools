#!/usr/bin/env python3
"""
AMD SMU PM Table Analyzer

This script analyzes the raw PM table data from the ryzen_smu kernel module
to help identify field offsets for different processor generations.

Usage:
    sudo python3 analyze_pm_table.py [options]

Options:
    --dump          Dump all float values
    --temps         Search for temperature values (30-95°C)
    --power         Search for power values (5-200W)
    --freq          Search for frequency values (1000-7000 MHz)
    --voltage       Search for voltage values (0.5-2.0V)
    --core-arrays   Search for 8/16 element arrays (per-core data)
    --all           Run all analyses
    --cpuinfo       Compare with /proc/cpuinfo frequencies
"""

import struct
import argparse
import sys
from pathlib import Path

SYSFS_PATH = "/sys/kernel/ryzen_smu_drv"
PM_TABLE_PATH = f"{SYSFS_PATH}/pm_table"
PM_VERSION_PATH = f"{SYSFS_PATH}/pm_table_version"
CODENAME_PATH = f"{SYSFS_PATH}/codename"


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


def read_f32(data, offset):
    """Read little-endian float at offset"""
    if offset + 4 > len(data):
        return None
    return struct.unpack('<f', data[offset:offset+4])[0]


def dump_all(data):
    """Dump all float values"""
    print(f"=== PM Table Dump ({len(data)} bytes) ===\n")
    for i in range(0, len(data), 4):
        value = read_f32(data, i)
        if value is not None:
            label = ""
            if 150 < value < 170:
                label = " <- possible PPT limit"
            elif 200 < value < 250:
                label = " <- possible EDC limit"
            elif 90 < value < 100:
                label = " <- possible TDC limit"
            elif 1500 < value < 3500:
                label = " <- possible FCLK/MCLK"
            elif 4000 < value < 6500:
                label = " <- possible core freq"
            print(f"  0x{i:04X} (field {i//4:3d}): {value:12.4f}{label}")


def search_temps(data):
    """Search for temperature values (30-95°C)"""
    print("=== Temperature Values (30-95°C) ===\n")
    for i in range(0, len(data) - 4, 4):
        value = read_f32(data, i)
        if value and 30 < value < 95:
            print(f"  0x{i:04X}: {value:7.2f}°C")


def search_power(data):
    """Search for power values (5-200W)"""
    print("=== Power Values (5-200W) ===\n")
    for i in range(0, len(data) - 4, 4):
        value = read_f32(data, i)
        if value and 5 < value < 200:
            print(f"  0x{i:04X}: {value:7.2f}W")


def search_freq(data):
    """Search for frequency values (1000-7000 MHz)"""
    print("=== Frequency Values (1000-7000 MHz) ===\n")
    for i in range(0, len(data) - 4, 4):
        value = read_f32(data, i)
        if value and 1000 < value < 7000:
            print(f"  0x{i:04X}: {value:7.1f} MHz")


def search_voltage(data):
    """Search for voltage values (0.5-2.0V)"""
    print("=== Voltage Values (0.5-2.0V) ===\n")
    for i in range(0, len(data) - 4, 4):
        value = read_f32(data, i)
        if value and 0.5 < value < 2.0:
            print(f"  0x{i:04X}: {value:7.4f}V")


def search_core_arrays(data, core_count=16):
    """Search for arrays that might be per-core data"""
    print(f"=== Searching for {core_count}-element arrays ===\n")

    # Search for temperature arrays
    print("Per-core temperature candidates (all values 25-95°C):")
    for base in range(0, len(data) - core_count * 4, 4):
        values = [read_f32(data, base + i * 4) for i in range(core_count)]
        if all(v and 25 < v < 95 for v in values):
            avg = sum(values) / len(values)
            if 30 < avg < 70:
                print(f"\n  0x{base:04X}: avg={avg:.1f}°C")
                for i, v in enumerate(values):
                    print(f"    Core {i:2d}: {v:.1f}°C")

    # Search for power arrays
    print("\n\nPer-core power candidates (all values 0.5-20W):")
    for base in range(0, len(data) - core_count * 4, 4):
        values = [read_f32(data, base + i * 4) for i in range(core_count)]
        if all(v and 0.5 < v < 20 for v in values):
            avg = sum(values) / len(values)
            if 2 < avg < 15:
                print(f"\n  0x{base:04X}: avg={avg:.1f}W")
                for i, v in enumerate(values):
                    print(f"    Core {i:2d}: {v:.2f}W")

    # Search for frequency arrays
    print("\n\nPer-core frequency candidates (most values 400-6000 MHz):")
    for base in range(0, len(data) - core_count * 4, 4):
        values = [read_f32(data, base + i * 4) for i in range(core_count)]
        valid = sum(1 for v in values if v and 400 < v < 6000)
        if valid >= core_count - 2:  # Allow 2 cores to be idle/different
            print(f"\n  0x{base:04X}:")
            for i, v in enumerate(values):
                print(f"    Core {i:2d}: {v:.0f} MHz")


def compare_cpuinfo(data):
    """Compare with /proc/cpuinfo frequencies"""
    print("=== Comparing with /proc/cpuinfo ===\n")

    try:
        with open('/proc/cpuinfo', 'r') as f:
            cpuinfo = f.read()
    except:
        print("Cannot read /proc/cpuinfo")
        return

    freqs = []
    for line in cpuinfo.split('\n'):
        if line.startswith('cpu MHz'):
            try:
                freq = float(line.split(':')[1].strip())
                freqs.append(freq)
            except:
                pass

    if not freqs:
        print("No frequencies found in cpuinfo")
        return

    print(f"Found {len(freqs)} cores in cpuinfo:")
    for i, freq in enumerate(freqs[:16]):
        print(f"  Core {i:2d}: {freq:.1f} MHz")

    print(f"\nSearching for matching values in PM table...")
    sample = freqs[0]
    found = False
    for i in range(0, len(data) - 4, 4):
        value = read_f32(data, i)
        if value and abs(value - sample) < 100:
            print(f"  Found {value:.0f} MHz at 0x{i:04X} (matches Core 0: {sample:.0f} MHz)")
            found = True

    if not found:
        print("  No matching frequencies found in PM table")
        print("  (Frequencies may need to be read from /proc/cpuinfo instead)")


def print_header(data):
    """Print system info header"""
    version = read_pm_version()
    codename_id = read_codename()
    codename = CODENAME_MAP.get(codename_id, f"Unknown ({codename_id})")

    print("=" * 60)
    print("AMD SMU PM Table Analyzer")
    print("=" * 60)
    print(f"Codename:         {codename} (ID: {codename_id})")
    print(f"PM Table Version: 0x{version:08X}")
    print(f"PM Table Size:    {len(data)} bytes")
    print("=" * 60)
    print()


def main():
    parser = argparse.ArgumentParser(description='Analyze AMD SMU PM Table')
    parser.add_argument('--dump', action='store_true', help='Dump all float values')
    parser.add_argument('--temps', action='store_true', help='Search for temperatures')
    parser.add_argument('--power', action='store_true', help='Search for power values')
    parser.add_argument('--freq', action='store_true', help='Search for frequencies')
    parser.add_argument('--voltage', action='store_true', help='Search for voltages')
    parser.add_argument('--core-arrays', action='store_true', help='Search for per-core arrays')
    parser.add_argument('--cpuinfo', action='store_true', help='Compare with cpuinfo')
    parser.add_argument('--all', action='store_true', help='Run all analyses')
    parser.add_argument('--cores', type=int, default=16, help='Number of cores (default: 16)')

    args = parser.parse_args()

    # If no options specified, show help
    if not any([args.dump, args.temps, args.power, args.freq, args.voltage,
                args.core_arrays, args.cpuinfo, args.all]):
        parser.print_help()
        print("\nExample: sudo python3 analyze_pm_table.py --all")
        sys.exit(0)

    data = read_pm_table()
    print_header(data)

    if args.dump or args.all:
        dump_all(data)
        print()

    if args.temps or args.all:
        search_temps(data)
        print()

    if args.power or args.all:
        search_power(data)
        print()

    if args.freq or args.all:
        search_freq(data)
        print()

    if args.voltage or args.all:
        search_voltage(data)
        print()

    if args.core_arrays or args.all:
        search_core_arrays(data, args.cores)
        print()

    if args.cpuinfo or args.all:
        compare_cpuinfo(data)
        print()


if __name__ == '__main__':
    main()
