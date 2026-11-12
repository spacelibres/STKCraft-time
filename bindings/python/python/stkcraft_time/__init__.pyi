from __future__ import annotations
from typing import Any, Iterable, Iterator, ClassVar
from dataclasses import dataclass
from os import PathLike
from enum import Enum
__version__: str
__core_version__: str
from . import compat as compat

__version__: Any = ...

class TimeError(ValueError):

    def __init__(self, message: Any, code: Any=None, index: int | None=None) -> None:
        ...
    code: str
    index: int | None

class SclkStringError(TimeError):

    def spice_short(self) -> str:
        ...

class WgcGpsError(TimeError):
    ...

class ContractError(TimeError):
    ...

class DtdbError(TimeError):
    ...

def gps_seconds_to_week_tow(seconds: float, lsk: LeapSecondTable) -> str:
    ...

class Calendar(str, Enum):
    Gregorian = 'Gregorian'
    Julian = 'Julian'
    Mixed = 'Mixed'

class TimeScale(str, Enum):
    Utc = 'Utc'
    Tai = 'Tai'
    Tt = 'Tt'
    Et = 'Et'
    Gps = 'Gps'
    Ut1 = 'Ut1'
    Tcg = 'Tcg'
    Tcb = 'Tcb'
    Bdt = 'Bdt'

class CalculationProfile(str, Enum):
    NaifSpice = 'NaifSpice'
    IauScientific = 'IauScientific'

class ScientificModel(str, Enum):
    GeocentricFairheadBretagnon = 'GeocentricFairheadBretagnon'
    ObserverFairheadBretagnon = 'ObserverFairheadBretagnon'

class Epoch(object):

    def __init__(self, sec: int=0, nanos: int=0) -> None:
        ...

    @classmethod
    def from_tai_seconds(cls, seconds: float) -> Epoch:
        ...

    @property
    def sec(self) -> int:
        ...

    @property
    def nanos(self) -> int:
        ...

    def tai_seconds(self) -> float:
        ...

    def checked_add(self, d: Duration) -> Epoch:
        ...

    def checked_sub(self, d: Duration | Epoch) -> Epoch | Duration:
        ...

    def __add__(self, d: Duration) -> Epoch:
        ...

    def __sub__(self, d: Duration | Epoch) -> Epoch | Duration:
        ...

    def __eq__(self, o: object) -> bool:
        ...

    def __lt__(self, o: Epoch) -> bool:
        ...

    def __le__(self, o: Epoch) -> bool:
        ...

    def __hash__(self) -> int:
        ...

    def __repr__(self) -> str:
        ...

    def to_json(self) -> str:
        ...

    @classmethod
    def from_json(cls, text: str | bytes) -> Epoch:
        ...

class Duration(object):

    def __init__(self, sec: int=0, nanos: int=0) -> None:
        ...

    @classmethod
    def from_nanoseconds(cls, ns: int) -> Duration:
        ...

    @classmethod
    def from_seconds_f64(cls, s: float) -> Duration:
        ...

    @property
    def sec(self) -> int:
        ...

    @property
    def nanos(self) -> int:
        ...

    def total_nanoseconds(self) -> int:
        ...

    def seconds_f64(self) -> float:
        ...

    def signum(self) -> int:
        ...

    def is_zero(self) -> bool:
        ...

    def __add__(self, o: Duration) -> Duration:
        ...

    def __sub__(self, o: Duration) -> Duration:
        ...

    def __neg__(self) -> Duration:
        ...

    def __abs__(self) -> Duration:
        ...

    def __mul__(self, n: int) -> Duration:
        ...
    __rmul__ = __mul__

    def __truediv__(self, n: int) -> Duration:
        ...

    def __eq__(self, o: object) -> bool:
        ...

    def __lt__(self, o: Duration) -> bool:
        ...

    def __le__(self, o: Duration) -> bool:
        ...

    def __hash__(self) -> int:
        ...

    def floor(self, step: Duration) -> Duration:
        ...

    def ceil(self, step: Duration) -> Duration:
        ...

    def round(self, step: Duration) -> Duration:
        ...

    def __str__(self) -> str:
        ...

    def __repr__(self) -> str:
        ...
    checked_add = __add__
    checked_sub = __sub__
    checked_mul_i64 = __mul__
    checked_neg = __neg__
    abs = __abs__

    def to_json(self) -> str:
        ...

    @classmethod
    def from_json(cls, text: str | bytes) -> Duration:
        ...
    ZERO: Duration
    NANOSECOND: Duration
    SECOND: Duration

