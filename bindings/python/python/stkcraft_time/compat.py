"""Explicit legacy f64 conversion API; new integration should use TimeContext."""
from . import _n,_call,_float,Epoch,TimeContext,Calendar

def _to(epoch,lsk,scale): return _call(_n.legacy_to,epoch._n,lsk._n,scale)
def _from(seconds,lsk,scale): return Epoch._wrap(_call(_n.legacy_from,_float(seconds),lsk._n,scale))
def utc_to_tai(utc,calendar,lsk): return TimeContext(lsk).utc_to_tai(utc,calendar)
def tai_to_utc(epoch,calendar,lsk): return TimeContext(lsk).tai_to_utc(epoch,calendar)
def utc_to_et(utc,calendar,lsk): return _call(_n.legacy_utc_et,utc._n,lsk._n,Calendar(calendar).value)
def et_to_utc(et,calendar,lsk):
    from . import NaifEt
    from . import UtcDateTime
    return UtcDateTime._wrap(_call(_n.legacy_et_utc,_float(et),lsk._n,Calendar(calendar).value))
def tai_to_gps(epoch): return _to(epoch,__import__('stkcraft_time').LeapSecondTable(),'gps')
def gps_to_tai(seconds): return _from(seconds,__import__('stkcraft_time').LeapSecondTable(),'gps')
def ut1_minus_utc(epoch,lsk,eop): return eop.ut1_minus_utc_at_epoch(epoch,lsk)
def tai_to_ut1(epoch,lsk,eop): return eop.tai_to_ut1(epoch,lsk)
def ut1_to_tai(ut1,lsk,eop): return eop.ut1_to_tai(ut1,lsk)
def utc_to_ut1(utc,calendar,lsk,eop): return eop.utc_to_ut1(utc,lsk,calendar)
def ut1_to_utc(ut1,calendar,lsk,eop): return eop.ut1_to_utc(ut1,lsk,calendar)

def tai_to_et(epoch,lsk): return _to(epoch,lsk,'et')
def et_to_tai(seconds,lsk): return _from(seconds,lsk,'et')

def tai_to_tt(epoch,lsk): return _to(epoch,lsk,'tt')
def tt_to_tai(seconds,lsk): return _from(seconds,lsk,'tt')

def tai_to_tcg(epoch,lsk): return _to(epoch,lsk,'tcg')
def tcg_to_tai(seconds,lsk): return _from(seconds,lsk,'tcg')

def tai_to_tcb(epoch,lsk): return _to(epoch,lsk,'tcb')
def tcb_to_tai(seconds,lsk): return _from(seconds,lsk,'tcb')

def tai_to_tdb(epoch,lsk): return _to(epoch,lsk,'tdb')
def tdb_to_tai(seconds,lsk): return _from(seconds,lsk,'tdb')

def tai_to_tdb_iau(epoch,lsk): return _to(epoch,lsk,'tdb_iau')
def tdb_iau_to_tai(seconds,lsk): return _from(seconds,lsk,'tdb_iau')

def tcg_minus_tt(epoch,lsk): return _to(epoch,lsk,'tcg_minus_tt')
