#!/usr/bin/env python3
"""Download the IERS `finals2000A.all` Earth Orientation Parameters file.

This script is *offline tooling*. It is not part of the crate, is not
built by Cargo, and is never a runtime dependency. It performs the one
network fetch that a deployment needs; the crate itself only reads a
local file with `EopTable::from_finals2000a`.

UT1 cannot be derived from a leap-second kernel. It requires this EOP
product, which is a separate data set published by the IERS.

Usage:
    python tools/fetch_eop.py --out data/finals2000A.all
    python tools/fetch_eop.py --out data/finals2000A.all --url MIRROR_URL
    python tools/fetch_eop.py --out data/finals2000A.all --insecure

The endpoint can be set once with the ``STKCRAFT_EOP_URL`` environment
variable; ``--url`` wins over the environment. Any mirror works because
the downloaded file is parsed and summarized immediately.

The file is refreshed daily by IERS. Re-run this script to update it;
the crate never downloads anything at runtime.
"""

from __future__ import annotations

import argparse
import os
import ssl
import sys
import urllib.request
from pathlib import Path

DEFAULT_URL = "https://datacenter.iers.org/data/9/finals2000A.all"
ENV_URL = "STKCRAFT_EOP_URL"

# The `UT1 - UTC` field lives in columns 59-68 (1-based).
UT1_SLICE = slice(58, 68)
MJD_SLICE = slice(6, 15)


def _context(insecure: bool) -> ssl.SSLContext:
    if insecure:
        return ssl._create_unverified_context()
    return ssl.create_default_context()


def _summarize(text: str) -> str:
    rows = 0
    observed = 0
    first_mjd = None
    last_mjd = None
    for line in text.splitlines():
        if len(line) < 68:
            continue
        ut1 = line[UT1_SLICE].strip()
        mjd = line[MJD_SLICE].strip()
        if not ut1 or not mjd:
            continue
        try:
            mjd_value = float(mjd)
        except ValueError:
            continue
        rows += 1
        if line[57:58] != "P":
            observed += 1
        if first_mjd is None:
            first_mjd = mjd_value
        last_mjd = mjd_value

    if rows == 0:
        return "no usable rows were read"

    return (
        f"{rows} rows with UT1-UTC ({observed} observed, "
        f"{rows - observed} predicted), MJD {first_mjd:.1f} .. {last_mjd:.1f}"
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--out",
        required=True,
        help="destination path for finals2000A.all",
    )
    parser.add_argument(
        "--url",
        default=os.environ.get(ENV_URL, DEFAULT_URL),
        help=f"source URL (default: ${ENV_URL} or {DEFAULT_URL})",
    )
    parser.add_argument(
        "--insecure",
        action="store_true",
        help="skip TLS certificate verification (only if required)",
    )
    args = parser.parse_args()

    request = urllib.request.Request(args.url, headers={"User-Agent": "stkcraft-time"})
    context = _context(args.insecure)

    try:
        with urllib.request.urlopen(request, context=context, timeout=120) as response:
            payload = response.read()
    except Exception as exc:  # noqa: BLE001 - surface the network error verbatim
        print(f"download failed: {exc}", file=sys.stderr)
        return 1

    text = payload.decode("utf-8", errors="replace")

    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    # Write the bytes verbatim: text-mode writes translate line endings on
    # Windows and would no longer match the upstream file.
    out.write_bytes(payload)

    print(f"wrote {out} ({len(payload)} bytes)")
    print(_summarize(text))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