class UtcDateTime(object):

    def __init__(self, year: int, month: int, day: int, hour: int=0, minute: int=0, second: float=0.0) -> None:
        ...

    @property
    def year(self) -> int:
        ...

    @property
    def month(self) -> int:
        ...

    @property
    def day(self) -> int:
        ...

    @property
    def hour(self) -> int:
        ...

    @property
    def minute(self) -> int:
        ...

    @property
    def second(self) -> float:
        ...

    def __repr__(self) -> str:
        ...

    def __eq__(self, o: Any) -> bool:
        ...

    def __hash__(self) -> int:
        ...

@dataclass(frozen=True)
class NaifEt:
    seconds: float

    def __post_init__(self) -> Any:
        ...

    @classmethod
    def from_seconds(cls, s: float) -> NaifEt:
        ...

    @property
    def profile(self) -> CalculationProfile:
        ...

@dataclass(frozen=True)
class NaifJulianUtc:
    days: float

    def __post_init__(self) -> Any:
        ...

@dataclass(frozen=True)
class IauTdb:
    seconds: float
    model: ScientificModel

    def __post_init__(self) -> Any:
        ...

    @property
    def profile(self) -> CalculationProfile:
        ...

class LeapSecondSource(str, Enum):
    IersOnly = 'IersOnly'
    Sofa = 'Sofa'

@dataclass(frozen=True)
class LeapEntry:
    jdn: int
    delta_at: int

    def __post_init__(self) -> Any:
        ...

@dataclass(frozen=True)
class LeapSecondPolicy:
    jdn: int | None = None

    def __post_init__(self) -> Any:
        ...

    @classmethod
    def frozen_after(cls, year: int, month: int, day: int) -> Any:
        ...
    TableDriven: ClassVar[LeapSecondPolicy]

class LeapSecondTable(object):

    def __init__(self, entries: Iterable[LeapEntry]=None) -> None:
        ...

    @classmethod
    def builtin(cls) -> LeapSecondTable:
        ...

    @classmethod
    def from_lsk(cls, text: str | bytes) -> LeapSecondTable:
        ...

    @classmethod
    def from_file(cls, path: str | PathLike[str]) -> LeapSecondTable:
        ...

    @property
    def entries(self) -> tuple[LeapEntry, ...]:
        ...

    def coverage(self) -> tuple[int, int]:
        ...

    @property
    def constants(self) -> tuple[float, float, float, float, float]:
        ...

    @property
    def source(self) -> LeapSecondSource:
        ...

    @property
    def policy(self) -> LeapSecondPolicy:
        ...

    def with_constants(self, delta_t_a: Any, k: Any, eb: Any, m0: Any, m1: Any) -> LeapSecondTable:
        ...

    def with_source(self, s: LeapSecondSource) -> LeapSecondTable:
        ...

    def with_policy(self, p: LeapSecondPolicy) -> LeapSecondTable:
        ...

    def delta_at_f64(self, jdn: int, day_fraction: Any=0.0) -> Any:
        ...

    def delta_at_for_jdn(self, jdn: int) -> Any:
        ...

    @property
    def delta_t_a(self) -> Any:
        ...

    @property
    def k(self) -> Any:
        ...

    @property
    def eb(self) -> Any:
        ...

    @property
    def m0(self) -> Any:
        ...

    @property
    def m1(self) -> Any:
        ...

@dataclass(frozen=True)
class NaifMetadata:
    leap_entry_dates: tuple[int, int]
    leap_source: LeapSecondSource
    leap_policy: LeapSecondPolicy
    deltet: tuple[float, float, float, float, float]

@dataclass(frozen=True)
class TimeContext:
    lsk: LeapSecondTable = ...

    def __post_init__(self) -> Any:
        ...

    @property
    def profile(self) -> CalculationProfile:
        ...

    @property
    def metadata(self) -> NaifMetadata:
        ...

    def to_et(self, epoch: Epoch) -> NaifEt:
        ...

    def utc_to_et(self, utc: UtcDateTime, calendar: Calendar=Calendar.Gregorian) -> NaifEt:
        ...

    def to_epoch(self, et: NaifEt) -> Epoch:
        ...

    def et_to_utc(self, et: NaifEt, calendar: Calendar=Calendar.Gregorian) -> UtcDateTime:
        ...

    def et_to_spice_julian_utc(self, et: NaifEt) -> NaifJulianUtc:
        ...

    def utc_to_tai(self, utc: UtcDateTime, calendar: Calendar=Calendar.Gregorian) -> Epoch:
        ...

    def tai_to_utc(self, epoch: Epoch, calendar: Calendar=Calendar.Gregorian) -> UtcDateTime:
        ...

    def utc_to_et_batch(self, values: Iterable[UtcDateTime], calendar: Calendar=Calendar.Gregorian) -> list[NaifEt]:
        ...

    def et_to_utc_batch(self, values: Iterable[NaifEt], calendar: Calendar=Calendar.Gregorian) -> list[UtcDateTime]:
        ...

    def convert(self, value: Any, source: TimeScale, target: TimeScale, *, calendar: Calendar=Calendar.Gregorian, eop: EopTable | None=None) -> Any:
        ...

