"""Rust-backed astronomical time. All numerical calculations execute in Rust."""
from __future__ import annotations
import json
import math
import operator
from dataclasses import dataclass
from enum import Enum
from pathlib import Path
from . import _native as _n

__version__ = _n.VERSION

class TimeError(ValueError):
    """A Rust time error with a stable variant code and original detail."""
    def __init__(self, message, code=None, index=None):
        super().__init__(message)
        self.index = index
        self.code = code or message.split('(')[0].split(' {')[0].strip('"')
class SclkStringError(TimeError):
    def spice_short(self): return str(self)
class WgcGpsError(TimeError): pass
class ContractError(TimeError): pass
class DtdbError(TimeError): pass

def _call(fn, *args):
    try: return fn(*args)
    except ValueError as e:
        message = str(e)
        cls = WgcGpsError if message.startswith('WgcGps(') else SclkStringError if message.startswith('SPICE(') else ContractError if message.startswith(('Time(', 'Scientific(', 'NonFinite', 'EpochOutOfRange', 'HistoricalUtc', 'NaifKernel', 'InvalidNaif')) else TimeError
        code = message.split('(')[0].split(' {')[0].strip('"')
        if code in ('Time', 'Scientific', 'WgcGps'):
            code = message.split('(', 1)[1].split('(')[0].split(' {')[0].rstrip(')')
        index = getattr(e,'index',None)
        detail = message if index is None else f'{message}; batch index {index}'
        raise cls(detail, code, index) from e

def _int(n, bits=64, signed=True):
    if type(n) is not int: raise TypeError('integer required; bool and floating-point values are rejected')
    if not (-(1 << (bits-1)) if signed else 0) <= n < (1 << (bits-1 if signed else bits)):
        raise OverflowError(f'integer outside {bits}-bit range')
    return n

def _float(n):
    if isinstance(n, bool) or not isinstance(n, (int, float)): raise TypeError('finite number required')
    n = float(n)
    if not math.isfinite(n): raise TimeError('NonFinite', 'NonFinite')
    return n

def _text(value):
    if isinstance(value,bytes): return value.decode('utf-8')
    if not isinstance(value,str): raise TypeError('UTF-8 text or bytes required')
    return value

def gps_seconds_to_week_tow(seconds,lsk):
    if isinstance(seconds,bool) or not isinstance(seconds,(int,float)): raise TypeError('number required')
    if not isinstance(lsk,LeapSecondTable): raise TypeError('LeapSecondTable required')
    value=float(seconds)
    if isinstance(seconds,int) and int(value)!=seconds: raise OverflowError('integer cannot be represented exactly as binary64')
    return _call(_n.wgc_gps_week_tow,value,lsk._n)

class _Immutable:
    __slots__ = ('_n',)
    def __setattr__(self, name, value):
        if hasattr(self, name): raise AttributeError('immutable value')
        object.__setattr__(self, name, value)
    @classmethod
    def _wrap(cls, native):
        obj = object.__new__(cls); obj._n = native; return obj

class Calendar(str, Enum):
    Gregorian = 'Gregorian'
    Julian = 'Julian'
    Mixed = 'Mixed'
def _cal(c): return Calendar(c).value
class TimeScale(str, Enum):
    Utc='Utc'; Tai='Tai'; Tt='Tt'; Et='Et'; Gps='Gps'; Ut1='Ut1'; Tcg='Tcg'; Tcb='Tcb'; Bdt='Bdt'
class CalculationProfile(str, Enum):
    NaifSpice='NaifSpice'; IauScientific='IauScientific'
class ScientificModel(str, Enum):
    GeocentricFairheadBretagnon='GeocentricFairheadBretagnon'
    ObserverFairheadBretagnon='ObserverFairheadBretagnon'

