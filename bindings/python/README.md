# STKCraft time for Python

Python 3.10+ API backed directly by the Rust core. Build requires Rust 1.83+
and maturin 1.12.6. Each calculation delegates to Rust; no mission data is
fetched automatically. Only CPython 3.11 on Windows is locally verified.

```python
from stkcraft_time import TimeContext, UtcDateTime
ctx = TimeContext()
et = ctx.utc_to_et(UtcDateTime(2000, 1, 1, 12, 0, 0.0))
print(et.seconds)
```

Build with `maturin build --release --locked`, test the installed wheel with
`python -m pytest tests`, and produce sources with `maturin sdist`.
Maturin includes the exact path-dependent core sources in the sdist and
rewrites dependency paths. Cargo dependencies require registry access on
first build; no offline source-build claim is made. Cargo.lock pins them.
Wheels use the CPython stable ABI (abi3-py310). CPython 3.11 on Windows
is locally verified; CI verifies the other advertised platform/runtime
combinations. Free-threaded Python and PyPy are not claimed.

Epoch origins, scientific range and finite-value checks follow the core
contract. Numeric nanoseconds describe storage resolution, not physical
accuracy. `NaifEt` and `IauTdb` are deliberately distinct runtime classes.
Integer arguments reject bool, float and integers outside their Rust range.
JSON interchange is explicit; integer seconds and nanoseconds are strings.
No implicit `datetime` adapter is provided because it loses leap labels and
nanosecond resolution. File helpers use UTF-8 and caller-supplied paths.

The public scientific surface is indexed in [API_COVERAGE.md](API_COVERAGE.md)
and machine-readable [API_INVENTORY.json](API_INVENTORY.json). Properties
replace Rust readout methods (`epoch.sec`, `lsk.entries`, `kernel.moduli`);
checked constructors/operators raise exceptions instead of returning Option.
`compat` preserves the existing low-level f64 routes; notably its UTC/ET
results may differ in final bits from the newer checked `TimeContext` route.

UTC labels are validated structurally when constructed and against the
selected calendar/LSK when converted. The native facade guards i32 calendar
year representability. Formatting conservatively rejects the final second
of year i32::MAX to prevent a rounded date from wrapping its year. Standalone JD/JDN decomposition conservatively supports
absolute days <=700,000,000,000; `etcal` supports absolute ET <=1e16 seconds.
Precision is accepted through 18 and follows the core's nine-digit clamp.
Legacy inverse adapters preflight the actual inverse Epoch-constructor input;
values that would panic, overflow, wrap years, or produce NaN are errors.
This validation mirrors the exact operation order of the pinned 0.1.32 core
convert/tdb/dtdb/coord implementations (including std sine and fixed-point
iteration). Its result is discarded; the original core converter supplies
every accepted return value. Review these guards whenever the core pin changes.
SCLK field decoding similarly checks reverse-modulus offset additions before
calling the original decoder, preserving valid field values without wrapping.
EOP sample MJDs must fit the same day guard; UT1 inverse correction candidates
must fit Epoch. These boundary guards do not change accepted ordinary mission
data or any golden-reference calculation.

UTC/ET, SCLK and UT1 batch methods cross the native boundary once, reuse
parsed data, abort on an error and include its index. Batch exceptions retain
the scalar exception class/code and expose `.index` separately. They return owned values.
Python sequence validation runs before calling Rust. No NumPy dependency or
GIL-release speedup is claimed. Iteration uses the native lazy TimeSeries.
Datetime adapters are deliberately absent; callers must choose any precision
loss and leap-second policy themselves.

Errors expose a stable core variant `.code` and original debug details.
`ContractError` separates checked model/coverage failures; `DtdbError` reports
observer-model validation. Python TypeError/OverflowError report invalid
language-level integer inputs. Exceptions never substitute a zero or NaN.

Local release verification (CPython 3.11.15, Rust 1.99.1, Windows x86_64):
`cargo fmt --manifest-path bindings/python/Cargo.toml -- --check`,
`cargo clippy --manifest-path bindings/python/Cargo.toml --locked -- -D warnings`,
installed-wheel pytest, strict mypy consumer, and source-distribution rebuild
outside the checkout. Use `tools/verify_artifacts.py WHEEL SDIST` to check wheel
notices/types, compare every bundled core source byte, build in a fresh external
temporary directory, install that rebuilt wheel, and execute the suite.
Mypy 1.19.1 and pytest 9.0.2 are the tested developer-tool versions.