class ScientificContext:

    def tt_seconds(self, e: Epoch) -> float:
        ...

    def to_tdb(self, e: Epoch) -> IauTdb:
        ...

    def to_tdb_at_observer(self, e: Epoch, ut1_fraction: float, longitude: float, u_km: float, v_km: float) -> IauTdb:
        ...

class EopSeries(str, Enum):
    BulletinA = 'BulletinA'
    BulletinB = 'BulletinB'
    FinalPreferred = 'FinalPreferred'

@dataclass(frozen=True)
class EopSample:
    mjd: float
    ut1_minus_utc: float
    observed: bool = True

    def __post_init__(self) -> Any:
        ...

class EopTable(object):

    def __init__(self, samples: Iterable[EopSample]) -> None:
        ...

    @classmethod
    def from_finals2000a(cls, text: str | bytes) -> EopTable:
        ...

    @classmethod
    def from_finals2000a_series(cls, text: str | bytes, series: EopSeries) -> EopTable:
        ...

    @classmethod
    def from_file(cls, path: str | PathLike[str], series: EopSeries=EopSeries.BulletinA) -> EopTable:
        ...

    @property
    def samples(self) -> tuple[EopSample, ...]:
        ...

    def range(self) -> tuple[float, float]:
        ...

    def __len__(self) -> int:
        ...

    def is_empty(self) -> bool:
        ...

    def observed_only(self) -> EopTable:
        ...

    def ut1_minus_utc(self, mjd: float) -> float:
        ...

    def tai_to_ut1(self, epoch: Epoch, lsk: LeapSecondTable) -> float:
        ...

    def ut1_minus_utc_at_epoch(self, epoch: Epoch, lsk: LeapSecondTable) -> float:
        ...

    def ut1_to_tai(self, ut1: float, lsk: LeapSecondTable) -> Epoch:
        ...

    def utc_to_ut1(self, utc: UtcDateTime, lsk: LeapSecondTable, calendar: Calendar=Calendar.Gregorian) -> float:
        ...

    def ut1_to_utc(self, ut1: float, lsk: LeapSecondTable, calendar: Calendar=Calendar.Gregorian) -> Any:
        ...

    def tai_to_ut1_batch(self, values: Iterable[Epoch], lsk: LeapSecondTable) -> list[float]:
        ...

    def ut1_to_tai_batch(self, values: Iterable[float], lsk: LeapSecondTable) -> list[Epoch]:
        ...

class SclkTimeSystem(str, Enum):
    Tdb = 'Tdb'
    Tdt = 'Tdt'

@dataclass(frozen=True)
class SclkCoefficient:
    ticks: float
    seconds: float
    rate: float

class SclkStringContext:

    def __init__(self) -> None:
        ...