class Epoch(_Immutable):
    """Exact TAI since 2000-01-01 00:00:00 TAI, with integer nanoseconds."""
    __slots__=()
    def __init__(self, sec=0, nanos=0): self._n=_call(_n.NativeEpoch,_int(sec),_int(nanos))
    @classmethod
    def from_tai_seconds(cls, seconds): return cls._wrap(_call(_n.NativeEpoch.from_seconds,_float(seconds)))
    @property
    def sec(self): return self._n.parts()[0]
    @property
    def nanos(self): return self._n.parts()[1]
    def tai_seconds(self): return self._n.seconds()
    def checked_add(self,d): return self+d
    def checked_sub(self,d): return self-d
    def __add__(self,d):
        if not isinstance(d,Duration): return NotImplemented
        return Epoch._wrap(_call(self._n.add,d._n))
    def __sub__(self,d):
        if isinstance(d,Duration): return Epoch._wrap(_call(self._n.sub,d._n))
        if isinstance(d,Epoch): return Duration._wrap(_call(self._n.difference,d._n))
        return NotImplemented
    def __eq__(self,o): return isinstance(o,Epoch) and self._n.parts()==o._n.parts()
    def __lt__(self,o):
        if not isinstance(o,Epoch): return NotImplemented
        return self._n.parts()<o._n.parts()
    def __le__(self,o): return self==o or self<o
    def __hash__(self): return hash(self._n.parts())
    def __repr__(self): return f'Epoch({self.sec}, {self.nanos})'
    def to_json(self): return json.dumps({'schema':'stkcraft-time/v1','scale':'TAI','origin':'2000-01-01T00:00:00 TAI','units':'seconds+nanoseconds','seconds':str(self.sec),'nanoseconds':str(self.nanos)})
    @classmethod
    def from_json(cls,text):
        v=json.loads(text)
        if (v.get('schema'),v.get('scale'),v.get('origin'),v.get('units'))!=('stkcraft-time/v1','TAI','2000-01-01T00:00:00 TAI','seconds+nanoseconds'): raise ValueError('incompatible epoch schema')
        if type(v['seconds']) is not str or type(v['nanoseconds']) is not str: raise TypeError('integer strings required')
        return cls(int(v['seconds']),int(v['nanoseconds']))

class Duration(_Immutable):
    __slots__=()
    def __init__(self,sec=0,nanos=0): self._n=_call(_n.NativeDuration,_int(sec),_int(nanos))
    @classmethod
    def from_nanoseconds(cls,ns): return cls._wrap(_call(_n.NativeDuration.from_ns,_int(ns,128)))
    @classmethod
    def from_seconds_f64(cls,s): return cls._wrap(_call(_n.NativeDuration.from_seconds,_float(s)))
    @property
    def sec(self): return self._n.parts()[0]
    @property
    def nanos(self): return self._n.parts()[1]
    def total_nanoseconds(self): return self._n.total_ns()
    def seconds_f64(self): return self._n.seconds()
    def signum(self): return self._n.signum()
    def is_zero(self): return self.signum()==0
    def __add__(self,o): return Duration._wrap(_call(self._n.add,o._n)) if isinstance(o,Duration) else NotImplemented
    def __sub__(self,o): return Duration._wrap(_call(self._n.sub,o._n)) if isinstance(o,Duration) else NotImplemented
    def __neg__(self): return Duration._wrap(_call(self._n.neg))
    def __abs__(self): return -self if self.signum()<0 else self
    def __mul__(self,n): return Duration._wrap(_call(self._n.mul,_int(n)))
    __rmul__=__mul__
    def __truediv__(self,n): return Duration._wrap(_call(self._n.div,_int(n)))
    def __eq__(self,o): return isinstance(o,Duration) and self.total_nanoseconds()==o.total_nanoseconds()
    def __lt__(self,o): return self.total_nanoseconds()<o.total_nanoseconds() if isinstance(o,Duration) else NotImplemented
    def __le__(self,o): return self==o or self<o
    def __hash__(self): return hash(self.total_nanoseconds())
    def floor(self,step): return Duration._wrap(_call(self._n.quantize,step._n,'floor'))
    def ceil(self,step): return Duration._wrap(_call(self._n.quantize,step._n,'ceil'))
    def round(self,step): return Duration._wrap(_call(self._n.quantize,step._n,'round'))
    def __str__(self): return self._n.display()
    def __repr__(self): return f'Duration({self.sec}, {self.nanos})'
    checked_add=__add__; checked_sub=__sub__; checked_mul_i64=__mul__; checked_neg=__neg__; abs=__abs__
    def to_json(self): return json.dumps({'schema':'stkcraft-time/v1','scale':'duration','origin':'relative','units':'nanoseconds','nanoseconds':str(self.total_nanoseconds())})
    @classmethod
    def from_json(cls,text):
        v=json.loads(text)
        if (v.get('schema'),v.get('scale'),v.get('origin'),v.get('units'))!=('stkcraft-time/v1','duration','relative','nanoseconds') or type(v.get('nanoseconds')) is not str: raise ValueError('incompatible duration schema')
        return cls.from_nanoseconds(int(v['nanoseconds']))
