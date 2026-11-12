import csv, json, math, struct
from pathlib import Path
import pytest
import stkcraft_time as t
ROOT=Path(__file__).resolve().parents[3]
def rows(name):
    with (ROOT/'tests'/'golden'/name).open() as f: return list(csv.DictReader(line for line in f if not line.startswith('#')))
def utc(text):
    date,clock=text.split('T'); y,m,d=map(int,date.split('-'));h,mi,s=clock.split(':');return t.UtcDateTime(y,m,d,int(h),int(mi),float(s))
def bits(x):return struct.pack('>d',x)

def test_exact_boundaries_and_errors():
    for n in [-(2**63),2**53+1,2**63-1]:
        e=t.Epoch(n,1); assert e.sec==n and e.nanos==1
        assert t.Epoch.from_json(e.to_json())==e
    assert t.Epoch(0,-1)==t.Epoch(-1,999999999)
    assert t.Epoch(2**53+1,0)+t.Duration.NANOSECOND==t.Epoch(2**53+1,1)
    for v in [True,1.5,'1']:
        with pytest.raises(TypeError):t.Epoch(v)
    for v in [2**63,-2**63-1]:
        with pytest.raises(OverflowError):t.Epoch(v)
    with pytest.raises(t.TimeError): t.Epoch(2**63-1,10**9)
    for v in [float('nan'),float('inf'),-float('inf')]:
        with pytest.raises(t.TimeError):t.Epoch.from_tai_seconds(v)
    d=t.Duration.from_nanoseconds(2**63*10**9-1)
    assert t.Duration.from_json(d.to_json())==d
    assert str(t.Duration(0,-1)).startswith('-')
    assert t.Duration(1)/2==t.Duration(0,500000000)
    with pytest.raises(t.TimeError):t.Duration(1)/0
    with pytest.raises(AttributeError):t.Epoch()._n=None

def test_checked_contract_oracle():
    ctx=t.TimeContext(); count=0
    for r in rows('contract_utc.csv'):
        e=ctx.utc_to_et(t.UtcDateTime(*(int(r[k]) for k in ('year','month','day','hour','minute')),float(r['second'])))
        assert bits(e.seconds)==bits(float(r['et_seconds'])),r
        count+=1
    assert count>100
    with pytest.raises(TypeError):ctx.to_epoch(t.IauTdb(0,t.ScientificModel.GeocentricFairheadBretagnon))
    with pytest.raises(t.ContractError):ctx.utc_to_et(t.UtcDateTime(1960,1,1))
    with pytest.raises(t.TimeError):ctx.utc_to_tai(t.UtcDateTime(2016,1,1,23,59,60))
    assert ctx.et_to_spice_julian_utc(t.NaifEt(0)).days>2451544
    assert ctx.metadata.deltet==ctx.lsk.constants

def test_erfa_geocentric_and_observer_oracles():
    for r in rows('dtdb.csv'):
        assert abs(t.dtdb_minus_tt(float(r['jd_tt']))-float(r['dtdb']))<1e-15
    for r in rows('dtdb_observer.csv'):
        args=[float(r[k]) for k in ('jd1','jd2','ut1_fraction','east_longitude_radians','u_km','v_km')]
        assert abs(t.dtdb_minus_tt_at_observer(*args)-float(r['tdb_minus_tt_seconds']))<1e-15
    sci=t.ScientificContext(); assert isinstance(sci.to_tdb(t.Epoch()),t.IauTdb)
    assert sci.tt_seconds(t.Epoch(43200))==32.184
    with pytest.raises(t.ContractError):sci.to_tdb(t.Epoch(2**63-1))

def test_sclk_oracle():
    lsk=t.LeapSecondTable(); kernels={}
    data=rows('sclk.csv')
    assert data
    for r in data:
        sid=int(r['clock']); kname='MRO_SCLKSCET.00120.tsc' if sid==-74 else 'juice_step_250412_v01.tsc'
        if sid not in kernels: kernels[sid]=t.SclkKernel((ROOT/'tests'/'kernels'/kname).read_text(),sid)
        k=kernels[sid]
        x=struct.unpack('>d',bytes.fromhex(r['input_bits']))[0]
        expected=bytes.fromhex(r['expected_bits'])
        actual=k.ticks_to_et(x,lsk).seconds if r['direction']=='forward' else k.et_to_ticks(t.NaifEt(x),lsk)
        assert bits(actual)==expected,r

def test_calendar_format_series_eop():
    assert t.calendar_to_jdn(2000,1,1)==2451545
    assert t.jdn_to_gregorian(2451545)==(2000,1,1)
    assert t.parse_kernel_date('@2001-FEB-29').day==1
    assert t.is_gregorian_leap(2000)
    assert t.days_in_month(2000,2)==29
    u=t.UtcDateTime(2000,1,1)
    assert t.iso8601(u).startswith('2000-01-01')
    assert str(t.Formatter(u,t.Format('%Y')))=='2000'
    assert list(t.TimeSeries.inclusive(t.Epoch(),t.Epoch(2),t.Duration.SECOND))==[t.Epoch(),t.Epoch(1),t.Epoch(2)]
    l=t.LeapSecondTable();eop=t.EopTable([t.EopSample(51544,0.1),t.EopSample(51545,0.2)])
    assert eop.ut1_minus_utc(51544.5)==pytest.approx(0.15)
    e=t.TimeContext(l).utc_to_tai(u)
    assert abs((eop.ut1_to_tai(eop.tai_to_ut1(e,l),l)-e).seconds_f64())<1e-8
    assert eop.observed_only().range()==eop.range()