class SclkKernel(object):

    def __init__(self, text: str | bytes, spacecraft_id: int) -> None:
        ...

    @classmethod
    def parse(cls, text: str | bytes, spacecraft_id: int) -> SclkKernel:
        ...

    @classmethod
    def from_file(cls, path: str | PathLike[str], spacecraft_id: int) -> SclkKernel:
        ...

    @property
    def spacecraft_id(self) -> int:
        ...

    @property
    def data_type(self) -> int:
        ...

    @property
    def time_system(self) -> SclkTimeSystem:
        ...

    @property
    def n_fields(self) -> int:
        ...

    @property
    def moduli(self) -> tuple[int, ...]:
        ...

    @property
    def offsets(self) -> tuple[int, ...]:
        ...

    @property
    def ticks_per_msb(self) -> float:
        ...

    @property
    def coefficients(self) -> tuple[SclkCoefficient, ...]:
        ...

    @property
    def partitions(self) -> tuple[tuple[float, ...], tuple[float, ...]]:
        ...

    def ticks_from_fields(self, fields: Iterable[int]) -> float:
        ...

    def fields_from_ticks(self, ticks: float) -> tuple[int, ...]:
        ...

    def decode_native_sclk(self, ticks: float) -> str:
        ...

    def encode_native_sclk(self, sclk: str) -> float:
        ...

    def encode_native_sclk_with_context(self, sclk: str, context: SclkStringContext) -> float:
        ...

    def parse_string(self, sclk: str) -> tuple[int, ...]:
        ...

    def format_string(self, fields: Iterable[int]) -> str:
        ...

    def parallel_seconds(self, ticks: float) -> float:
        ...

    def ticks_from_parallel(self, seconds: float) -> float:
        ...

    def ticks_to_et(self, ticks: float, lsk: LeapSecondTable) -> NaifEt:
        ...

    def et_to_ticks(self, et: NaifEt, lsk: LeapSecondTable) -> float:
        ...

    def ticks_to_tai(self, ticks: float, lsk: LeapSecondTable) -> Epoch:
        ...

    def tai_to_ticks(self, epoch: Epoch, lsk: LeapSecondTable) -> float:
        ...

    def ticks_to_et_batch(self, values: Iterable[float], lsk: LeapSecondTable) -> list[NaifEt]:
        ...

    def et_to_ticks_batch(self, values: Iterable[NaifEt], lsk: LeapSecondTable) -> list[float]:
        ...

@dataclass(frozen=True)
class SclkClock:
    kernel: SclkKernel
    lsk: LeapSecondTable

    def sclk_to_et(self, sclk: str) -> NaifEt:
        ...

    def et_to_sclk(self, et: NaifEt) -> str:
        ...

class TimeSeries:

    def __init__(self, start: Epoch, end: Epoch, step: Duration, inclusive: bool=True) -> None:
        ...

    @classmethod
    def inclusive(cls, start: Epoch, end: Epoch, step: Duration) -> Any:
        ...

    @classmethod
    def exclusive(cls, start: Epoch, end: Epoch, step: Duration) -> Any:
        ...

    def __iter__(self) -> TimeSeries:
        ...

    def __next__(self) -> Epoch:
        ...

@dataclass(frozen=True)
class FormatOptions:
    precision: int = 3
    calendar: Calendar = Calendar.Gregorian
    lsk: LeapSecondTable | None = None

    def __post_init__(self) -> Any:
        ...

    def with_calendar(self, c: Calendar) -> FormatOptions:
        ...

    def with_leap_seconds(self, lsk: LeapSecondTable) -> FormatOptions:
        ...

def iso8601(utc: UtcDateTime, precision: int=3) -> str:
    ...

def iso8601_with(utc: UtcDateTime, options: FormatOptions) -> str:
    ...

def rfc3339(utc: UtcDateTime, precision: int=3) -> str:
    ...

def rfc3339_with(utc: UtcDateTime, options: FormatOptions) -> str:
    ...

def spice_calendar(utc: UtcDateTime, precision: int=3) -> str:
    ...

def spice_calendar_with(utc: UtcDateTime, options: FormatOptions) -> str:
    ...

def rfc2822(utc: UtcDateTime) -> str:
    ...

def strftime(utc: UtcDateTime, pattern: str) -> str:
    ...

def strftime_with(utc: UtcDateTime, pattern: str, options: FormatOptions) -> str:
    ...

def etcal(et: NaifEt, precision: int=3) -> str:
    ...

@dataclass(frozen=True)
class Format:
    pattern: str

    def __post_init__(self) -> Any:
        ...

    @classmethod
    def parse(cls, pattern: str) -> Format:
        ...

@dataclass(frozen=True)
class Formatter:
    utc: UtcDateTime
    format: Format
    options: FormatOptions = FormatOptions()

    @classmethod
    def with_options(cls, utc: UtcDateTime, format: Format, options: FormatOptions) -> Formatter:
        ...

    def __str__(self) -> str:
        ...

@dataclass(frozen=True)
class KernelDate:
    seconds_past_j2000: float
    year: int
    month: int
    day: int
    hour: int
    minute: int
    second: float

def parse_kernel_date(text: str | bytes) -> KernelDate:
    ...

def calendar_to_jdn(year: int, month: int, day: int, calendar: Calendar=Calendar.Gregorian) -> int:
    ...

def calendar_to_jd(year: int, month: int, day: int, hour: int=0, minute: int=0, second: float=0.0, calendar: Calendar=Calendar.Gregorian) -> float:
    ...