Duration.ZERO=Duration(); Duration.NANOSECOND=Duration(0,1); Duration.SECOND=Duration(1)

class UtcDateTime(_Immutable):
    __slots__=()
    def __init__(self,year,month,day,hour=0,minute=0,second=0.0): self._n=_call(_n.NativeUtc,_int(year,32),_int(month,8,False),_int(day,8,False),_int(hour,8,False),_int(minute,8,False),_float(second))
    @property
    def year(self): return self._n.fields()[0]
    @property
    def month(self): return self._n.fields()[1]
    @property
    def day(self): return self._n.fields()[2]
    @property
    def hour(self): return self._n.fields()[3]
    @property
    def minute(self): return self._n.fields()[4]
    @property
    def second(self): return self._n.fields()[5]
    def __repr__(self): return 'UtcDateTime'+repr(self._n.fields())
    def __eq__(self,o): return isinstance(o,UtcDateTime) and self._n.fields()==o._n.fields()
    def __hash__(self): return hash(self._n.fields())

@dataclass(frozen=True)
class NaifEt:
    seconds: float
    def __post_init__(self): object.__setattr__(self,'seconds',_float(self.seconds))
    @classmethod
    def from_seconds(cls,s): return cls(s)
    @property
    def profile(self): return CalculationProfile.NaifSpice
@dataclass(frozen=True)
class NaifJulianUtc:
    days: float
    def __post_init__(self): object.__setattr__(self,'days',_float(self.days))
@dataclass(frozen=True)
class IauTdb:
    seconds: float
    model: ScientificModel
    def __post_init__(self):
        object.__setattr__(self,'seconds',_float(self.seconds)); object.__setattr__(self,'model',ScientificModel(self.model))
    @property
    def profile(self): return CalculationProfile.IauScientific

def _et(e):
    if type(e) is not NaifEt: raise TypeError('NaifEt required; choose the model explicitly')
    return e.seconds

class LeapSecondSource(str,Enum):
    IersOnly='IersOnly'; Sofa='Sofa'
@dataclass(frozen=True)
class LeapEntry:
    jdn:int
    delta_at:int
    def __post_init__(self): _int(self.jdn); _int(self.delta_at,32)
@dataclass(frozen=True)
class LeapSecondPolicy:
    jdn:int|None=None
    def __post_init__(self):
        if self.jdn is not None: _int(self.jdn)
    @classmethod
    def frozen_after(cls,year,month,day): return cls(calendar_to_jdn(year,month,day))
LeapSecondPolicy.TableDriven=LeapSecondPolicy()

class LeapSecondTable(_Immutable):
    __slots__=()
    def __init__(self,entries=None): self._n=_call(_n.NativeLsk,None) if entries is None else _call(_n.NativeLsk.from_entries,[(e.jdn,e.delta_at) for e in entries])
    @classmethod
    def builtin(cls): return cls()
    @classmethod
    def from_lsk(cls,text): return cls._wrap(_call(_n.NativeLsk,_text(text)))
    @classmethod
    def from_file(cls,path): return cls.from_lsk(Path(path).read_text(encoding='utf-8'))
    @property
    def entries(self): return tuple(LeapEntry(*v) for v in self._n.entries())
    def coverage(self): return self._n.coverage()
    @property
    def constants(self): return self._n.constants()
    @property
    def source(self): return LeapSecondSource(self._n.source())
    @property
    def policy(self): return LeapSecondPolicy(self._n.policy())
    def with_constants(self,delta_t_a,k,eb,m0,m1): return self._wrap(_call(self._n.with_constants,*map(_float,(delta_t_a,k,eb,m0,m1))))
    def with_source(self,s): return self._wrap(_call(self._n.with_source,LeapSecondSource(s).value))
    def with_policy(self,p): return self._wrap(_call(self._n.with_policy,p.jdn))
    def delta_at_f64(self,jdn,day_fraction=0.0): return _call(self._n.delta_at,_int(jdn),_float(day_fraction))
    def delta_at_for_jdn(self,jdn): return self._n.delta_at_for_jdn(_int(jdn))
    @property
    def delta_t_a(self): return self.constants[0]
    @property
    def k(self): return self.constants[1]
    @property
    def eb(self): return self.constants[2]
    @property
    def m0(self): return self.constants[3]
    @property
    def m1(self): return self.constants[4]