def test_bdt_and_batches():
    b=t.Bdt.from_utc(t.UtcDateTime(2006,1,1),t.Calendar.Gregorian,t.LeapSecondTable())
    assert b.sec==0 and b.nanos==0
    assert t.Bdt.from_week_nanoseconds(-1,604799999999999)==t.Bdt(-1,999999999)
    assert b.to_epoch()==t.Epoch(189388833)
    ctx=t.TimeContext(); values=[t.UtcDateTime(2000,1,1),t.UtcDateTime(2016,12,31,23,59,60.5)]
    assert ctx.utc_to_et_batch(values)==[ctx.utc_to_et(v) for v in values]
    assert ctx.et_to_utc_batch(ctx.utc_to_et_batch(values))==[ctx.et_to_utc(ctx.utc_to_et(v)) for v in values]
def test_legacy_historical_coord_bdt_corpora():
    l=t.LeapSecondTable();ctx=t.TimeContext(l)
    for r in rows('utc_to_et.csv'):
        assert abs(t.compat.utc_to_et(utc(r['utc']),t.Calendar.Gregorian,l)-float(r['et']))<1e-6
    historical=t.TimeContext(l.with_source(t.LeapSecondSource.Sofa))
    for r in rows('pre_1972.csv'):
        u=t.UtcDateTime(*(int(r[k]) for k in ('year','month','day','hour','minute')),float(r['second']))
        if r['valid']=='0':
            with pytest.raises(t.TimeError):historical.utc_to_tai(u)
        else:
            e=historical.utc_to_tai(u)
            assert (e.sec,e.nanos)==(int(r['tai_sec']),int(r['tai_nanos']))
    for r in rows('tcg_tcb.csv'):
        u=utc(r['tai']);e=t.Epoch(t.calendar_to_jdn(u.year,u.month,u.day)*86400-t.J2000_JDN*86400+u.hour*3600+u.minute*60,int(u.second*1e9))
        assert abs(t.compat.tai_to_tcg(e,l)-float(r['tcg']))<5e-7
        assert abs(t.compat.tai_to_tcb(e,l)-float(r['tcb']))<1e-6
    for r in rows('bdt.csv'):
        u=t.UtcDateTime(*(int(r[k]) for k in ('year','month','day','hour','minute')),float(r['second']))
        b=t.Bdt.from_utc(u,t.Calendar.Gregorian,l)
        assert (b.sec,b.nanos)==(int(r['bdt_sec']),int(r['bdt_nanos']))
        assert t.Bdt.from_json(b.to_json())==b

def test_native_raw_julian_oracles():
    def f64(s):return struct.unpack('>d',bytes.fromhex(s))[0]
    for name in ('native.tsv','raw.tsv','original-lsk.tsv'):
        contexts={}
        for line in (ROOT/'tests'/'reference'/'native-jdutc'/name).read_text().splitlines():
            f=line.split('|')
            if f[0]=='C':
                n=int(f[2]);constants=t.NaifDeltet(*(f64(s) for s in f[3:8]));entries=[t.LeapEntry(int(f[8+i*2]),int(f[9+i*2])) for i in range(n)]
                if f[-1]=='OK': contexts[f[1]]=t.NaifUtcContext.from_raw_parts(entries,constants)
                continue
            if f[0]!='R' or f[1] not in contexts:continue
            if f[3]=='N': assert bits(contexts[f[1]].et_to_spice_julian_utc(t.NaifEt(f64(f[2]))).days)==bytes.fromhex(f[4])
            else:
                with pytest.raises(t.TimeError):contexts[f[1]].et_to_spice_julian_utc(t.NaifEt(f64(f[2])))

def test_extra_input_and_lowlevel_errors():
    l=t.LeapSecondTable()
    with pytest.raises(t.TimeError): t.Epoch(2**63-1)-t.Epoch(-2**63)
    with pytest.raises(TypeError): t.Duration.from_nanoseconds(True)
    with pytest.raises(OverflowError): t.Duration.from_nanoseconds(2**128)
    with pytest.raises(t.TimeError): t.Duration(-2**63).checked_neg()
    with pytest.raises(t.TimeError): t.Bdt(2**63-1).to_epoch()
    with pytest.raises(t.TimeError): t.LeapSecondTable.from_lsk('garbage')
    with pytest.raises(t.TimeError): t.EopTable.from_finals2000a('garbage')
    with pytest.raises(t.DtdbError): t.dtdb_minus_tt_at_observer(2451545,0,-1,0,0,0)
    ctx=t.TimeContext(l)
    with pytest.raises(t.TimeError,match='index 1'):ctx.utc_to_et_batch([t.UtcDateTime(2000,1,1),t.UtcDateTime(1900,1,1)])
    for scale in ('tt','gps','tcg','tcb','tdb','tdb_iau'):
        forward=getattr(t.compat,'tai_to_'+scale);back=getattr(t.compat,scale+'_to_tai')
        s=forward(t.Epoch(43200),l) if scale!='gps' else forward(t.Epoch(43200))
        e=back(s,l) if scale!='gps' else back(s)
        assert abs((e-t.Epoch(43200)).seconds_f64())<1e-6
