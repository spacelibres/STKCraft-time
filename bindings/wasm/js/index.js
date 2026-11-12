import initWasm, * as raw from '../pkg/stkcraft_time.js';
let initialized = false;
export async function init(input = new URL('../pkg/stkcraft_time_bg.wasm', import.meta.url)) {
  if (input instanceof URL || typeof input === 'string') input = await (await fetch(input)).arrayBuffer();
  await initWasm({module_or_path: input}); initialized = true;
}
function ready() { if (!initialized) fail('NotInitialized', 'Call and await init first'); }
function fail(code, message) { const e = new Error(message); e.name='TimeError'; e.code=code; throw e; }
function finite(v) { if (typeof v !== 'number' || !Number.isFinite(v)) fail('NonFinite', 'Expected finite number'); return v; }
function integer(v,min,max) { finite(v); if (!Number.isInteger(v) || v<min || v>max) fail('Range','Integer outside range'); return v; }
function bigint(v) { if (typeof v !== 'bigint') fail('Type','Expected bigint'); return v; }
function instance(v,c) { if (!(v instanceof c)) fail('Type',`Expected ${c.name}`); return v; }
const wrap = (C,r) => { const o=Object.create(C.prototype); handles.set(o,r); return Object.freeze(o); };
const handles=new WeakMap();
function h(o) { const r=handles.get(o); if (!r) fail('Disposed','Object has been disposed'); return r; }
class Owned { free(){const r=h(this);r.free();handles.delete(this);} [Symbol.dispose](){this.free();} }
export const Calendar=Object.freeze({Gregorian:'Gregorian',Julian:'Julian',Mixed:'Mixed'});
export function version(){ready();return raw.version();}
export class Epoch extends Owned {
 constructor(seconds,nanoseconds=0n){super();ready();handles.set(this,new raw.RawEpoch(bigint(seconds),bigint(nanoseconds)));Object.freeze(this);}
 static fromTaiSeconds(v){ready();return wrap(Epoch,raw.RawEpoch.from_float(finite(v)));}
 get seconds(){return h(this).seconds();} get nanoseconds(){return h(this).nanos();}
 add(d){return wrap(Epoch,h(this).add(h(instance(d,Duration)),false));}
 subtract(d){return wrap(Epoch,h(this).add(h(instance(d,Duration)),true));}
 difference(e){return wrap(Duration,h(this).difference(h(instance(e,Epoch))));}
 compare(e){return h(this).compare(h(instance(e,Epoch)));}
 toTaiSeconds(){return h(this).approximate_seconds();}
 toJSON(){return {schema:'stkcraft-time/1',type:'Epoch',scale:'TAI',origin:'2000-01-01T00:00:00 TAI',units:'seconds+nanoseconds',seconds:String(this.seconds),nanoseconds:this.nanoseconds};}
 static fromJSON(o){if(o?.schema!=='stkcraft-time/1'||o.type!=='Epoch'||o.scale!=='TAI'||o.origin!=='2000-01-01T00:00:00 TAI'||o.units!=='seconds+nanoseconds'||typeof o.seconds!=='string'||!/^[-]?\d+$/.test(o.seconds))fail('Schema','Invalid exact Epoch schema');return new Epoch(BigInt(o.seconds),BigInt(integer(o.nanoseconds,0,999999999)));}
}
export class Duration extends Owned {
 constructor(totalNanoseconds){super();ready();handles.set(this,new raw.RawDuration(bigint(totalNanoseconds)));Object.freeze(this);}
 static fromSeconds(v){ready();return wrap(Duration,raw.RawDuration.from_float(finite(v)));}
 get totalNanoseconds(){return h(this).total();} get seconds(){return h(this).seconds();} get nanoseconds(){return h(this).nanos();}
 add(d){return wrap(Duration,h(this).calculate('add',h(instance(d,Duration))));}
 subtract(d){return wrap(Duration,h(this).calculate('subtract',h(instance(d,Duration))));}
 multiply(k){return wrap(Duration,h(this).multiply(bigint(k)));}
 negate(){return wrap(Duration,h(this).negate());}
 abs(){return this.totalNanoseconds<0n?this.negate():new Duration(this.totalNanoseconds);}
 floor(d){return wrap(Duration,h(this).calculate('floor',h(instance(d,Duration))));}
 ceil(d){return wrap(Duration,h(this).calculate('ceil',h(instance(d,Duration))));}
 round(d){return wrap(Duration,h(this).calculate('round',h(instance(d,Duration))));}
 toSeconds(){return h(this).approximate_seconds();}
 toJSON(){return {schema:'stkcraft-time/1',type:'Duration',units:'nanoseconds',nanoseconds:String(this.totalNanoseconds)};}
 static fromJSON(o){if(o?.schema!=='stkcraft-time/1'||o.type!=='Duration'||o.units!=='nanoseconds'||typeof o.nanoseconds!=='string'||!/^[-]?\d+$/.test(o.nanoseconds))fail('Schema','Invalid Duration schema');return new Duration(BigInt(o.nanoseconds));}
}
export class UtcDateTime {
 constructor(year,month,day,hour=0,minute=0,second=0){this.year=integer(year,-2147483648,2147483647);this.month=integer(month,1,12);this.day=integer(day,1,31);this.hour=integer(hour,0,23);this.minute=integer(minute,0,59);this.second=finite(second);if(second<0||second>=61)fail('InvalidSecond','Second outside 0..61');Object.freeze(this);}
 static fromDate(date){instance(date,Date);if(!Number.isFinite(date.getTime()))fail('InvalidDate','Invalid JS Date');return new UtcDateTime(date.getUTCFullYear(),date.getUTCMonth()+1,date.getUTCDate(),date.getUTCHours(),date.getUTCMinutes(),(date.getUTCSeconds()*1000+date.getUTCMilliseconds())/1000);}
 toDate(loss='reject'){calendar.toJdn(this.year,this.month,this.day);if(!['reject','truncate'].includes(loss))fail('Policy','Unknown loss policy');if(this.second>=60)fail('InvalidLeapSecond','Date cannot represent leap labels');const nearest=Math.round(this.second*1000);const exactMs=this.second===nearest/1000;if(loss==='reject'&&!exactMs)fail('PrecisionLoss','Date has millisecond resolution');const d=new Date(0);d.setUTCFullYear(this.year,this.month-1,this.day);d.setUTCHours(this.hour,this.minute,Math.floor((exactMs?nearest:Math.trunc(this.second*1000))/1000),(exactMs?nearest:Math.trunc(this.second*1000))%1000);if(!Number.isFinite(d.getTime()))fail('Range','Date outside range');return d;}
 _array(){return [this.year,this.month,this.day,this.hour,this.minute,this.second];}
}
const token=Symbol();
export class NaifEt { constructor(seconds){this.seconds=finite(seconds);this.profile='NaifSpice';Object.freeze(this);} }
export class NaifJulianUtc { constructor(days,key){if(key!==token)fail('Type','Construct via TimeContext');this.days=days;Object.freeze(this);} }
export class IauTdb { constructor(seconds,model,key){if(key!==token)fail('Type','Construct via ScientificContext');this.seconds=seconds;this.model=model;this.profile='IauScientific';Object.freeze(this);} }
export class TimeContext extends Owned {
 constructor(lskText){super();ready();if(lskText instanceof Uint8Array)lskText=new TextDecoder('utf-8',{fatal:true}).decode(lskText);if(lskText!==undefined&&typeof lskText!=='string')fail('Type','LSK must be text');handles.set(this,new raw.RawContext(lskText));Object.freeze(this);}
 select({source='IersOnly',frozenAfterJdn=null}={}){if(frozenAfterJdn!==null)bigint(frozenAfterJdn);return wrap(TimeContext,h(this).select(source,frozenAfterJdn));}
 get metadata(){return JSON.parse(h(this).metadata());}
 utcToEpoch(utc,calendar='Gregorian'){return wrap(Epoch,h(this).utc_to_epoch(instance(utc,UtcDateTime)._array(),calendar));}
 epochToUtc(epoch,calendar='Gregorian'){return new UtcDateTime(...h(this).epoch_to_utc(h(instance(epoch,Epoch)),calendar));}
 utcToEt(utc,calendar='Gregorian'){return new NaifEt(h(this).utc_to_et(instance(utc,UtcDateTime)._array(),calendar));}
 etToUtc(et,calendar='Gregorian'){return new UtcDateTime(...h(this).et_to_utc(instance(et,NaifEt).seconds,calendar));}
 toEt(epoch){return new NaifEt(h(this).to_number(h(instance(epoch,Epoch)),'ET'));}
 toEpoch(et){return wrap(Epoch,h(this).from_number(instance(et,NaifEt).seconds,'ET'));}
 etToSpiceJulianUtc(et){return new NaifJulianUtc(h(this).julian_utc(instance(et,NaifEt).seconds),token);}
 toScale(epoch,scale){return h(this).to_number(h(instance(epoch,Epoch)),scale);}
 fromScale(seconds,scale){return wrap(Epoch,h(this).from_number(finite(seconds),scale));}
 format(utc,{calendar='Gregorian',style='iso8601',precision=3,pattern=''}={}){return h(this).format(instance(utc,UtcDateTime)._array(),calendar,style,integer(precision,0,9),pattern);}
 utcToEtBatch(values,calendar='Gregorian'){return h(this).utc_batch(Array.from(values,v=>instance(v,UtcDateTime)._array()),calendar);}
 etToUtcBatch(values,calendar='Gregorian'){return Array.from(h(this).et_batch(Float64Array.from(Array.from(values,finite)),calendar),a=>new UtcDateTime(...a));}
}
export class ScientificContext {
 constructor(){ready();this.context=new TimeContext();Object.freeze(this);}
 ttSeconds(epoch){return this.context.toScale(epoch,'TT.IAU');}
 toTdb(epoch){return new IauTdb(this.context.toScale(epoch,'TDB.IAU'),'GeocentricFairheadBretagnon',token);}
 toTdbAtObserver(epoch,{ut1Fraction,longitude,uKm,vKm}){return new IauTdb(h(this.context).observer(h(instance(epoch,Epoch)),[ut1Fraction,longitude,uKm,vKm].map(finite)),'ObserverFairheadBretagnon',token);}
 free(){this.context.free();}
}
export class SclkKernel extends Owned {
 constructor(text,spacecraftId){super();ready();if(text instanceof Uint8Array)text=new TextDecoder('utf-8',{fatal:true}).decode(text);handles.set(this,new raw.RawSclk(text,integer(spacecraftId,-2147483648,2147483647)));Object.freeze(this);}
 get metadata(){const m=JSON.parse(h(this).metadata());m.moduli=m.moduli.map(BigInt);m.offsets=m.offsets.map(BigInt);return m;}
 ticksToEt(ticks,context){return new NaifEt(h(this).convert(h(instance(context,TimeContext)),finite(ticks),false));}
 etToTicks(et,context){return h(this).convert(h(instance(context,TimeContext)),instance(et,NaifEt).seconds,true);}
 ticksToEpoch(ticks,context){return wrap(Epoch,h(this).to_epoch(h(instance(context,TimeContext)),finite(ticks)));}
 epochToTicks(epoch,context){return h(this).from_epoch(h(instance(context,TimeContext)),h(instance(epoch,Epoch)));}
 fieldsFromTicks(ticks){return h(this).fields(finite(ticks));}
 ticksFromFields(fields){return h(this).ticks(Array.from(fields,bigint));}
 parseString(text){return h(this).parse(text);}
 formatString(fields){return h(this).format(Array.from(fields,bigint));}
 parallelSeconds(ticks){return h(this).parallel(finite(ticks),false);}
 ticksFromParallel(seconds){return h(this).parallel(finite(seconds),true);}
 ticksToEtBatch(values,context){return h(this).batch(h(instance(context,TimeContext)),Float64Array.from(Array.from(values,finite)),false);}
}
export class EopTable extends Owned {
 constructor(text,series='FinalPreferred'){super();ready();if(text instanceof Uint8Array)text=new TextDecoder('utf-8',{fatal:true}).decode(text);handles.set(this,new raw.RawEop(text,series));Object.freeze(this);}
 observedOnly(){return wrap(EopTable,h(this).observed());}
 get metadata(){return JSON.parse(h(this).metadata());}
 interpolate(mjd){return h(this).interpolate(finite(mjd));}
 toUt1(epoch,context){return h(this).to_ut1(h(instance(context,TimeContext)),h(instance(epoch,Epoch)),false);}
 ut1MinusUtc(epoch,context){return h(this).to_ut1(h(instance(context,TimeContext)),h(instance(epoch,Epoch)),true);}
 fromUt1(seconds,context){return wrap(Epoch,h(this).from_ut1(h(instance(context,TimeContext)),finite(seconds)));}
}
export function parseKernelDate(text){ready();return JSON.parse(raw.kernel_date(text));}
export class TimeSeries {
 constructor(start,end,step,{inclusive=true}={}){instance(start,Epoch);instance(end,Epoch);instance(step,Duration);this.start=start;this.end=end;this.step=step;if(typeof inclusive!=='boolean')fail('Type','inclusive must be boolean');this.inclusive=inclusive;Object.freeze(this);}
 *[Symbol.iterator](){const it=new raw.RawSeries(h(this.start),h(this.end),h(this.step),this.inclusive);try{for(;;){const n=it.next_epoch();if(n===undefined)break;yield wrap(Epoch,n);}}finally{it.free();}}
}
export class Bdt extends Owned {
 constructor(seconds,nanoseconds=0n){super();ready();handles.set(this,new raw.RawBdt(bigint(seconds),bigint(nanoseconds)));Object.freeze(this);}
 static fromEpoch(epoch){return wrap(Bdt,raw.RawBdt.from_epoch(h(instance(epoch,Epoch))));}
 static fromSeconds(seconds){ready();return wrap(Bdt,raw.RawBdt.from_float(finite(seconds)));}
 static fromNanoseconds(ns){ready();return wrap(Bdt,raw.RawBdt.from_total(bigint(ns)));}
 static fromWeekNanoseconds(week,nanoseconds){ready();return wrap(Bdt,raw.RawBdt.from_week(bigint(week),bigint(nanoseconds)));}
 get seconds(){return h(this).seconds();} get nanoseconds(){return h(this).nanos();} get totalNanoseconds(){return h(this).total();}
 toSeconds(){return h(this).approximate_seconds();} toEpoch(){return wrap(Epoch,h(this).to_epoch());} weekNanoseconds(){return h(this).week();}
 static fromUtc(utc,context,calendar='Gregorian'){instance(context,TimeContext);const e=context.utcToEpoch(utc,calendar);try{return Bdt.fromEpoch(e);}finally{e.free();}}
 toUtc(context,calendar='Gregorian'){instance(context,TimeContext);const e=this.toEpoch();try{return context.epochToUtc(e,calendar);}finally{e.free();}}
 static fromEt(et,context){instance(context,TimeContext);const e=context.toEpoch(et);try{return Bdt.fromEpoch(e);}finally{e.free();}}
 toEt(context){instance(context,TimeContext);const e=this.toEpoch();try{return context.toEt(e);}finally{e.free();}}
 toJSON(){return {schema:'stkcraft-time/1',type:'Bdt',scale:'BDT',origin:'2006-01-01T00:00:00 UTC',units:'seconds+nanoseconds',seconds:String(this.seconds),nanoseconds:this.nanoseconds};}
 static fromJSON(o){if(o?.schema!=='stkcraft-time/1'||o.type!=='Bdt'||o.scale!=='BDT'||o.origin!=='2006-01-01T00:00:00 UTC'||o.units!=='seconds+nanoseconds'||typeof o.seconds!=='string'||!/^[-]?\d+$/.test(o.seconds))fail('Schema','Invalid BDT schema');return new Bdt(BigInt(o.seconds),BigInt(integer(o.nanoseconds,0,999999999)));}
}
export const calendar=Object.freeze({
 isGregorianLeap:year=>{ready();return raw.calendar_call('isGregorianLeap',[year],'Gregorian');},
 isJulianLeap:year=>{ready();return raw.calendar_call('isJulianLeap',[year],'Julian');},
 daysInMonth:(year,month,cal='Gregorian')=>{ready();return raw.calendar_call('daysInMonth',[year,month],cal);},
 toJdn:(year,month,day,cal='Gregorian')=>{ready();return raw.calendar_call('toJdn',[year,month,day],cal);},
 fromJdn:(jdn,cal='Gregorian')=>{ready();return raw.calendar_call('fromJdn',[bigint(jdn)],cal);},
 toJd:(utc,cal='Gregorian')=>{ready();return raw.calendar_call('toJd',instance(utc,UtcDateTime)._array(),cal);},
 jdToMjd:jd=>{ready();return raw.calendar_call('jdToMjd',[finite(jd)],'Gregorian');},
 mjdToJd:mjd=>{ready();return raw.calendar_call('mjdToJd',[finite(mjd)],'Gregorian');}
});
export const models=Object.freeze(Object.fromEntries(['compactMinusTt','dtdbMinusTt','dtdbMinusTtSplit','dtdbAtObserver','tcbMinusTdb','sofaDeltaAt'].map(op=>[op,(...args)=>{ready();return raw.model_call(op,args.map(finite));}])));
TimeContext.fromEntries=function(entries,deltet){ready();return wrap(TimeContext,raw.RawContext.from_entries(Array.from(entries,([jdn,offset])=>[bigint(jdn),integer(offset,-2147483648,2147483647)]),Array.from(deltet,finite)));};
TimeContext.prototype.deltaAt=function(jdn,dayFraction=0){return h(this).delta_at(bigint(jdn),finite(dayFraction));};
EopTable.fromSamples=function(samples){ready();return wrap(EopTable,raw.RawEop.from_samples(Array.from(samples,s=>[finite(s.mjd),finite(s.ut1MinusUtc),s.observed])));};
EopTable.prototype.fromUt1Batch=function(values,context){return Array.from(h(this).batch(h(instance(context,TimeContext)),Float64Array.from(Array.from(values,finite))),r=>wrap(Epoch,r));};
SclkKernel.prototype.etToTicksBatch=function(values,context){return h(this).batch(h(instance(context,TimeContext)),Float64Array.from(Array.from(values,finite)),true);};
export class NaifUtcContext {
 constructor(entries,deltet){ready();this.entries=Object.freeze(Array.from(entries,([j,o])=>Object.freeze([bigint(j),integer(o,-2147483648,2147483647)])));this.deltet=Object.freeze(Array.from(deltet,finite));Object.freeze(this);}
 etToSpiceJulianUtc(et){return new NaifJulianUtc(raw.raw_naif_julian(this.entries,this.deltet,instance(et,NaifEt).seconds),token);}
}
export const constants=Object.freeze({J2000_JDN:2451545n,JDN_1960:2436935n,JDN_1972:2441318n,GPS_EPOCH_JDN:2444245n,GREGORIAN_SWITCH_JDN:2299161n,DELTA_T_A:32.184,K:0.001657,EB:0.01671,M0:6.239996,M1:1.99096871e-7,L_G:6.969290134e-10,L_B:1.550519768e-8,TDB0:-6.55e-5,TAI_MINUS_BDT_SECONDS:33n,GPS_MINUS_BDT_SECONDS:14n,BDT_ORIGIN_TAI_SECONDS:189388833n,NANOSECONDS_PER_WEEK:604800000000000n});
export const compatibility=Object.freeze({
 utcToEt:(utc,context,calendar='Gregorian')=>h(instance(context,TimeContext)).legacy_utc_et(instance(utc,UtcDateTime)._array(),calendar),
 etToUtc:(seconds,context,calendar='Gregorian')=>new UtcDateTime(...h(instance(context,TimeContext)).legacy_et_utc(finite(seconds),calendar)),
 taiToEt:(epoch,context)=>h(instance(context,TimeContext)).legacy_to_et(h(instance(epoch,Epoch))),
 etToTai:(seconds,context)=>wrap(Epoch,h(instance(context,TimeContext)).legacy_from_et(finite(seconds))),
 toScale:(epoch,scale,context)=>scale==='ET'?h(instance(context,TimeContext)).legacy_to_et(h(instance(epoch,Epoch))):instance(context,TimeContext).toScale(epoch,scale),
 fromScale:(seconds,scale,context)=>scale==='ET'?wrap(Epoch,h(instance(context,TimeContext)).legacy_from_et(finite(seconds))):instance(context,TimeContext).fromScale(seconds,scale)
});
Duration.fromParts=function(seconds,nanoseconds=0n){ready();return wrap(Duration,raw.RawDuration.from_parts(bigint(seconds),bigint(nanoseconds)));};
Duration.prototype.divide=function(k){return wrap(Duration,h(this).divide(bigint(k)));};
export const formatting=Object.freeze({etcal:(seconds,precision=3)=>{ready();return raw.format_numeric('etcal',finite(seconds),integer(precision,0,9));},jdToCalendar:jd=>{ready();return new UtcDateTime(...raw.format_numeric('jdToCalendar',finite(jd),0));}});
export class TimeError extends Error {
 constructor(code,message){super(message);this.name='TimeError';this.code=code;}
 static [Symbol.hasInstance](e){return e instanceof Error&&e.name==='TimeError'&&typeof e.code==='string';}
}
export class ContractError extends TimeError {
 static [Symbol.hasInstance](e){return e instanceof TimeError&&['NonFinite','EpochOutOfRange','HistoricalUtcRequiresExplicitModel','NaifKernelModelRequired','InvalidNaifLeapSeconds'].includes(e.code);}
}
export class DtdbError extends TimeError {
 static [Symbol.hasInstance](e){return e instanceof TimeError&&['NonFiniteInput','InvalidUt1Fraction','InvalidObserverDistance','OutsideModelRange'].includes(e.code);}
}
TimeContext.prototype.utcToUt1=function(utc,eop,calendar='Gregorian'){const e=this.utcToEpoch(utc,calendar);try{return instance(eop,EopTable).toUt1(e,this);}finally{e.free();}};
TimeContext.prototype.ut1ToUtc=function(seconds,eop,calendar='Gregorian'){const e=instance(eop,EopTable).fromUt1(seconds,this);try{return this.epochToUtc(e,calendar);}finally{e.free();}};
TimeContext.prototype.tcgMinusTt=function(epoch){return h(this).to_number(h(instance(epoch,Epoch)),'TCG-TT');};
EopTable.prototype.toUt1Batch=function(epochs,context){return h(this).to_batch(h(instance(context,TimeContext)),Array.from(epochs,e=>[instance(e,Epoch).seconds,BigInt(e.nanoseconds)]));};
Object.defineProperty(TimeContext.prototype,'profile',{get(){h(this);return 'NaifSpice';}});
Object.defineProperty(ScientificContext.prototype,'profile',{get(){h(this.context);return 'IauScientific';}});
TimeContext.prototype.deltaAtForJdn=function(jdn){return h(this).delta_at_integer(bigint(jdn))??null;};
NaifEt.fromSeconds=function(seconds){return new NaifEt(seconds);};