@dataclass(frozen=True)
class NaifMetadata:
    leap_entry_dates:tuple[int,int]
    leap_source:LeapSecondSource
    leap_policy:LeapSecondPolicy
    deltet:tuple[float,float,float,float,float]
@dataclass(frozen=True)
class TimeContext:
    lsk:LeapSecondTable=None
    def __post_init__(self):
        if self.lsk is None: object.__setattr__(self,'lsk',LeapSecondTable())
        if not isinstance(self.lsk,LeapSecondTable): raise TypeError('LeapSecondTable required')
    @property
    def profile(self): return CalculationProfile.NaifSpice
    @property
    def metadata(self): return NaifMetadata(self.lsk.coverage(),self.lsk.source,self.lsk.policy,self.lsk.constants)
    def to_et(self,epoch): return NaifEt(_call(self.lsk._n.to_et,epoch._n))
    def utc_to_et(self,utc,calendar=Calendar.Gregorian): return NaifEt(_call(self.lsk._n.utc_to_et,utc._n,_cal(calendar)))
    def to_epoch(self,et): return Epoch._wrap(_call(self.lsk._n.to_epoch,_et(et)))
    def et_to_utc(self,et,calendar=Calendar.Gregorian): return UtcDateTime._wrap(_call(self.lsk._n.et_to_utc,_et(et),_cal(calendar)))
    def et_to_spice_julian_utc(self,et): return NaifJulianUtc(_call(self.lsk._n.julian,_et(et)))
    def utc_to_tai(self,utc,calendar=Calendar.Gregorian): return Epoch._wrap(_call(_n.utc_tai,utc._n,self.lsk._n,_cal(calendar)))
    def tai_to_utc(self,epoch,calendar=Calendar.Gregorian): return UtcDateTime._wrap(_call(_n.tai_utc,epoch._n,self.lsk._n,_cal(calendar)))
    def utc_to_et_batch(self,values,calendar=Calendar.Gregorian): return [NaifEt(x) for x in _call(_n.utc_et_batch,self.lsk._n,[v._n for v in values],_cal(calendar))]
    def et_to_utc_batch(self,values,calendar=Calendar.Gregorian): return [UtcDateTime._wrap(v) for v in _call(_n.et_utc_batch,self.lsk._n,[_et(v) for v in values],_cal(calendar))]
    def convert(self,value,source,target,*,calendar=Calendar.Gregorian,eop=None):
        source,target=TimeScale(source),TimeScale(target)
        if source==TimeScale.Utc: epoch=self.utc_to_tai(value,calendar)
        elif source==TimeScale.Tai:
            if not isinstance(value,Epoch): raise TypeError('Epoch required for TAI')
            epoch=value
        elif source==TimeScale.Et: epoch=self.to_epoch(value)
        elif source==TimeScale.Ut1:
            if eop is None: raise TimeError('EopMissing')
            epoch=eop.ut1_to_tai(value,self.lsk)
        elif source==TimeScale.Bdt: epoch=value.to_epoch()
        else: epoch=compat._from(value,self.lsk,source.value.lower())
        if target==TimeScale.Tai: return epoch
        if target==TimeScale.Utc: return self.tai_to_utc(epoch,calendar)
        if target==TimeScale.Et: return self.to_et(epoch)
        if target==TimeScale.Ut1:
            if eop is None: raise TimeError('EopMissing')
            return eop.tai_to_ut1(epoch,self.lsk)
        if target==TimeScale.Bdt: return Bdt.from_epoch(epoch)
        return compat._to(epoch,self.lsk,target.value.lower())

