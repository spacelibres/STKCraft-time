export type CalendarName = 'Gregorian'|'Julian'|'Mixed';
export type Scale = 'ET'|'TT'|'GPS'|'TCG'|'TCB.IAU'|'TDB.Compact'|'TDB.IAU'|'TT.IAU';
export const Calendar: Readonly<Record<CalendarName,CalendarName>>;
export function init(input?:URL|string|BufferSource|WebAssembly.Module):Promise<void>;
export function version():string;
export interface ExactEpochJSON {schema:'stkcraft-time/1';type:'Epoch';scale:'TAI';origin:'2000-01-01T00:00:00 TAI';units:'seconds+nanoseconds';seconds:string;nanoseconds:number}
export interface DurationJSON {schema:'stkcraft-time/1';type:'Duration';units:'nanoseconds';nanoseconds:string}
export class Epoch {
 constructor(seconds:bigint,nanoseconds?:bigint); static fromTaiSeconds(seconds:number):Epoch;
 readonly seconds:bigint; readonly nanoseconds:number; add(duration:Duration):Epoch; subtract(duration:Duration):Epoch; difference(epoch:Epoch):Duration; compare(epoch:Epoch):number; toTaiSeconds():number;
 toJSON():ExactEpochJSON; static fromJSON(value:unknown):Epoch; free():void;
}
export class Duration {
 constructor(totalNanoseconds:bigint);static fromSeconds(seconds:number):Duration;
 readonly totalNanoseconds:bigint;readonly seconds:bigint;readonly nanoseconds:number;
 add(d:Duration):Duration;subtract(d:Duration):Duration;multiply(k:bigint):Duration;negate():Duration;abs():Duration;floor(step:Duration):Duration;ceil(step:Duration):Duration;round(step:Duration):Duration;toSeconds():number;toJSON():DurationJSON;static fromJSON(o:unknown):Duration;free():void;
}
export class UtcDateTime {
 constructor(year:number,month:number,day:number,hour?:number,minute?:number,second?:number);
 readonly year:number;readonly month:number;readonly day:number;readonly hour:number;readonly minute:number;readonly second:number;
 static fromDate(date:Date):UtcDateTime;toDate(loss?:'reject'|'truncate'):Date;
}
export class NaifEt {constructor(seconds:number);readonly seconds:number;readonly profile:'NaifSpice'}
export class NaifJulianUtc {private constructor();readonly days:number}
export class IauTdb {private constructor();readonly seconds:number;readonly model:'GeocentricFairheadBretagnon'|'ObserverFairheadBretagnon';readonly profile:'IauScientific'}
export interface FormatOptions {calendar?:CalendarName;style?:'iso8601'|'rfc3339'|'spice'|'rfc2822'|'strftime';precision?:number;pattern?:string}
export interface LeapMetadata {coverage:[string,string];source:'IersOnly'|'Sofa';policy:string;deltet:[number,number,number,number,number];entries:[string,number][]}
export class TimeContext {
 constructor(lskText?:string|Uint8Array);select(options?:{source?:'IersOnly'|'Sofa';frozenAfterJdn?:bigint|null}):TimeContext;readonly metadata:LeapMetadata;
 utcToEpoch(utc:UtcDateTime,calendar?:CalendarName):Epoch;epochToUtc(epoch:Epoch,calendar?:CalendarName):UtcDateTime;
 utcToEt(utc:UtcDateTime,calendar?:CalendarName):NaifEt;etToUtc(et:NaifEt,calendar?:CalendarName):UtcDateTime;toEt(epoch:Epoch):NaifEt;toEpoch(et:NaifEt):Epoch;etToSpiceJulianUtc(et:NaifEt):NaifJulianUtc;
 parseWgcWeekTow(text:string):number;wgcWeekTowToEt(text:string):NaifEt;wgcGpsSecondsToEt(seconds:number):NaifEt;gpsSecondsToWeekTow(seconds:number):string;etToWgcGpsDisplaySeconds(et:NaifEt):number;etToWgcCalendarDoy(et:NaifEt,system:'UTC'|'TDB'|'TT'|'TDT'|'GPS'):string;etToWgcCalendarYmd(et:NaifEt,system:'UTC'|'TDB'|'TT'|'TDT'|'GPS'):string;
 toScale(epoch:Epoch,scale:Scale):number;fromScale(seconds:number,scale:Exclude<Scale,'TT.IAU'>):Epoch;
 format(utc:UtcDateTime,options?:FormatOptions):string;utcToEtBatch(values:Iterable<UtcDateTime>,calendar?:CalendarName):Float64Array;etToUtcBatch(values:Iterable<number>,calendar?:CalendarName):UtcDateTime[];free():void;
}
export class ScientificContext {constructor();ttSeconds(epoch:Epoch):number;toTdb(epoch:Epoch):IauTdb;toTdbAtObserver(epoch:Epoch,observer:{ut1Fraction:number;longitude:number;uKm:number;vKm:number}):IauTdb;free():void}
export interface SclkMetadata {spacecraftId:number;dataType:number;timeSystem:'Tdb'|'Tdt';nFields:number;moduli:bigint[];offsets:bigint[];coefficients:{ticks:number;seconds:number;rate:number}[];partitionStarts:number[];partitionEnds:number[];ticksPerMsb:number}
export class SclkStringContext {constructor();free():void}
export class SclkKernel {constructor(text:string|Uint8Array,spacecraftId:number);readonly metadata:SclkMetadata;encodeNativeSclk(text:string):number;encodeNativeSclkWithContext(text:string,state:SclkStringContext):number;decodeNativeSclk(ticks:number):string;nativeSclkToEt(text:string,context:TimeContext):NaifEt;nativeSclkToEtWithContext(text:string,context:TimeContext,state:SclkStringContext):NaifEt;nativeEtToSclk(et:NaifEt,context:TimeContext):string;ticksToEt(ticks:number,context:TimeContext):NaifEt;etToTicks(et:NaifEt,context:TimeContext):number;ticksToEpoch(ticks:number,context:TimeContext):Epoch;epochToTicks(epoch:Epoch,context:TimeContext):number;fieldsFromTicks(ticks:number):bigint[];ticksFromFields(fields:Iterable<bigint>):number;parseString(text:string):bigint[];formatString(fields:Iterable<bigint>):string;parallelSeconds(ticks:number):number;ticksFromParallel(seconds:number):number;ticksToEtBatch(values:Iterable<number>,context:TimeContext):Float64Array;free():void}
export interface EopMetadata {range:[number,number];samples:{mjd:number;ut1MinusUtc:number;observed:boolean}[]}
export class EopTable {constructor(text:string|Uint8Array,series?:'BulletinA'|'BulletinB'|'FinalPreferred');observedOnly():EopTable;readonly metadata:EopMetadata;interpolate(mjd:number):number;toUt1(epoch:Epoch,context:TimeContext):number;ut1MinusUtc(epoch:Epoch,context:TimeContext):number;fromUt1(seconds:number,context:TimeContext):Epoch;free():void}
export interface KernelDate {secondsPastJ2000:number;year:number;month:number;day:number;hour:number;minute:number;second:number}
export function parseKernelDate(text:string):KernelDate;
export class TimeSeries implements Iterable<Epoch> {constructor(start:Epoch,end:Epoch,step:Duration,options?:{inclusive?:boolean});[Symbol.iterator]():Generator<Epoch>}
export class Bdt {constructor(seconds:bigint,nanoseconds?:bigint);static fromEpoch(epoch:Epoch):Bdt;static fromSeconds(seconds:number):Bdt;static fromNanoseconds(ns:bigint):Bdt;static fromWeekNanoseconds(week:bigint,nanoseconds:bigint):Bdt;readonly seconds:bigint;readonly nanoseconds:number;readonly totalNanoseconds:bigint;toSeconds():number;toEpoch():Epoch;weekNanoseconds():[bigint,bigint];static fromUtc(utc:UtcDateTime,context:TimeContext,calendar?:CalendarName):Bdt;toUtc(context:TimeContext,calendar?:CalendarName):UtcDateTime;static fromEt(et:NaifEt,context:TimeContext):Bdt;toEt(context:TimeContext):NaifEt;toJSON():object;static fromJSON(o:unknown):Bdt;free():void}
export const calendar:{isGregorianLeap(year:number):boolean;isJulianLeap(year:number):boolean;daysInMonth(year:number,month:number,cal?:CalendarName):number|null;toJdn(year:number,month:number,day:number,cal?:CalendarName):bigint;fromJdn(jdn:bigint,cal?:CalendarName):[number,number,number];toJd(utc:UtcDateTime,cal?:CalendarName):number;jdToMjd(jd:number):number;mjdToJd(mjd:number):number};
export const models:{compactMinusTt(jd:number):number;dtdbMinusTt(jd:number):number;dtdbMinusTtSplit(jd1:number,jd2:number):number;dtdbAtObserver(jd1:number,jd2:number,ut1:number,longitude:number,uKm:number,vKm:number):number;tcbMinusTdb(tdb:number):number;sofaDeltaAt(mjd:number):number};
export interface TimeContext {deltaAt(jdn:bigint,dayFraction?:number):number}
export namespace TimeContext {function fromEntries(entries:Iterable<readonly [bigint,number]>,deltet:Iterable<number>):TimeContext}
export interface EopSample {mjd:number;ut1MinusUtc:number;observed:boolean}
export interface EopTable {fromUt1Batch(values:Iterable<number>,context:TimeContext):Epoch[]}
export namespace EopTable {function fromSamples(samples:Iterable<EopSample>):EopTable}
export interface SclkKernel {etToTicksBatch(values:Iterable<number>,context:TimeContext):Float64Array}
export class NaifUtcContext {constructor(entries:Iterable<readonly [bigint,number]>,deltet:Iterable<number>);etToSpiceJulianUtc(et:NaifEt):NaifJulianUtc}
export const constants:Readonly<Record<string,number|bigint>>;
export const compatibility:{utcToEt(utc:UtcDateTime,context:TimeContext,calendar?:CalendarName):number;etToUtc(seconds:number,context:TimeContext,calendar?:CalendarName):UtcDateTime;taiToEt(epoch:Epoch,context:TimeContext):number;etToTai(seconds:number,context:TimeContext):Epoch;toScale(epoch:Epoch,scale:Scale,context:TimeContext):number;fromScale(seconds:number,scale:Scale,context:TimeContext):Epoch};
export namespace Duration {function fromParts(seconds:bigint,nanoseconds?:bigint):Duration}
export interface Duration {divide(k:bigint):Duration}
export const formatting:{etcal(seconds:number,precision?:number):string;jdToCalendar(jd:number):UtcDateTime};
export class TimeError extends Error {constructor(code:string,message:string);readonly code:string;readonly index?:number}
export class ContractError extends TimeError {}
export class DtdbError extends TimeError {}
export interface TimeContext {utcToUt1(utc:UtcDateTime,eop:EopTable,calendar?:CalendarName):number;ut1ToUtc(seconds:number,eop:EopTable,calendar?:CalendarName):UtcDateTime;tcgMinusTt(epoch:Epoch):number}
export interface EopTable {toUt1Batch(epochs:Iterable<Epoch>,context:TimeContext):Float64Array}
export interface TimeContext {readonly profile:'NaifSpice';deltaAtForJdn(jdn:bigint):number|null}
export interface ScientificContext {readonly profile:'IauScientific'}
export namespace NaifEt {function fromSeconds(seconds:number):NaifEt}