def test_extreme_legacy_and_civil_guards():
    l=t.LeapSecondTable();ctx=t.TimeContext(l)
    for e in (t.Epoch(2**63-1),t.Epoch(-2**63)):
        with pytest.raises(t.TimeError):ctx.tai_to_utc(e)
        with pytest.raises(t.TimeError):ctx.et_to_utc(ctx.to_et(e))
    for scale in ('et','tt','gps','tcg','tcb','tdb','tdb_iau'):
        f=getattr(t.compat,scale+'_to_tai')
        for x in (1e300,-1e300):
            with pytest.raises(t.TimeError):f(x,l) if scale!='gps' else f(x)
    with pytest.raises(t.TimeError):t.Duration(-2**63).floor(t.Duration(3))
    with pytest.raises(t.TimeError):t.jd_to_calendar(1e300)
    with pytest.raises(t.TimeError):t.NaifUtcContext.from_raw_parts([],t.NaifDeltet(*l.constants))
def test_legacy_routes_match_literal_core_outputs():
    l=t.LeapSecondTable();different=0
    with (Path(__file__).parent/'legacy_routes.csv').open() as f:
        for r in csv.DictReader(f):
            u=t.UtcDateTime(*(int(r[k]) for k in ('year','month','day','hour','minute')),float(r['second']))
            legacy=t.compat.utc_to_et(u,t.Calendar.Gregorian,l)
            checked=t.TimeContext(l).utc_to_et(u).seconds
            assert bits(legacy)==bytes.fromhex(r['legacy_et_bits'])
            assert bits(checked)==bytes.fromhex(r['checked_et_bits'])
            assert bits(t.compat.et_to_utc(legacy,t.Calendar.Gregorian,l).second)==bytes.fromhex(r['legacy_inverse_second_bits'])
            different+=legacy!=checked
    assert different>0

def test_kernel_dates_and_kernel_selection_oracles():
    with (ROOT/'tests'/'reference'/'kernel-dates'/'native.tsv').open() as f:
        for r in csv.DictReader(f,delimiter='\t'):
            if r['native_load']=='OK':assert bits(t.parse_kernel_date(r['literal']).seconds_past_j2000)==bytes.fromhex(r['native_value'].split('|')[-1])
            else:
                with pytest.raises(t.TimeError) as e:t.parse_kernel_date(r['literal'])
                assert e.value.code=='KernelDateUnsupported'
    full=t.LeapSecondTable();tables={'naif0012':full}
    for name,cutoff in [('naif0011',t.calendar_to_jdn(2017,1,1)),('naif0010',t.calendar_to_jdn(2015,7,1))]:tables[name]=t.LeapSecondTable([e for e in full.entries if e.jdn<cutoff])
    for r in rows('wgc2_kernel_selection.csv'):
        if '23:59:60' in r['utc'] and r['kernel']!='naif0012':
            with pytest.raises(t.TimeError) as e:t.compat.utc_to_et(utc(r['utc']),t.Calendar.Gregorian,tables[r['kernel']])
            assert e.value.code=='InvalidLeapSecond'
            continue
        assert abs(t.compat.utc_to_et(utc(r['utc']),t.Calendar.Gregorian,tables[r['kernel']])-float(r['et']))<5e-7
def test_structural_values_and_unmarked_nan_are_rejected():
    with pytest.raises(t.TimeError):t.UtcDateTime(2000,1,1,0,0,1e300)
    with pytest.raises(t.TimeError):t.dtdb_minus_tt(1e300)
    with pytest.raises(t.TimeError):t.EopTable([t.EopSample(1e300,0)])
def test_remaining_scalar_surface_smoke():
    l=t.LeapSecondTable();u=t.UtcDateTime(2000,1,1,12,0,0.25);ctx=t.TimeContext(l);e=ctx.utc_to_tai(u);et=ctx.utc_to_et(u)
    assert l.with_source(t.LeapSecondSource.Sofa).source==t.LeapSecondSource.Sofa
    assert l.with_policy(t.LeapSecondPolicy.frozen_after(2030,1,1)).policy.jdn==t.calendar_to_jdn(2030,1,1)
    assert l.with_constants(*l.constants).constants==l.constants
    assert l.delta_at_for_jdn(t.J2000_JDN)==l.delta_at_f64(t.J2000_JDN)
    assert t.TimeContext(l.with_policy(t.LeapSecondPolicy.TableDriven)).metadata.leap_policy==t.LeapSecondPolicy.TableDriven
    opts=t.FormatOptions(6).with_calendar(t.Calendar.Gregorian).with_leap_seconds(l)
    for fn in (t.iso8601_with,t.rfc3339_with,t.spice_calendar_with):assert '2000' in fn(u,opts)
    assert t.rfc2822(u) and t.strftime(u,'%Y')=='2000'
    assert str(t.Formatter.with_options(u,t.Format.parse('%Y'),opts))=='2000'
    assert t.etcal(0) and t.jd_to_calendar(2451545)[:3]==(2000,1,1)
    assert t.jd_to_mjd(2451545)==51544.5 and t.mjd_to_jd(51544.5)==2451545
    assert t.tdb_minus_tt_seconds(2451545) and t.dtdb_minus_tt_split(2451545,1e-10)
    assert t.sofa_delta_at(36934)==pytest.approx(0.943482) and t.tcb_minus_tdb(0)
    assert t.compat.tcg_minus_tt(e,l)
    assert ctx.convert(u,t.TimeScale.Utc,t.TimeScale.Et)==ctx.to_et(e)
    assert ctx.convert(et,t.TimeScale.Et,t.TimeScale.Tai)==ctx.to_epoch(et)
    b=t.Bdt.from_epoch(e);assert b.to_utc(t.Calendar.Gregorian,l)==u
    assert t.Bdt.from_nanoseconds(b.total_nanoseconds())==b
    assert t.Bdt.from_seconds(0).seconds()==0
    assert t.Bdt.from_et(b.to_et(ctx),ctx).to_epoch()==ctx.to_epoch(b.to_et(ctx))
    assert t.Duration(3).floor(t.Duration(2))==t.Duration(2)
    assert t.Duration(3).ceil(t.Duration(2))==t.Duration(4)
    assert t.Duration(3).round(t.Duration(2))==t.Duration(4)
    assert abs(t.Duration(-1))==t.Duration(1)
    text=(ROOT/'tests'/'ut1.rs').read_text(encoding='utf-8');snippet=text.split('const FINALS_SNIPPET: &str = "\\\n')[1].split('\n";')[0]
    assert len(t.EopTable.from_finals2000a(snippet))==3
    assert len(t.EopTable.from_finals2000a_series(snippet,t.EopSeries.FinalPreferred))==3
    eop=t.EopTable([t.EopSample(51544,0.1),t.EopSample(51545,0.2)])
    base=t.TimeContext(l).utc_to_tai(t.UtcDateTime(2000,1,1))
    ut=eop.tai_to_ut1(base,l)
    assert eop.tai_to_ut1_batch([base],l)==[ut]
    assert eop.ut1_to_tai_batch([ut],l)==[eop.ut1_to_tai(ut,l)]
    assert t.compat.ut1_minus_utc(base,l,eop)==eop.ut1_minus_utc_at_epoch(base,l)
    assert t.compat.utc_to_ut1(t.UtcDateTime(2000,1,1),t.Calendar.Gregorian,l,eop)==ut
    k=t.SclkKernel((ROOT/'tests'/'kernels'/'MRO_SCLKSCET.00120.tsc').read_text(),-74)
    assert k.spacecraft_id==-74 and k.data_type==1 and k.n_fields==2 and k.moduli and k.offsets and k.coefficients and k.partitions
    first=k.coefficients[0].ticks; parallel=k.parallel_seconds(first)
    assert k.ticks_from_parallel(parallel)==first
    assert k.ticks_to_et_batch([first],l)==[k.ticks_to_et(first,l)]
    assert k.et_to_ticks_batch(k.ticks_to_et_batch([first],l),l)==[k.et_to_ticks(k.ticks_to_et(first,l),l)]
    assert isinstance(k.ticks_to_tai(first,l),t.Epoch)
    assert math.isfinite(k.tai_to_ticks(k.ticks_to_tai(first,l),l))

