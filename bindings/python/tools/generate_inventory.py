"""Generate source-line-complete public API mapping; fail on unmapped items."""
from pathlib import Path
import re,json
BASE=Path(__file__).resolve().parents[1];ROOT=BASE.parents[1]
# Each entry names the supported public Python object, never a private ABI class.
ALIASES={('EopTable','len'):'EopTable.__len__',('Epoch','from_parts'):'Epoch',('Epoch','checked_from_parts'):'Epoch',('Epoch','checked_from_tai_seconds'):'Epoch.from_tai_seconds',('UtcDateTime','new'):'UtcDateTime',('Duration','from_parts'):'Duration',('Duration','checked_from_parts'):'Duration',('Duration','checked_from_seconds_f64'):'Duration.from_seconds_f64',('Bdt','checked_from_parts'):'Bdt',('Bdt','checked_from_seconds'):'Bdt.from_seconds',('Bdt','checked_from_nanoseconds'):'Bdt.from_nanoseconds',('FormatOptions','with_leap_seconds'):'FormatOptions.with_leap_seconds',('LeapSecondTable','new'):'LeapSecondTable',('SclkKernel','parse'):'SclkKernel.parse',('EopTable','new'):'EopTable',('FormatOptions','new'):'FormatOptions',('Formatter','new'):'Formatter',('TimeContext','new'):'TimeContext',('ScientificContext','new'):'ScientificContext',('LeapSecondPolicy','frozen_after'):'LeapSecondPolicy.frozen_after'}
rows=[]
for p in sorted((ROOT/'src').rglob('*.rs')):
 if 'bin' in p.relative_to(ROOT/'src').parts:continue
 text=p.read_text(encoding="utf-8").split('#[cfg(test)]')[0];owner=None;depth=0;impl_depth=None
 for line_no,line in enumerate(text.splitlines(),1):
  im=re.match(r'impl(?:<.*?>)?\s+(\w+)(?:<[^>]+>)?',line)
  if im and '{' in line:owner=im[1];impl_depth=depth
  found=re.match(r'\s*pub (fn|struct|enum|const|type|trait) (\w+)',line)
  if found:
   kind,name=found.groups(); module='::'.join(p.relative_to(ROOT/'src').with_suffix('').parts)
   if module=='contract::naif_utc':module='contract'
   core=f'{module}::'+(owner+'::' if kind in ('fn','const') and owner else '')+name
   if module=='mcp':target=None;status='unavailable';reason='MCP is an optional stdio transport, not a scientific binding capability.'
   elif kind=='trait':target='SclkClock';status='protocol';reason='Owned SclkClock represents SclkMap; Python structural methods replace Rust trait/lifetimes.'
   else:
    status='wrapped';reason='Direct core delegation; integer narrowing and finite checks occur at the boundary.'
    if owner and kind in ('fn','const'):
     target=ALIASES.get((owner,name),owner+'.'+name)
     if name in ('checked_from_parts','checked_from_tai_seconds','checked_from_seconds_f64','checked_from_seconds','checked_from_nanoseconds'):reason='Checked core constructor; raises instead of returning Rust Option.'
    elif module=='lsk':target='LeapSecondTable.from_lsk' if name=='parse_lsk' else 'LeapSecondTable.builtin'
    elif module=='wgc_gps':
     target={'parse_week_tow':'parse_wgc_week_tow','gps_seconds_to_et':'wgc_gps_seconds_to_et',
             'week_tow_to_et':'wgc_week_tow_to_et','et_to_gps_display_seconds':'et_to_wgc_gps_display_seconds',
             'et_to_calendar_doy':'et_to_wgc_calendar_doy','et_to_calendar_ymd':'et_to_wgc_calendar_ymd',
             'et_to_utc_calendar_doy':'et_to_wgc_calendar_doy'}.get(name,name)
     if name=='et_to_utc_calendar_doy':reason='Select UTC explicitly in the shared Python day-of-year formatter.'
    elif module=='sclk' and name=='NativeSclkError':target='TimeError';reason='Native string and verified range symbols use SclkStringError; other time errors use TimeError.'
    elif module in ('convert','coord','tdb','dtdb') and (name.startswith('tai_to_') or name.endswith('_to_tai') or module=='convert' or name=='tcg_minus_tt'):target='compat.'+name
    else:target=name
   rows.append(dict(core=core,python=target,status=status,source=f'{p.relative_to(ROOT).as_posix()}:{line_no}',reason=reason))
  depth+=line.count('{')-line.count('}')
  if impl_depth is not None and depth<=impl_depth:owner=None;impl_depth=None
(BASE/'API_INVENTORY.json').write_text(json.dumps(rows,indent=2)+'\n')
header='''# Python API coverage inventory

Every public declaration in core src/**/*.rs is listed with its source line.
The Rust core performs all time calculations. Checked constructors raise
TimeError/OverflowError rather than returning Option. Comparison, hashing,
operators and iteration map to Python protocols. Lifetimes map to owned,
immutable kernel/configuration snapshots and ephemeral native borrows.

Each wrapped entry is validated for existence by test_inventory.py. Golden
numerical tests cover the checked UTC, legacy UTC, SCLK, historical UTC,
ERFA geocentric/observer TDB, TCG/TCB, BDT and raw/native JDUTC categories.
Calendar/format/options, EOP parsing/interpolation/UT1, metadata, errors,
series, arithmetic and batch behavior have adapter smoke/boundary tests.
This is a capability inventory, not a claim of individual branch coverage.

The low-level compat namespace deliberately retains the legacy floating
conversion routes. Traits have protocol equivalents; MCP transport functions
remain outside the scientific package. No public scientific declaration is
intentionally omitted. Private decimal/parser internals are exercised through
SCLK/LSK/kernel-date parsers and are not exported as a second scientific API.

| Core declaration | Python surface | Status | Mapping |
| --- | --- | --- | --- |
'''
(BASE/'API_COVERAGE.md').write_text(header+'\n'.join(f"| `{r['core']}` | `{r['python'] or '- '}` | {r['status']} | {r['reason']} |" for r in rows)+'\n')
print(len(rows),'public declarations inventoried')