class ScientificContext:
    __slots__=()
    def tt_seconds(self,e): return _call(_n.scientific_tt,e._n)
    def to_tdb(self,e):
        s,m=_call(_n.scientific,e._n,None); return IauTdb(s,ScientificModel(m))
    def to_tdb_at_observer(self,e,ut1_fraction,longitude,u_km,v_km):
        s,m=_call(_n.scientific,e._n,tuple(map(_float,(ut1_fraction,longitude,u_km,v_km)))); return IauTdb(s,ScientificModel(m))

def _batch(fn,values,*args):
    out=[]
    for i,v in enumerate(values):
        try: out.append(fn(v,*args))
        except (ValueError,TypeError,OverflowError) as e: raise TimeError(f'batch index {i}: {e}',getattr(e,'code','InvalidInput')) from e
    return out
class EopSeries(str,Enum):
    BulletinA='BulletinA'; BulletinB='BulletinB'; FinalPreferred='FinalPreferred'
@dataclass(frozen=True)
class EopSample:
    mjd:float
    ut1_minus_utc:float
    observed:bool=True
    def __post_init__(self):
        _float(self.mjd);_float(self.ut1_minus_utc)
        if type(self.observed) is not bool: raise TypeError('observed must be bool')
class EopTable(_Immutable):
    __slots__=()
    def __init__(self,samples): self._n=_call(_n.NativeEop,[(s.mjd,s.ut1_minus_utc,s.observed) for s in samples])
    @classmethod
    def from_finals2000a(cls,text): return cls.from_finals2000a_series(text,EopSeries.BulletinA)
    @classmethod
    def from_finals2000a_series(cls,text,series): return cls._wrap(_call(_n.NativeEop.parse,_text(text),EopSeries(series).value))
    @classmethod
    def from_file(cls,path,series=EopSeries.BulletinA): return cls.from_finals2000a_series(Path(path).read_text(encoding='utf-8'),series)
    @property
    def samples(self): return tuple(EopSample(*v) for v in self._n.samples())
    def range(self): return self._n.range()
    def __len__(self): return len(self.samples)
    def is_empty(self): return len(self)==0
    def observed_only(self): return self._wrap(_call(self._n.observed_only))
    def ut1_minus_utc(self,mjd): return _call(self._n.dut1,_float(mjd))
    def tai_to_ut1(self,epoch,lsk): return _call(self._n.convert,lsk._n,epoch._n,'ut1')
    def ut1_minus_utc_at_epoch(self,epoch,lsk): return _call(self._n.convert,lsk._n,epoch._n,'offset')
    def ut1_to_tai(self,ut1,lsk): return Epoch._wrap(_call(self._n.to_tai,lsk._n,_float(ut1)))
    def utc_to_ut1(self,utc,lsk,calendar=Calendar.Gregorian): return self.tai_to_ut1(TimeContext(lsk).utc_to_tai(utc,calendar),lsk)
    def ut1_to_utc(self,ut1,lsk,calendar=Calendar.Gregorian): return TimeContext(lsk).tai_to_utc(self.ut1_to_tai(ut1,lsk),calendar)
    def tai_to_ut1_batch(self,values,lsk): return _call(_n.ut1_batch,self._n,lsk._n,[v._n for v in values])
    def ut1_to_tai_batch(self,values,lsk): return [Epoch._wrap(v) for v in _call(_n.ut1_inverse_batch,self._n,lsk._n,[_float(v) for v in values])]
class SclkTimeSystem(str,Enum):
    Tdb='Tdb'; Tdt='Tdt'
@dataclass(frozen=True)
class SclkCoefficient:
    ticks:float
    seconds:float
    rate:float
class SclkStringContext:
    __slots__=('_n',)
    def __init__(self): self._n=_n.NativeSclkStringContext()