def jdn_to_calendar(jdn: int, calendar: Calendar=Calendar.Gregorian) -> tuple[int, int, int]:
    ...

def jdn_to_gregorian(jdn: int) -> tuple[int, int, int]:
    ...

def jdn_to_julian(jdn: int) -> tuple[int, int, int]:
    ...

def gregorian_to_jdn(year: int, month: int, day: int) -> Any:
    ...

def julian_to_jdn(year: int, month: int, day: int) -> Any:
    ...

def jd_to_calendar(jd: float) -> tuple[int, int, int, int, int, float]:
    ...

def days_in_month(year: int, month: int, calendar: Calendar=Calendar.Gregorian) -> int | None:
    ...

def is_gregorian_leap(year: int) -> bool:
    ...

def is_julian_leap(year: int) -> bool:
    ...

def jd_to_mjd(jd: float) -> Any:
    ...

def mjd_to_jd(mjd: float) -> Any:
    ...

def sofa_delta_at(mjd: float) -> Any:
    ...

def tcb_minus_tdb(tdb: Any) -> Any:
    ...

def tdb_minus_tt_seconds(jd_tt: float) -> Any:
    ...

def dtdb_minus_tt(jd_tt: float) -> Any:
    ...

def dtdb_minus_tt_split(jd1: float, jd2: float) -> Any:
    ...

def dtdb_minus_tt_at_observer(jd1: float, jd2: float, ut1_fraction: float, east_longitude_radians: float, u_km: float, v_km: float) -> Any:
    ...

@dataclass(frozen=True)
class NaifDeltet:
    delta_t_a: float
    k: float
    eb: float
    m0: float
    m1: float

    def __post_init__(self) -> Any:
        ...

@dataclass(frozen=True)
class NaifUtcContext:
    entries: tuple[LeapEntry, ...]
    deltet: NaifDeltet

    def __post_init__(self) -> Any:
        ...

    @classmethod
    def from_raw_parts(cls, entries: Iterable[LeapEntry], deltet: NaifDeltet) -> NaifUtcContext:
        ...

    def et_to_spice_julian_utc(self, et: NaifEt) -> NaifJulianUtc:
        ...
J2000_JDN = 2451545
GPS_EPOCH_JDN = 2444245
GREGORIAN_SWITCH_JDN = 2299161
DELTA_T_A = 32.184
K = 0.001657
EB = 0.01671
M0 = 6.239996
M1 = 1.99096871e-07
JDN_1960 = 2436935
JDN_1972 = 2441318
L_G = 6.969290134e-10
L_B = 1.550519768e-08
TDB0: Any = ...
__core_version__ = __version__

class Bdt(object):

    def __init__(self, sec: int=0, nanos: int=0) -> None:
        ...

    @classmethod
    def from_nanoseconds(cls, ns: int) -> Bdt:
        ...

    @classmethod
    def from_seconds(cls, seconds: float) -> Bdt:
        ...

    @classmethod
    def from_epoch(cls, e: Epoch) -> Bdt:
        ...

    @classmethod
    def from_week_nanoseconds(cls, week: int, nanoseconds: int) -> Bdt:
        ...

    @classmethod
    def from_utc(cls, utc: UtcDateTime, calendar: Calendar, lsk: LeapSecondTable) -> Bdt:
        ...

    @classmethod
    def from_et(cls, et: NaifEt, context: TimeContext) -> Bdt:
        ...

    @property
    def sec(self) -> int:
        ...

    @property
    def nanos(self) -> int:
        ...

    def total_nanoseconds(self) -> int:
        ...

    def seconds(self) -> float:
        ...

    def week_nanoseconds(self) -> tuple[int, int]:
        ...

    def to_epoch(self) -> Epoch:
        ...

    def to_utc(self, calendar: Calendar, lsk: LeapSecondTable) -> UtcDateTime:
        ...

    def to_et(self, context: TimeContext) -> NaifEt:
        ...

    def __eq__(self, o: Any) -> bool:
        ...

    def __hash__(self) -> int:
        ...

    def __repr__(self) -> str:
        ...

    def to_json(self) -> str:
        ...

    @classmethod
    def from_json(cls, text: str | bytes) -> Bdt:
        ...
TAI_MINUS_BDT_SECONDS = 33
GPS_MINUS_BDT_SECONDS = 14
ORIGIN_TAI_SECONDS = 189388833
NANOSECONDS_PER_WEEK = 604800000000000
