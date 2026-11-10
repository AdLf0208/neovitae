#!/usr/bin/env python3
"""Parse a `/neovitae altar X Y Z` line into shell assignments.

The line is

    Ara Vitae at [202, 101, 0]: tier 1, lp 200/18000, reserve 0, progress 0, slot minecraft:diamond x1, clicks 1/0, t=1234

and the script writes `TIER LP CAP RESERVE PROGRESS SLOT USES RIGHT T ok=1`
on stdout for the caller's `eval`. `ok=0` when the line is not an altar
readout, so a wrong reply fails the scenario instead of parsing as zeros.
"""
import re
import sys

TIER = re.compile(r"tier (-?\d+)")
TANK = re.compile(r"lp (\d+)/(\d+)")
RESERVE = re.compile(r"reserve (\d+)")
PROGRESS = re.compile(r"progress (\d+)")
SLOTS = re.compile(r"slots (\d+), slot (.+?), clicks")
CLICKS = re.compile(r"clicks (\d+)/(\d+), t=(\d+)$")


def main() -> int:
    line = sys.argv[1] if len(sys.argv) > 1 else ""
    tier = TIER.search(line)
    tank = TANK.search(line)
    reserve = RESERVE.search(line)
    progress = PROGRESS.search(line)
    slot = SLOTS.search(line)
    clicks = CLICKS.search(line)
    if not (tier and tank and reserve and progress and slot and clicks):
        print("ok=0")
        return 0
    print(
        " ".join(
            [
                f"TIER={tier.group(1)}",
                f"LP={tank.group(1)}",
                f"CAP={tank.group(2)}",
                f"RESERVE={reserve.group(1)}",
                f"PROGRESS={progress.group(1)}",
                f"SLOTS={slot.group(1)}",
                f"SLOT='{slot.group(2)}'",
                f"USES={clicks.group(1)}",
                f"RIGHT={clicks.group(2)}",
                f"T={clicks.group(3)}",
                "ok=1",
            ]
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
