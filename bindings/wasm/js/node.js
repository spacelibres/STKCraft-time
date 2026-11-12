import {readFile} from 'node:fs/promises';
import {init as initialize} from './index.js';
export * from './index.js';
export async function init(input){await initialize(input ?? await readFile(new URL('../pkg/stkcraft_time_bg.wasm',import.meta.url)));}