def test_hostile_finite_kernels_and_eop():
    l=t.LeapSecondTable();maxfloat=float.fromhex('0x1.fffffffffffffp+1023')
    eop=t.EopTable([t.EopSample(51544,0),t.EopSample(51545,0)])
    with pytest.raises(t.TimeError):eop.ut1_to_tai(maxfloat,l)
    extreme=t.LeapSecondTable([t.LeapEntry(2451545,32)]).with_constants(maxfloat,0.001657,0.01671,6.239996,1.99096871e-7)
    with pytest.raises(t.TimeError):t.compat.tt_to_tai(0,extreme)
    extreme=l.with_constants(32.184,0.001657,0.01671,6.239996,maxfloat)
    with pytest.raises(t.TimeError):t.compat.tai_to_et(t.Epoch(1),extreme)
def test_format_calendar_extreme_year_and_delta_at_distinction():
    u=t.UtcDateTime(2147483647,12,31,23,59,59.9999)
    with pytest.raises(t.TimeError):t.iso8601(u,0)
    with pytest.raises(t.TimeError):t.jd_to_calendar(1e12)
    l=t.LeapSecondTable().with_source(t.LeapSecondSource.Sofa)
    assert l.delta_at_for_jdn(t.JDN_1960) is None
    assert l.delta_at_f64(t.JDN_1960)==pytest.approx(0.943482)
def test_kernel_bytes_inputs():
    text=(ROOT/'src'/'naif0012.tls').read_bytes()
    assert t.LeapSecondTable.from_lsk(text).constants==t.LeapSecondTable().constants
    kernel=(ROOT/'tests'/'kernels'/'MRO_SCLKSCET.00120.tsc').read_bytes()
    assert t.SclkKernel.parse(kernel,-74).spacecraft_id==-74
def test_finals_parser_extreme_mjd_guard():
    line='73 1 2 41684.00 I  0.120733 0.009786  0.136966 0.015902  I 0.8084178 0.0002710'
    hostile=line[:6]+'    1e300'+line[15:]
    with pytest.raises(t.TimeError):t.EopTable.from_finals2000a(hostile)
def test_review_nonfinite_eop_legacy_and_structured_batch_errors():
    import sys
    maximum=sys.float_info.max
    l=t.LeapSecondTable()
    eop=t.EopTable([t.EopSample(51544,maximum),t.EopSample(51545,-maximum)])
    with pytest.raises(t.TimeError):eop.ut1_minus_utc(51544.5)
    epoch=t.TimeContext(l).utc_to_tai(t.UtcDateTime(2000,1,1,12))
    for fn in (lambda:eop.ut1_minus_utc_at_epoch(epoch,l),lambda:eop.tai_to_ut1(epoch,l),lambda:eop.tai_to_ut1_batch([epoch],l)):
        with pytest.raises(t.TimeError):fn()
    extreme=l.with_constants(32.184,0.001657,0.01671,6.239996,maximum)
    with pytest.raises(t.TimeError):t.compat.utc_to_et(t.UtcDateTime(2000,1,1),t.Calendar.Gregorian,extreme)
    ctx=t.TimeContext(l)
    with pytest.raises(t.ContractError) as scalar:ctx.utc_to_et(t.UtcDateTime(1900,1,1))
    with pytest.raises(t.ContractError) as batch:ctx.utc_to_et_batch([t.UtcDateTime(1900,1,1)])
    assert batch.value.code==scalar.value.code=='HistoricalUtcRequiresExplicitModel'
    assert batch.value.index==0