class SclkKernel(_Immutable):
    __slots__=()
    def __init__(self,text,spacecraft_id): self._n=_call(_n.NativeSclk,_text(text),_int(spacecraft_id,32))
    @classmethod
    def parse(cls,text,spacecraft_id): return cls(text,spacecraft_id)
    @classmethod
    def from_file(cls,path,spacecraft_id): return cls(Path(path).read_text(encoding='utf-8'),spacecraft_id)
    @property
    def spacecraft_id(self): return self._n.info()[0]
    @property
    def data_type(self): return self._n.info()[1]
    @property
    def time_system(self): return SclkTimeSystem(self._n.info()[2])
    @property
    def n_fields(self): return self._n.info()[3]
    @property
    def moduli(self): return tuple(self._n.info()[4])
    @property
    def offsets(self): return tuple(self._n.info()[5])
    @property
    def ticks_per_msb(self): return self._n.info()[6]
    @property
    def coefficients(self): return tuple(SclkCoefficient(*v) for v in self._n.coefficients())
    @property
    def partitions(self): return tuple(tuple(v) for v in self._n.partitions())
    def ticks_from_fields(self,fields): return _call(self._n.ticks_from_fields,[_int(v) for v in fields])
    def fields_from_ticks(self,ticks): return tuple(_call(self._n.fields_from_ticks,_float(ticks)))
    def decode_native_sclk(self,ticks): return _call(self._n.decode_native_sclk,_float(ticks))
    def encode_native_sclk(self,sclk): return _call(self._n.encode_native_sclk,sclk)
    def encode_native_sclk_with_context(self,sclk,context): return _call(self._n.encode_native_sclk_with_context,sclk,context._n)
    def parse_string(self,sclk): return tuple(_call(self._n.parse_string,sclk))
    def format_string(self,fields): return _call(self._n.format_string,[_int(v) for v in fields])
    def parallel_seconds(self,ticks): return _call(self._n.parallel,_float(ticks),False)
    def ticks_from_parallel(self,seconds): return _call(self._n.parallel,_float(seconds),True)
    def ticks_to_et(self,ticks,lsk): return NaifEt(_call(self._n.convert,_float(ticks),lsk._n,False))
    def et_to_ticks(self,et,lsk): return _call(self._n.convert,_et(et),lsk._n,True)
    def ticks_to_tai(self,ticks,lsk): return Epoch._wrap(_call(self._n.to_tai,_float(ticks),lsk._n))
    def tai_to_ticks(self,epoch,lsk): return _call(self._n.tai_to_ticks,epoch._n,lsk._n)
    def ticks_to_et_batch(self,values,lsk): return [NaifEt(x) for x in _call(_n.sclk_batch,self._n,lsk._n,[_float(v) for v in values],False)]
    def et_to_ticks_batch(self,values,lsk): return _call(_n.sclk_batch,self._n,lsk._n,[_et(v) for v in values],True)
@dataclass(frozen=True)
class SclkClock:
    kernel:SclkKernel
    lsk:LeapSecondTable
    def sclk_to_et(self,sclk): return NaifEt(_call(self.kernel._n.clock_to_et,sclk,self.lsk._n))
    def et_to_sclk(self,et): return _call(self.kernel._n.clock_from_et,_et(et),self.lsk._n)
class TimeSeries:
    __slots__=('_n',)
    def __init__(self,start,end,step,inclusive=True):
        if type(inclusive) is not bool: raise TypeError('inclusive must be bool')
        self._n=_n.NativeSeries(start._n,end._n,step._n,inclusive)
    @classmethod
    def inclusive(cls,start,end,step): return cls(start,end,step,True)
    @classmethod
    def exclusive(cls,start,end,step): return cls(start,end,step,False)
    def __iter__(self): return self
    def __next__(self):
        v=self._n.next()
        if v is None: raise StopIteration
        return Epoch._wrap(v)
@dataclass(frozen=True)
class FormatOptions:
    precision:int=3
    calendar:Calendar=Calendar.Gregorian
    lsk:LeapSecondTable|None=None
    def __post_init__(self):
        _int(self.precision,32,False); _cal(self.calendar)
        if self.precision>18: raise ValueError('precision must be <=18')
    def with_calendar(self,c): return FormatOptions(self.precision,Calendar(c),self.lsk)
    def with_leap_seconds(self,lsk): return FormatOptions(self.precision,self.calendar,lsk)
def _format(utc,kind,options,pattern=''):
    return _call(_n.format_utc,utc._n,kind,options.precision,_cal(options.calendar),options.lsk._n if options.lsk is not None else None,pattern)
