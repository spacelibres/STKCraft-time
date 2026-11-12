"""Regenerate the checked-in declaration/coverage inventories from adapter syntax."""
from pathlib import Path
import ast,re
BASE=Path(__file__).resolve().parents[1]
# A declaration inventory preserves every public symbol. Explicit annotations
# describe exact time values; remaining dynamic low-level arguments use Any.
PARAM={'s':'float','x':'float','n':'int','e':'Epoch','index':'int | None','sec':'int','nanos':'int','ns':'int','nanoseconds':'int','week':'int','year':'int','month':'int','day':'int','hour':'int','minute':'int','second':'float','seconds':'float','precision':'int','jdn':'int','jd':'float','jd1':'float','jd2':'float','mjd':'float','jd_tt':'float','ticks':'float','ut1':'float','ut1_fraction':'float','longitude':'float','east_longitude_radians':'float','u_km':'float','v_km':'float','text':'str | bytes','pattern':'str','sclk':'str','spacecraft_id':'int','utc':'UtcDateTime','epoch':'Epoch','start':'Epoch','end':'Epoch','step':'Duration','lsk':'LeapSecondTable','context':'TimeContext','calendar':'Calendar','options':'FormatOptions','et':'NaifEt','entries':'Iterable[LeapEntry]','samples':'Iterable[EopSample]','fields':'Iterable[int]','inclusive':'bool','format':'Format','deltet':'NaifDeltet'}
RET={'s':'float','x':'float','n':'int','e':'Epoch','index':'int | None','sec':'int','nanos':'int','total_nanoseconds':'int','seconds':'float','tai_seconds':'float','seconds_f64':'float','signum':'int','is_zero':'bool','is_empty':'bool','__len__':'int','__hash__':'int','__eq__':'bool','__lt__':'bool','__le__':'bool','__repr__':'str','__str__':'str','to_json':'str','to_epoch':'Epoch','utc_to_tai':'Epoch','tai_to_utc':'UtcDateTime','et_to_utc':'UtcDateTime','to_utc':'UtcDateTime','utc_to_et':'NaifEt','to_et':'NaifEt','et_to_spice_julian_utc':'NaifJulianUtc','tt_seconds':'float','to_tdb':'IauTdb','to_tdb_at_observer':'IauTdb','week_nanoseconds':'tuple[int, int]','coverage':'tuple[int, int]','range':'tuple[float, float]','constants':'tuple[float, float, float, float, float]','source':'LeapSecondSource','policy':'LeapSecondPolicy','metadata':'NaifMetadata','profile':'CalculationProfile','fields_from_ticks':'tuple[int, ...]','parse_string':'tuple[int, ...]','format_string':'str','parse_kernel_date':'KernelDate','calendar_to_jdn':'int','calendar_to_jd':'float','jdn_to_calendar':'tuple[int, int, int]','jdn_to_gregorian':'tuple[int, int, int]','jdn_to_julian':'tuple[int, int, int]','jd_to_calendar':'tuple[int, int, int, int, int, float]','is_gregorian_leap':'bool','is_julian_leap':'bool','days_in_month':'int | None','coefficients':'tuple[SclkCoefficient, ...]','partitions':'tuple[tuple[float, ...], tuple[float, ...]]','moduli':'tuple[int, ...]','offsets':'tuple[int, ...]','n_fields':'int','data_type':'int','spacecraft_id':'int','ticks_per_msb':'float','time_system':'SclkTimeSystem','year':'int','month':'int','day':'int','hour':'int','minute':'int','second':'float'}
def declaration(n,owner=None):
 n=ast.fix_missing_locations(n)
 if isinstance(n,(ast.FunctionDef,ast.AsyncFunctionDef)):
  if n.name.startswith('_') and not n.name.startswith('__'):return None
  for a in n.args.posonlyargs+n.args.args+n.args.kwonlyargs:
   if a.annotation is None and a.arg not in ('self','cls'):
    annotation=PARAM.get(a.arg,'Any')
    if owner=='Epoch' and a.arg=='d':annotation='Duration | Epoch' if n.name in ('__sub__','checked_sub') else 'Duration'
    if owner=='Epoch' and a.arg=='o':annotation='object' if n.name=='__eq__' else 'Epoch'
    if owner=='Duration' and a.arg in ('o','d','other'):annotation='object' if n.name=='__eq__' else 'Duration'
    if owner=='LeapSecondTable' and a.arg=='s':annotation='LeapSecondSource'
    if owner=='LeapSecondTable' and a.arg=='p':annotation='LeapSecondPolicy'
    if owner=='FormatOptions' and a.arg=='c':annotation='Calendar'
    if a.arg=='values':
     annotation='Iterable[NaifEt]' if n.name in ('et_to_utc_batch','et_to_ticks_batch') else 'Iterable[UtcDateTime]' if n.name=='utc_to_et_batch' else 'Iterable[float]' if n.name in ('ticks_to_et_batch','ut1_to_tai_batch') else 'Iterable[Epoch]'
    if a.arg=='path':annotation='str | PathLike[str]'
    if a.arg=='source' or a.arg=='target':annotation='TimeScale'
    if a.arg=='eop':annotation='EopTable | None'
    if a.arg=='series':annotation='EopSeries'
    a.annotation=ast.parse(annotation,mode='eval').body
  if n.returns is None:
   result=RET.get(n.name,'Any')
   if n.name=='__init__':result='None'
   elif n.name=='__iter__':result='TimeSeries'
   elif n.name=='__next__':result='Epoch'
   elif owner and (n.name.startswith('from_') or n.name.startswith('with_') or n.name in ('builtin','parse','observed_only','floor','ceil','round','abs','checked_neg','checked_add','checked_sub','checked_mul_i64')):result=owner
   elif n.name in ('iso8601','iso8601_with','rfc3339','rfc3339_with','spice_calendar','spice_calendar_with','rfc2822','strftime','strftime_with','etcal'):result='str'
   elif (n.name not in RET and n.name.startswith('tai_to_')) or n.name in ('ut1_minus_utc','utc_to_ut1','ut1_minus_utc_at_epoch','ut1_minus_utc','parallel_seconds','ticks_from_parallel','ticks_from_fields','et_to_ticks','tai_to_ticks'):result='float'
   elif n.name not in RET and n.name.endswith('_to_tai'):result='Epoch'
   elif n.name=='ticks_to_et':result='NaifEt'
   elif n.name=='ticks_to_tai':result='Epoch'
   if owner=='Epoch' and n.name in ('__add__','checked_add'):result='Epoch'
   if owner=='Epoch' and n.name in ('__sub__','checked_sub'):result='Epoch | Duration'
   if owner=='Duration' and n.name in ('__add__','__sub__','__neg__','__abs__','__mul__','__truediv__'):result='Duration'
   if module=='compat' and n.name=='utc_to_et':result='float'
   if owner=='LeapSecondTable' and n.name=='entries':result='tuple[LeapEntry, ...]'
   if owner=='EopTable' and n.name=='samples':result='tuple[EopSample, ...]'
   if owner=='TimeContext' and n.name=='utc_to_et_batch':result='list[NaifEt]'
   if owner=='TimeContext' and n.name=='et_to_utc_batch':result='list[UtcDateTime]'
   if owner=='SclkKernel' and n.name=='ticks_to_et_batch':result='list[NaifEt]'
   if owner=='SclkKernel' and n.name=='et_to_ticks_batch':result='list[float]'
   if owner=='EopTable' and n.name=='tai_to_ut1_batch':result='list[float]'
   if owner=='EopTable' and n.name=='ut1_to_tai_batch':result='list[Epoch]'
   if n.name=='sclk_to_et':result='NaifEt'
   if n.name=='et_to_sclk':result='str'
   n.returns=ast.parse(result,mode='eval').body
  n.body=[ast.Expr(value=ast.Constant(value=Ellipsis))];return n
 if isinstance(n,ast.ClassDef):
  if n.name.startswith('_'):return None
  n.bases=[ast.Name(id='object',ctx=ast.Load()) if isinstance(b,ast.Name) and b.id=='_Immutable' else b for b in n.bases]
  n.body=[x for q in n.body if (x:=declaration(q,n.name)) is not None] or [ast.Expr(value=ast.Constant(value=Ellipsis))]
  if n.name=='Duration': n.body.extend(ast.parse('ZERO: Duration\nNANOSECOND: Duration\nSECOND: Duration').body)
  if n.name=='LeapSecondPolicy':n.body.extend(ast.parse('TableDriven: ClassVar[LeapSecondPolicy]').body)
  if n.name=='TimeError':n.body.extend(ast.parse('code: str\nindex: int | None').body)
  return n
 if isinstance(n,ast.AnnAssign):
  if owner=='TimeContext' and isinstance(n.target,ast.Name) and n.target.id=='lsk':n.value=ast.Constant(value=Ellipsis)
  return n
 if isinstance(n,ast.Assign):
  names=[x.id for x in n.targets if isinstance(x,ast.Name)]
  if names and (not names[0].startswith('_') or (names[0].startswith('__') and names[0] not in ('__slots__',))):
   if isinstance(n.value,ast.Constant):return n
   if isinstance(n.value,ast.Name) and n.value.id.startswith('__'):return n
   if owner and isinstance(n.value,ast.Name) and n.value.id.startswith('__'):return n
   return ast.AnnAssign(target=n.targets[0],annotation=ast.Name(id='Any',ctx=ast.Load()),value=ast.Constant(value=Ellipsis),simple=1)
 return None
for module in ('__init__','compat'):
 tree=ast.parse((BASE/f'python/stkcraft_time/{module}.py').read_text())
 body=[x for q in tree.body if (x:=declaration(q)) is not None]
 pre='from __future__ import annotations\nfrom typing import Any, Iterable, Iterator, ClassVar\nfrom dataclasses import dataclass\nfrom os import PathLike\nfrom enum import Enum\n'
 pre+= '__version__: str\n__core_version__: str\nfrom . import compat as compat\n' if module=='__init__' else 'from . import Epoch, UtcDateTime, LeapSecondTable, Calendar\n'
 out=ast.unparse(ast.fix_missing_locations(ast.Module(body=body,type_ignores=[])))
 (BASE/f'python/stkcraft_time/{module}.pyi').write_text(pre+'\n'+out+'\n')