def _synthetic_clock(offsets=(0,)):
    count=len(offsets)
    return t.SclkKernel('KPL/SCLK\n\\begindata\nSCLK_DATA_TYPE_1=(1)\nSCLK01_TIME_SYSTEM_1=(1)\nSCLK01_N_FIELDS_1=('+str(count)+')\nSCLK01_MODULI_1=('+' '.join(['1000']*count)+')\nSCLK01_OFFSETS_1=('+' '.join(map(str,offsets))+')\nSCLK01_OUTPUT_DELIM_1=(2)\nSCLK01_COEFFICIENTS_1=(0 0 1)\n\\begintext',-1)

def test_actual_inverse_and_sclk_field_overflow_guards():
    l=t.LeapSecondTable().with_constants(32.184,1e19,0.01671,1.57,1.99096871e-7)
    with pytest.raises(t.TimeError):t.compat.et_to_tai(0,l)
    with pytest.raises(t.TimeError):_synthetic_clock().ticks_to_tai(0,l)
    for offsets in ((2**63-1,),(0,2**63-1)):
        k=_synthetic_clock(offsets)
        assert k.fields_from_ticks(0.0)==offsets
        with pytest.raises(t.TimeError):k.fields_from_ticks(1.0)
def test_active_integer_kernel_jump_overflow_guards():
    l=t.LeapSecondTable([t.LeapEntry(2451545,-2**31),t.LeapEntry(2451546,2**31-1)])
    ctx=t.TimeContext(l);u=t.UtcDateTime(2000,1,1);e=t.Epoch(0);et=ctx.to_et(e)
    calls=[lambda:ctx.tai_to_utc(e),lambda:ctx.utc_to_tai(u),lambda:ctx.utc_to_et(u),lambda:ctx.et_to_utc(et),lambda:t.compat.utc_to_et(u,t.Calendar.Gregorian,l),lambda:t.compat.et_to_utc(et.seconds,t.Calendar.Gregorian,l),lambda:ctx.utc_to_et_batch([u]),lambda:ctx.et_to_utc_batch([et]),lambda:t.iso8601_with(u,t.FormatOptions().with_leap_seconds(l))]
    eop=t.EopTable([t.EopSample(51544,0),t.EopSample(51545,0),t.EopSample(51546,0)])
    calls += [lambda:eop.tai_to_ut1(e,l),lambda:eop.ut1_minus_utc_at_epoch(e,l),lambda:eop.ut1_to_tai(0,l),lambda:eop.tai_to_ut1_batch([e],l),lambda:eop.ut1_to_tai_batch([0],l)]
    for call in calls:
        with pytest.raises(t.TimeError) as error:call()
        assert error.value.code=='Overflow'
    frozen=l.with_policy(t.LeapSecondPolicy.frozen_after(2000,1,1))
    frozen_ctx=t.TimeContext(frozen)
    assert isinstance(frozen_ctx.tai_to_utc(e),t.UtcDateTime)
    assert isinstance(frozen_ctx.utc_to_tai(u),t.Epoch)
    assert t.iso8601_with(u,t.FormatOptions().with_leap_seconds(frozen))
    first=t.calendar_to_jdn(1972,1,1);boundary=t.calendar_to_jdn(2200,1,1)
    historical=t.LeapSecondTable([t.LeapEntry(first,2**31-1),t.LeapEntry(boundary,-2**31)]).with_source(t.LeapSecondSource.Sofa)
    hc=t.TimeContext(historical);hu=t.UtcDateTime(1960,1,1)
    he=hc.utc_to_tai(hu)
    assert hc.tai_to_utc(he)==hu
    assert t.iso8601_with(hu,t.FormatOptions().with_leap_seconds(historical))
    raw=t.NaifUtcContext.from_raw_parts(l.entries,t.NaifDeltet(*l.constants))
    # Native JDUTC has a separate paired-table contract, unaffected by civil guards.
    with pytest.raises(t.ContractError) as error:raw.et_to_spice_julian_utc(t.NaifEt(0))
    assert error.value.code=='InvalidNaifLeapSeconds'
    with pytest.raises(t.ContractError) as error:ctx.et_to_spice_julian_utc(t.NaifEt(0))
    assert error.value.code=='InvalidNaifLeapSeconds'

def test_partitioned_sclk_clock_uses_native_discrete_boundary():
    kernel=t.SclkKernel.from_file(ROOT/'tests'/'kernels'/'synthetic_partition_reset.tsc',-999)
    clock=t.SclkClock(kernel,t.LeapSecondTable.builtin())
    assert clock.et_to_sclk(t.NaifEt(-0.00008)) == '1/001:00:001'
    assert clock.et_to_sclk(t.NaifEt(3.0000816666666665)) == '3/002:00:001'
    assert clock.sclk_to_et('1/001:00:001').seconds == 0.0

def test_partitioned_sclk_native_parser_context():
    kernel=t.SclkKernel.from_file(ROOT/'tests'/'kernels'/'MRO_SCLKSCET.00120.tsc',-74)
    context=t.SclkStringContext()
    assert kernel.decode_native_sclk(0.0) == '1/0000000000.000'
    assert kernel.encode_native_sclk_with_context('1/0.0',context) == 0.0
    with pytest.raises(t.SclkStringError) as bad:
        kernel.encode_native_sclk_with_context('1/UNKNOWN',context)
    assert bad.value.spice_short() == 'SPICE(INVALIDSCLKSTRING)'
    with pytest.raises(t.SclkStringError):
        kernel.encode_native_sclk_with_context('1/.1',context)
    assert kernel.encode_native_sclk_with_context('1/0.0',context) == 0.0
    assert kernel.encode_native_sclk_with_context('1/.1',context) == 1.0

