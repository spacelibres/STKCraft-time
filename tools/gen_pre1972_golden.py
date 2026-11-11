"""Generate independent historical UTC vectors using PyERFA 2.0.1.5.

Authority: ERFA commit 1a8044cde5b7763295d472a6443387239127c6c8,
https://github.com/liberfa/erfa/blob/1a8044cde5b7763295d472a6443387239127c6c8/src/dat.c
and the dtf2d.c / utctai.c routines at the same revision.
Install with: python -m pip install pyerfa==2.0.1.5
Run from the repository root: python tools/gen_pre1972_golden.py
"""

import csv
import datetime as dt
from decimal import Decimal, ROUND_HALF_UP
from pathlib import Path
import warnings

import erfa

assert erfa.__version__ == "2.0.1.5", erfa.__version__
# Effective dates from ERFA's historical DAT table, plus the 1972 handover.
TRANSITIONS = [
    (1961, 1, 1), (1961, 8, 1), (1962, 1, 1), (1963, 11, 1),
    (1964, 1, 1), (1964, 4, 1), (1964, 9, 1), (1965, 1, 1),
    (1965, 3, 1), (1965, 7, 1), (1965, 9, 1), (1966, 1, 1),
    (1968, 2, 1), (1972, 1, 1),
]


def cases():
    yield (1960, 1, 1, 0, 0, 0.000000001)
    for y, m, d in TRANSITIONS:
        today = dt.date(y, m, d)
        previous = today - dt.timedelta(days=1)
        py, pm, pd = previous.year, previous.month, previous.day
        dat0 = erfa.dat(py, pm, pd, 0.0)
        dat12 = erfa.dat(py, pm, pd, 0.5)
        dat24 = erfa.dat(y, m, d, 0.0)
        limit = 60.0 + dat24 - (2.0 * dat12 - dat0)
        yield (py, pm, pd, 12, 0, 0.123456789)
        yield (py, pm, pd, 23, 59, 59.0)
        yield (py, pm, pd, 23, 59, round(limit - 0.01, 9))
        yield (py, pm, pd, 23, 59, round(limit + 0.01, 9))
        if limit > 60.01:
            yield (py, pm, pd, 23, 59, 60.005)
        yield (y, m, d, 0, 0, 0.0)
        yield (y, m, d, 0, 0, 0.000000001)


def main():
    destination = Path("tests/golden/pre_1972.csv")
    with destination.open("w", newline="", encoding="utf-8") as stream:
        stream.write("# PyERFA 2.0.1.5; ERFA 1a8044cde5b7763295d472a6443387239127c6c8\n")
        stream.write("# dtf2d UTC validation and utctai; split JD preserved with Decimal\n")
        writer = csv.writer(stream, lineterminator="\n")
        writer.writerow(["year", "month", "day", "hour", "minute", "second", "valid", "tai_sec", "tai_nanos"])
        for case in cases():
            with warnings.catch_warnings(record=True) as messages:
                warnings.simplefilter("always", erfa.ErfaWarning)
                u1, u2 = erfa.dtf2d("UTC", *case)
            valid = not messages
            if valid:
                t1, t2 = erfa.utctai(u1, u2)
                # Never combine JD parts or the large absolute second count in f64.
                seconds = ((Decimal.from_float(float(t1)) - Decimal("2451544.5"))
                           + Decimal.from_float(float(t2))) * 86400
                ns = int((seconds * 10**9).to_integral_value(rounding=ROUND_HALF_UP))
                sec, nanos = divmod(ns, 10**9)
            else:
                sec, nanos = "", ""
            writer.writerow([*case, int(valid), sec, nanos])


if __name__ == "__main__":
    main()