def iso8601(utc,precision=3): return iso8601_with(utc,FormatOptions(precision))
def iso8601_with(utc,options): return _format(utc,'iso8601',options)
def rfc3339(utc,precision=3): return rfc3339_with(utc,FormatOptions(precision))
def rfc3339_with(utc,options): return _format(utc,'rfc3339',options)
def spice_calendar(utc,precision=3): return spice_calendar_with(utc,FormatOptions(precision))
def spice_calendar_with(utc,options): return _format(utc,'spice_calendar',options)
def rfc2822(utc): return _format(utc,'rfc2822',FormatOptions())
def strftime(utc,pattern): return strftime_with(utc,pattern,FormatOptions())
def strftime_with(utc,pattern,options): return _format(utc,'strftime',options,pattern)
def etcal(et,precision=3): return _call(_n.etcal,_float(et),_int(precision,32,False))
@dataclass(frozen=True)
class Format:
    pattern:str
    def __post_init__(self): _call(_n.validate_format,self.pattern)
    @classmethod
    def parse(cls,pattern): return cls(pattern)
@dataclass(frozen=True)
class Formatter:
    utc:UtcDateTime
    format:Format
    options:FormatOptions=FormatOptions()
    @classmethod
    def with_options(cls,utc,format,options): return cls(utc,format,options)
    def __str__(self): return strftime_with(self.utc,self.format.pattern,self.options)
@dataclass(frozen=True)
class KernelDate:
    seconds_past_j2000:float
    year:int; month:int; day:int; hour:int; minute:int; second:float
def parse_kernel_date(text):
    seconds,fields=_call(_n.kernel_date,_text(text)); return KernelDate(seconds,*fields)
def calendar_to_jdn(year,month,day,calendar=Calendar.Gregorian): return _call(_n.calendar_to_jdn,_int(year,32),_int(month,8,False),_int(day,8,False),_cal(calendar))
def calendar_to_jd(year,month,day,hour=0,minute=0,second=0.0,calendar=Calendar.Gregorian): return _call(_n.calendar_to_jd,UtcDateTime(year,month,day,hour,minute,second)._n,_cal(calendar))
def jdn_to_calendar(jdn,calendar=Calendar.Gregorian): return _call(_n.jdn_to_calendar,_int(jdn),_cal(calendar))
def jdn_to_gregorian(jdn): return jdn_to_calendar(jdn,Calendar.Gregorian)
def jdn_to_julian(jdn): return jdn_to_calendar(jdn,Calendar.Julian)
def gregorian_to_jdn(year,month,day): return calendar_to_jdn(year,month,day,Calendar.Gregorian)
def julian_to_jdn(year,month,day): return calendar_to_jdn(year,month,day,Calendar.Julian)
def jd_to_calendar(jd): return _call(_n.jd_to_calendar,_float(jd))
def days_in_month(year,month,calendar=Calendar.Gregorian): return _call(_n.days_in_month,_int(year,32),_int(month,8,False),_cal(calendar))
def is_gregorian_leap(year): return _n.leap_year(_int(year,32),False)
def is_julian_leap(year): return _n.leap_year(_int(year,32),True)
def _model(name,x,y=0.0,observer=None): return _call(_n.model,name,_float(x),_float(y),observer)
def jd_to_mjd(jd): return _model('jd_to_mjd',jd)
def mjd_to_jd(mjd): return _model('mjd_to_jd',mjd)
def sofa_delta_at(mjd): return _model('sofa_delta_at',mjd)
def tcb_minus_tdb(tdb): return _model('tcb_minus_tdb',tdb)
def tdb_minus_tt_seconds(jd_tt): return _model('tdb_minus_tt_seconds',jd_tt)
def dtdb_minus_tt(jd_tt): return _model('dtdb_minus_tt',jd_tt)
def dtdb_minus_tt_split(jd1,jd2): return _model('dtdb_minus_tt_split',jd1,jd2)
def dtdb_minus_tt_at_observer(jd1,jd2,ut1_fraction,east_longitude_radians,u_km,v_km):
    try: return _model('dtdb_minus_tt_at_observer',jd1,jd2,tuple(map(_float,(ut1_fraction,east_longitude_radians,u_km,v_km))))
    except TimeError as e: raise DtdbError(str(e),e.code) from e
@dataclass(frozen=True)
class NaifDeltet:
    delta_t_a:float; k:float; eb:float; m0:float; m1:float
    def __post_init__(self):
        for v in (self.delta_t_a,self.k,self.eb,self.m0,self.m1): _float(v)