def test_partitioned_sclk_python_matches_native_large_fields():
    path=ROOT/'tests'/'reference'/'sclk-partition'
    kernel=t.SclkKernel.from_file(path/'large-field.tsc',-902)
    with (path/'large-field.tsv').open(encoding='utf-8',newline='') as f:
        rows=list(csv.DictReader(f,delimiter='\t'))
    assert len(rows)==6008
    for row in rows:
        ticks=struct.unpack('>d',bytes.fromhex(row['ticks_bits']))[0]
        assert 'S:'+kernel.decode_native_sclk(ticks)==row['result']

def test_partitioned_sclk_python_matches_native_extreme_buffer():
    path=ROOT/'tests'/'reference'/'sclk-partition'
    kernel=t.SclkKernel.from_file(path/'extreme-field.tsc',-902)
    with (path/'extreme-field.tsv').open(encoding='utf-8',newline='') as f:
        rows=list(csv.DictReader(f,delimiter='\t'))
    assert len(rows)==16
    for row in rows:
        ticks=struct.unpack('>d',bytes.fromhex(row['ticks_bits']))[0]
        assert 'S:'+kernel.decode_native_sclk(ticks)==row['result']

def test_partitioned_sclk_python_native_metadata_errors():
    path=ROOT/'tests'/'reference'/'sclk-partition'
    kernel=t.SclkKernel.from_file(path/'eleven-fields.tsc',-902)
    for convert in (lambda:kernel.decode_native_sclk(0.0),lambda:kernel.encode_native_sclk('1/0')):
        with pytest.raises(t.SclkStringError) as error:
            convert()
        assert error.value.spice_short()=='SPICE(VALUEOUTOFRANGE)'
    text=(path/'large-field.tsc').read_text(encoding='utf-8').replace('SCLK01_OUTPUT_DELIM_902=(1)','')
    missing=t.SclkKernel(text,-902)
    with pytest.raises(t.SclkStringError) as error:
        missing.decode_native_sclk(0.0)
    assert error.value.spice_short()=='SPICE(KERNELVARNOTFOUND)'

def test_partitioned_sclk_python_native_width_boundaries():
    path=ROOT/'tests'/'reference'/'sclk-partition'
    source=(path/'large-field.tsc').read_text(encoding='utf-8')
    with (path/'width-boundaries.tsv').open(encoding='utf-8',newline='') as f:
        rows=list(csv.DictReader(f,delimiter='\t'))
    assert len(rows)==80
    for row in rows:
        text=source.replace('SCLK01_MODULI_902=(10000000000000000)',f"SCLK01_MODULI_902=({row['modulus']})")
        assert 'S:'+t.SclkKernel(text,-902).decode_native_sclk(0.0)==row['result']

def test_wgc_gps_week_tow_matches_live_reference():
    lsk=t.LeapSecondTable.builtin()
    base=ROOT/'tests'/'reference'/'wgc-gps'
    counts={'week-tow-cases.tsv':354,'week-tow-edge-cases.tsv':38,
            'decimal-ties.tsv':6,'decimal-adjacent.tsv':8,'seeded-half-grid.tsv':400,
            'independent-half-probe.tsv':480,'nonfinite-inputs.tsv':3}
    for name,count in counts.items():
        with (base/name).open(encoding='utf-8',newline='') as f:
            rows=list(csv.DictReader(f,delimiter='\t'))
        assert len(rows)==count
        for row in rows:
            assert t.gps_seconds_to_week_tow(float(row['input_gps_seconds']),lsk)==row['week_tow']
    for value in (float('nan'),float('inf'),float('-inf')):
        assert t.gps_seconds_to_week_tow(value,lsk)=='0, NaN'

def test_wgc_gps_python_rejects_silent_integer_rounding():
    lsk=t.LeapSecondTable.builtin()
    with pytest.raises(OverflowError):
        t.gps_seconds_to_week_tow(2**53+1,lsk)
    with pytest.raises(TypeError):
        t.gps_seconds_to_week_tow(True,lsk)

def test_wgc_week_tow_input_matches_nasa():
    lsk=t.LeapSecondTable.builtin()
    with (ROOT/'tests'/'reference'/'wgc-gps-input'/'seeded-week-tow.tsv').open(encoding='utf-8',newline='') as f:
        rows=list(csv.DictReader(f,delimiter='\t'))
    assert len(rows)==200
    for row in rows:
        seconds=t.parse_wgc_week_tow(row['input'])
        assert t.gps_seconds_to_week_tow(seconds,lsk)==row['week_tow']
    assert t.parse_wgc_week_tow('1, 0x1.0p2')==604804.0
    with pytest.raises(t.WgcGpsError): t.parse_wgc_week_tow('1,2')

def test_wgc_week_unicode_digits_match_nasa():
    path=ROOT/'tests'/'reference'/'wgc-gps-input'/'grammar-unicode.tsv'
    with path.open(encoding='ascii',newline='') as f: rows=list(csv.DictReader(f,delimiter='\t'))
    assert len(rows)==413
    for row in rows:
        text=bytes.fromhex(row['input_utf8_hex']).decode('utf8')
        if row['accepted']=='1':
            assert struct.pack('>d',t.parse_wgc_week_tow(text)).hex()==row['output_bits']
        else:
            with pytest.raises(t.WgcGpsError):t.parse_wgc_week_tow(text)

