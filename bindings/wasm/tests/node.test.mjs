import {test} from 'node:test';
import {readFile} from 'node:fs/promises';
import * as m from '../js/node.js';
import {run} from './suite.mjs';
test('actual WASM scientific and boundary suite',async()=>{await m.init();console.log(await run(m,p=>readFile(new URL('../../../'+p,import.meta.url),'utf8')));});
