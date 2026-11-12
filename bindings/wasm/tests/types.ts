import {init,Epoch,Duration,TimeContext,ScientificContext,NaifEt,Bdt,TimeSeries,calendar,models} from '../js/node.js';
await init();
const epoch = new Epoch(9007199254740993n).add(new Duration(1n));
const et:NaifEt=new TimeContext().toEt(epoch);
const n:number=calendar.toJd(new TimeContext().etToUtc(et));
for(const e of new TimeSeries(new Epoch(0n),new Epoch(10n),new Duration(1000000000n)))e.free();
const b:Bdt=Bdt.fromEpoch(epoch);
models.dtdbMinusTt(2451545);
// @ts-expect-error Scientific TDB cannot be used as NAIF ET.
new TimeContext().toEpoch(new ScientificContext().toTdb(epoch));
void [n,b];