def test_native_sclk_clock_preserves_spice_errors_and_et_corpus():
    kernel=t.SclkKernel.from_file(ROOT/'tests'/'kernels'/'MRO_SCLKSCET.00120.tsc',-74)
    clock=t.SclkClock(kernel,t.LeapSecondTable.builtin())
    with (ROOT/'tests'/'reference'/'sclk-partition'/'et.tsv').open(encoding='utf-8',newline='') as f:
        rows=list(csv.DictReader(f,delimiter='\t'))
    assert len(rows)==94
    for row in rows:
        expected=struct.unpack('>d',bytes.fromhex(row['et_or_error']))[0]
        assert struct.pack('>d',clock.native_sclk_to_et(row['input']).seconds)==struct.pack('>d',expected)
        assert clock.native_et_to_sclk(t.NaifEt(expected))==row['decoded']
    with pytest.raises(t.SclkStringError) as error: clock.native_sclk_to_et('PI/0')
    assert error.value.spice_short()=='SPICE(NOTINPART)'
    reset=t.SclkKernel.from_file(ROOT/'tests'/'kernels'/'synthetic_partition_reset.tsc',-999)
    reset_clock=t.SclkClock(reset,t.LeapSecondTable.builtin())
    below=struct.unpack('>d',bytes.fromhex('bf1a36e2eb1c432d'))[0]
    with pytest.raises(t.SclkStringError) as error: reset_clock.native_et_to_sclk(t.NaifEt(below))
    assert error.value.spice_short()=='SPICE(VALUEOUTOFRANGE)'

def test_native_sclk_clock_cspice_et_error_grid():
    lsk=t.LeapSecondTable.builtin()
    kernels={
        'mro':t.SclkKernel.from_file(ROOT/'tests'/'kernels'/'MRO_SCLKSCET.00120.tsc',-74),
        'reset':t.SclkKernel.from_file(ROOT/'tests'/'kernels'/'synthetic_partition_reset.tsc',-999),
        'coeff':t.SclkKernel.from_file(ROOT/'tests'/'reference'/'sclk-partition'/'coefficient-range.tsc',-999),
    }
    path=ROOT/'tests'/'reference'/'sclk-partition'/'et-error-grid.tsv'
    with path.open(encoding='utf-8',newline='') as f: rows=list(csv.DictReader(f,delimiter='\t'))
    assert len(rows)==660
    for row in rows:
        et=struct.unpack('>d',bytes.fromhex(row['et_bits']))[0]
        clock=t.SclkClock(kernels[row['kernel']],lsk)
        if row['result'].startswith('E:'):
            with pytest.raises(t.SclkStringError) as error: clock.native_et_to_sclk(t.NaifEt(et))
            assert error.value.spice_short()==row['result'][2:]
        else:
            assert 'S:'+clock.native_et_to_sclk(t.NaifEt(et))==row['result']
    offset=t.SclkKernel.from_file(ROOT/'tests'/'reference'/'sclk-partition'/'coefficient-range.tsc',-999)
    with pytest.raises(t.SclkStringError) as error: t.SclkClock(offset,t.LeapSecondTable.builtin()).native_sclk_to_et('1/001:00:001')
    assert error.value.spice_short()=='SPICE(VALUEOUTOFRANGE)'
    tiny=t.SclkKernel.from_file(ROOT/'tests'/'reference'/'sclk-partition'/'tiny-rate.tsc',-999)
    for et in (-1.0,1.0):
        with pytest.raises(t.SclkStringError) as error:t.SclkClock(tiny,lsk).native_et_to_sclk(t.NaifEt(et))
        assert error.value.spice_short()=='SPICE(VALUEOUTOFRANGE)'

def test_native_sclk_finite_et_nan_intermediate_two_native_builds():
    base=ROOT/'tests'/'reference'/'sclk-partition'/'nan-finite-et'
    record=json.loads((base/'capture.json').read_text(encoding='utf8'))
    assert len(record['rows'])==12
    lsk=t.LeapSecondTable.builtin()
    for row in record['rows']:
        clock=t.SclkClock(t.SclkKernel.from_file(base/(row['name']+'.tsc'),-777),lsk)
        if row['outcome']=='ERROR':
            with pytest.raises(t.SclkStringError) as error:clock.native_et_to_sclk(t.NaifEt(0.0))
            assert error.value.spice_short()==row['windows_result']
        else:assert clock.native_et_to_sclk(t.NaifEt(0.0))==row['windows_result']

def test_native_sclk_nonfinite_output_is_rejected_by_checked_wrapper():
    path=ROOT/'tests'/'reference'/'sclk-partition'/'overflow-rate.tsc'
    clock=t.SclkClock(t.SclkKernel.from_file(path,-777),t.LeapSecondTable.builtin())
    assert struct.pack('>d',clock.native_sclk_to_et('1/001').seconds).hex()=='7fe1ccf385ebc89f'
    with pytest.raises(t.TimeError) as error:
        clock.native_sclk_to_et('1/002')
    assert error.value.code=='NonFinite'

def test_wgc_calendar_doy_all_systems_against_nasa():
    lsk=t.LeapSecondTable.builtin()
    for system in ('UTC','TDB','TT','GPS'):
        for suffix in ('seeded','epochs','leaps'):
            path=ROOT/'tests'/'reference'/'wgc-gps-input'/f'{system.lower()}-doy-{suffix}.tsv'
            with path.open(encoding='utf-8',newline='') as f: rows=list(csv.DictReader(f,delimiter='\t'))
            assert len(rows) in (240,261,336)
            for row in rows:
                expected=row.get('output',row.get('utc_doy'))
                assert t.et_to_wgc_calendar_doy(t.NaifEt(float(row['et'])),system,lsk)==expected

