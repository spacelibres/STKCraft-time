from stkcraft_time import Bdt, Calendar, Duration, Epoch, IauTdb, NaifEt, ScientificContext, TimeContext, UtcDateTime
ctx = TimeContext()
e: Epoch = Epoch(2**53+1, 1) + Duration.NANOSECOND
et: NaifEt = ctx.utc_to_et(UtcDateTime(2000,1,1))
e = ctx.to_epoch(et)
scientific: IauTdb = ScientificContext().to_tdb(e)
bdt: Bdt = Bdt.from_et(et,ctx)
e = bdt.to_epoch()
# Runtime model separation is also part of the static contract.
ctx.to_epoch(scientific)  # type: ignore[arg-type]
