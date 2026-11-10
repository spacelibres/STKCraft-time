#!/usr/bin/env python3
"""Measure how WGC2 TIME_CONVERSION depends on the selected kernel.

This reproduces the "Kernel selection" behaviour of the JPL WebGeocalc
Time Conversion tool using the public WGC2 API. The same UTC epochs are
converted to TDB with several leap-second kernels:

    naif0012.tls  includes the 2017-01-01 leap second
    naif0011.tls  includes 2015 but not 2017
    naif0010.tls  includes 2012 but neither 2015 nor 2017

The output is a CSV of measured values, written to the path given with
``--out`` (default ``tests/golden/wgc2_kernel_selection.csv``). The CSV
is then checked offline by ``tests/kernel_selection.rs``.

Only the Python standard library is used. Network access is required to
run this script; the crate itself performs no network I/O.

Usage:
    python tools/wgc2_kernel_sweep.py
    python tools/wgc2_kernel_sweep.py --insecure
"""

from __future__ import annotations

import argparse
import json
import ssl
import sys
import time
import urllib.request
from pathlib import Path

API_BASE = "https://wgc2.jpl.nasa.gov:8443/webgeocalc/api"

EPOCHS = [
    "2017-06-01T00:00:00",
    "2015-06-01T00:00:00",
    "2016-12-31T23:59:60.5",
    "2024-01-01T00:00:00",
]

KERNELS = [
    "naif0012",
    "naif0011",
    "naif0010",
]


def request_json(url: str, payload: dict | None, context: ssl.SSLContext) -> dict:
    data = None if payload is None else json.dumps(payload).encode()
    request = urllib.request.Request(
        url,
        data=data,
        headers={"Content-Type": "application/json", "Accept": "application/json"},
        method="POST" if payload is not None else "GET",
    )
    with urllib.request.urlopen(request, context=context, timeout=40) as response:
        return json.loads(response.read().decode())


def convert(utc_times: list[str], kernel: str, context: ssl.SSLContext, timeout: float) -> list[float]:
    payload = {
        "calculationType": "TIME_CONVERSION",
        "kernels": [{"type": "KERNEL", "path": f"pds/wgc/kernels/lsk/{kernel}.tls"}],
        "timeSystem": "UTC",
        "timeFormat": "CALENDAR",
        "times": utc_times,
        "timeStep": 1,
        "timeStepUnits": "SECONDS",
        "outputTimeSystem": "TDB",
        "outputTimeFormat": "SECONDS_PAST_J2000",
    }

    created = request_json(f"{API_BASE}/calculation/new", payload, context)
    if created.get("status") != "OK":
        raise RuntimeError(f"{kernel}: create failed: {created}")

    calculation_id = created["calculationId"]
    deadline = time.time() + timeout

    while time.time() < deadline:
        status = request_json(f"{API_BASE}/calculation/{calculation_id}", None, context)
        phase = status.get("result", {}).get("phase")

        if phase == "COMPLETE":
            results = request_json(
                f"{API_BASE}/calculation/{calculation_id}/results", None, context
            )
            return [float(row[1]) for row in results.get("rows", [])]

        if phase in ("FAILED", "CANCELLED", "EXPIRED"):
            raise RuntimeError(f"{kernel}: calculation {phase}: {status}")

        time.sleep(0.4)

    raise TimeoutError(f"{kernel}: calculation did not complete in {timeout}s")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--out",
        default="tests/golden/wgc2_kernel_selection.csv",
        help="output CSV path",
    )
    parser.add_argument("--insecure", action="store_true")
    parser.add_argument("--timeout", type=float, default=90.0)
    args = parser.parse_args()

    context = (
        ssl._create_unverified_context() if args.insecure else ssl.create_default_context()
    )

    rows: list[tuple[str, str, str]] = []
    for kernel in KERNELS:
        values = convert(EPOCHS, kernel, context, args.timeout)
        if len(values) != len(EPOCHS):
            raise RuntimeError(
                f"{kernel}: expected {len(EPOCHS)} values, got {len(values)}"
            )
        for utc, et in zip(EPOCHS, values):
            rows.append((kernel, utc, repr(et)))
        time.sleep(0.3)

    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    with out.open("w", encoding="utf-8", newline="\n") as handle:
        handle.write(
            "# Measured from the live WGC2 API "
            "(https://wgc2.jpl.nasa.gov:8443/webgeocalc/api/).\n"
        )
        handle.write(
            "# TIME_CONVERSION: UTC calendar -> TDB seconds past J2000, "
            "one individual LSK kernel per row set.\n"
        )
        handle.write(
            "# WGC2 rounds output to about 6 decimals; tests use a 5e-7 s "
            "tolerance.\n"
        )
        handle.write("kernel,utc,et\n")
        for kernel, utc, et in rows:
            handle.write(f"{kernel},{utc},{et}\n")

    print(f"wrote {out} ({len(rows)} rows)")
    for kernel, utc, et in rows:
        print(f"  {kernel:9} {utc:22} {et}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