def test_wgc_calendar_doy_matches_cspice_binary64():
    lsk=t.LeapSecondTable.builtin()
    path=ROOT/'tests'/'reference'/'wgc-gps-input'/'cspice-doy-seeded.tsv'
    with path.open(encoding='utf-8',newline='') as f: rows=list(csv.DictReader(f,delimiter='\t'))
    assert len(rows)==12000
    for row in rows:
        et=struct.unpack('>d',bytes.fromhex(row['et_bits']))[0]
        assert t.et_to_wgc_calendar_doy(t.NaifEt(et),row['system'],lsk)==row['output']

def test_wgc_calendar_formats_match_cspice_altered_lsk():
    base=ROOT/'tests'/'reference'/'wgc-gps-input'
    lsk=t.LeapSecondTable.from_file(base/'altered-lsk.tls')
    with (base/'altered-lsk-doy-ymd.tsv').open(encoding='utf-8',newline='') as f: rows=list(csv.DictReader(f,delimiter='\t'))
    assert len(rows)==3200
    for row in rows:
        et=t.NaifEt(struct.unpack('>d',bytes.fromhex(row['et_bits']))[0])
        render=t.et_to_wgc_calendar_doy if row['style']=='DOY' else t.et_to_wgc_calendar_ymd
        assert render(et,row['system'],lsk)==row['output']

def test_wgc_calendar_wide_epochs_match_native_components_and_year():
    lsk=t.LeapSecondTable.builtin()
    path=ROOT/'tests'/'reference'/'wgc-gps-input'/'review-wide.tsv'
    with path.open(encoding='utf8',newline='') as f:rows=list(csv.DictReader(f,delimiter='\t'))
    assert len(rows)==20490
    for row in rows:
        et=t.NaifEt(struct.unpack('>d',bytes.fromhex(row['et_bits']))[0])
        render=t.et_to_wgc_calendar_doy if row['style']=='DOY' else t.et_to_wgc_calendar_ymd
        year,suffix=render(et,row['system'],lsk).split('-',1)
        actual=1-int(year[:-3]) if year.endswith(' BC') else int(year.removesuffix(' AD'))
        assert actual==int(row['year'])
        assert suffix==row['native_suffix']

def test_wgc_calendar_with_altered_leap_tables_matches_native():
    base=(ROOT/'src'/'naif0012.tls').read_text(encoding='utf8')
    offsets=[35,36,38,40,0,-100]
    lsks={n:t.LeapSecondTable.from_lsk(base.replace('37,   @2017-JAN-1',f'{n}, @2017-JAN-1')) for n in offsets}
    path=ROOT/'tests'/'reference'/'wgc-gps-input'/'review-altered-leaps.tsv'
    with path.open(encoding='utf8',newline='') as f:rows=list(csv.DictReader(f,delimiter='\t'))
    assert len(rows)==49248
    for row in rows:
        et=t.NaifEt(struct.unpack('>d',bytes.fromhex(row['et_bits']))[0])
        render=t.et_to_wgc_calendar_doy if row['style']=='DOY' else t.et_to_wgc_calendar_ymd
        assert render(et,'UTC',lsks[int(row['offset'])])==row['output']

def test_wgc_week_to_et_and_numeric_gps_seconds():
    lsk=t.LeapSecondTable.builtin()
    path=ROOT/'tests'/'reference'/'wgc-gps-input'/'week-seconds-seeded.tsv'
    with path.open(encoding='utf-8',newline='') as f: rows=list(csv.DictReader(f,delimiter='\t'))
    assert len(rows)==200
    for row in rows:
        et=t.wgc_week_tow_to_et(row['input'],lsk)
        assert struct.pack('>d',t.et_to_wgc_gps_display_seconds(et,lsk)).hex()==row['output_bits']
    with pytest.raises(t.WgcGpsError): t.wgc_week_tow_to_et('0, NaN',lsk)

def test_wgc_calendar_ymd_matches_nasa():
    lsk=t.LeapSecondTable.builtin()
    for system in ('UTC','TDB','TT','GPS'):
        path=ROOT/'tests'/'reference'/'wgc-gps-input'/f'{system.lower()}-ymd-epochs.tsv'
        with path.open(encoding='utf-8',newline='') as f: rows=list(csv.DictReader(f,delimiter='\t'))
        assert len(rows)==261
        for row in rows:
            assert t.et_to_wgc_calendar_ymd(t.NaifEt(float(row['et'])),system,lsk)==row['output']

def test_wgc_gps_seconds_input_matches_nasa_numeric_display():
    lsk=t.LeapSecondTable.builtin()
    for name,count in [('gps-seconds-numeric.tsv',330),('gps-numeric-rounding-grid.tsv',1200),('gps-signed-zero.tsv',10)]:
        path=ROOT/'tests'/'reference'/'wgc-gps-input'/name
        with path.open(encoding='utf-8',newline='') as f: rows=list(csv.DictReader(f,delimiter='\t'))
        assert len(rows)==count
        for row in rows:
            et=t.wgc_gps_seconds_to_et(float(row['input']),lsk)
            assert struct.pack('>d',t.et_to_wgc_gps_display_seconds(et,lsk)).hex()==row['output_bits']
    with pytest.raises(t.WgcGpsError): t.wgc_gps_seconds_to_et(float('inf'),lsk)
    with pytest.raises(TypeError): t.wgc_gps_seconds_to_et(True,lsk)