@dataclass(frozen=True)
class NaifUtcContext:
    entries:tuple[LeapEntry,...]
    deltet:NaifDeltet
    def __post_init__(self):
        object.__setattr__(self,'entries',tuple(self.entries))
        d=self.deltet
        _call(_n.raw_validate,[(e.jdn,e.delta_at) for e in self.entries],(d.delta_t_a,d.k,d.eb,d.m0,d.m1))
    @classmethod
    def from_raw_parts(cls,entries,deltet): return cls(tuple(entries),deltet)
    def et_to_spice_julian_utc(self,et):
        d=self.deltet
        return NaifJulianUtc(_call(_n.raw_julian,[(e.jdn,e.delta_at) for e in self.entries],(d.delta_t_a,d.k,d.eb,d.m0,d.m1),_et(et)))

J2000_JDN=2451545; GPS_EPOCH_JDN=2444245; GREGORIAN_SWITCH_JDN=2299161
DELTA_T_A=32.184; K=1.657e-3; EB=1.671e-2; M0=6.239996; M1=1.99096871e-7
JDN_1960=2436935; JDN_1972=2441318
L_G=6.969290134e-10; L_B=1.550519768e-8; TDB0=-6.55e-5
__core_version__=__version__
from . import compat
class Bdt(_Immutable):
    """Exact BeiDou elapsed time since 2006-01-01 00:00:00 BDT."""
    __slots__=()
    def __init__(self,sec=0,nanos=0): self._n=_call(_n.NativeBdt,_int(sec),_int(nanos))
    @classmethod
    def from_nanoseconds(cls,ns): return cls._wrap(_call(_n.NativeBdt.from_ns,_int(ns,128)))
    @classmethod
    def from_seconds(cls,seconds): return cls._wrap(_call(_n.NativeBdt.from_seconds,_float(seconds)))
    @classmethod
    def from_epoch(cls,e): return cls._wrap(_call(_n.NativeBdt.from_epoch,e._n))
    @classmethod
    def from_week_nanoseconds(cls,week,nanoseconds): return cls._wrap(_call(_n.NativeBdt.from_week,_int(week),_int(nanoseconds,64,False)))
    @classmethod
    def from_utc(cls,utc,calendar,lsk): return cls._wrap(_call(_n.NativeBdt.from_utc,utc._n,_cal(calendar),lsk._n))
    @classmethod
    def from_et(cls,et,context): return cls._wrap(_call(_n.NativeBdt.from_et,_et(et),context.lsk._n))
    @property
    def sec(self): return self._n.parts()[0]
    @property
    def nanos(self): return self._n.parts()[1]
    def total_nanoseconds(self): return self._n.total_ns()
    def seconds(self): return self._n.seconds()
    def week_nanoseconds(self): return self._n.week()
    def to_epoch(self): return Epoch._wrap(_call(self._n.to_epoch))
    def to_utc(self,calendar,lsk): return UtcDateTime._wrap(_call(self._n.to_utc,_cal(calendar),lsk._n))
    def to_et(self,context): return NaifEt(_call(self._n.to_et,context.lsk._n))
    def __eq__(self,o): return isinstance(o,Bdt) and self._n.parts()==o._n.parts()
    def __hash__(self): return hash(self._n.parts())
    def __repr__(self): return f'Bdt({self.sec}, {self.nanos})'
    def to_json(self): return json.dumps({'schema':'stkcraft-time/v1','scale':'BDT','origin':'2006-01-01T00:00:00 BDT','units':'seconds+nanoseconds','seconds':str(self.sec),'nanoseconds':str(self.nanos)})
    @classmethod
    def from_json(cls,text):
        v=json.loads(text)
        if (v.get('schema'),v.get('scale'),v.get('origin'),v.get('units'))!=('stkcraft-time/v1','BDT','2006-01-01T00:00:00 BDT','seconds+nanoseconds') or type(v.get('seconds')) is not str or type(v.get('nanoseconds')) is not str: raise ValueError('incompatible BDT schema')
        return cls(int(v['seconds']),int(v['nanoseconds']))
TAI_MINUS_BDT_SECONDS=33; GPS_MINUS_BDT_SECONDS=14; ORIGIN_TAI_SECONDS=189388833; NANOSECONDS_PER_WEEK=604800000000000
