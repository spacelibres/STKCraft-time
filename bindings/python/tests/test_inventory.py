import json
from pathlib import Path
import pytest
import stkcraft_time as t
ROWS=json.loads((Path(__file__).parents[1]/'API_INVENTORY.json').read_text())
@pytest.mark.parametrize('row',ROWS,ids=[r['core'] for r in ROWS])
def test_public_capability_mapping(row):
    if row['status']=='unavailable':
        assert row['core'].startswith('mcp::') and row['reason'];return
    obj=t
    for part in row['python'].split('.'):
        if isinstance(obj,type) and part in getattr(obj,'__annotations__',{}):
            assert obj.__annotations__[part]; return
        obj=getattr(obj,part)
