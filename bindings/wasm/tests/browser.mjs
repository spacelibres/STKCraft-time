import {createServer} from 'node:http';
import {readFile} from 'node:fs/promises';
import {resolve,extname,sep} from 'node:path';
import {fileURLToPath} from 'node:url';
import {chromium} from 'playwright';
import {build} from 'esbuild';
const root=resolve(fileURLToPath(new URL('../../..',import.meta.url)));
const server=createServer(async(req,res)=>{try{const p=resolve(root,'.'+decodeURIComponent(req.url.split('?')[0]));if(!p.startsWith(root+sep))throw Error();res.setHeader('Content-Type',extname(p)==='.wasm'?'application/wasm':extname(p)==='.js'||extname(p)==='.mjs'?'text/javascript':'text/plain');res.end(await readFile(p));}catch{res.statusCode=404;res.end();}}).listen(0,'127.0.0.1');
await new Promise(r=>server.once('listening',r));const url=`http://127.0.0.1:${server.address().port}`;
let browser;
try{browser=await chromium.launch({channel:process.env.STKCRAFT_BROWSER_CHANNEL||'chrome',headless:true});const page=await browser.newPage();await page.goto(url+'/bindings/wasm/README.md');
const results=await page.evaluate(async()=>{const m=await import('/bindings/wasm/js/index.js');await m.init();const {run}=await import('/bindings/wasm/tests/suite.mjs');return run(m,async p=>(await fetch('/'+p)).text());});console.log(JSON.stringify({browser:browser.version(),...results}));
const worker=await page.evaluate(()=>new Promise((resolve,reject)=>{const w=new Worker(URL.createObjectURL(new Blob([`import {init,Epoch} from '${location.origin}/bindings/wasm/js/index.js'; await init(await WebAssembly.compile(await (await fetch('${location.origin}/bindings/wasm/pkg/stkcraft_time_bg.wasm')).arrayBuffer())); postMessage(String(new Epoch(9007199254740993n).seconds));`],{type:'text/javascript'})),{type:'module'});w.onmessage=e=>{w.terminate();resolve(e.data)};w.onerror=e=>reject(Error(e.message));}));if(worker!=='9007199254740993')throw Error('Worker integer transport failed');
await build({entryPoints:[fileURLToPath(new URL('../js/index.js',import.meta.url))],bundle:true,format:'esm',platform:'browser',outfile:fileURLToPath(new URL('../target/bundle.js',import.meta.url))});
const bundled=await page.evaluate(async()=>{const m=await import('/bindings/wasm/target/bundle.js');await m.init(new URL('/bindings/wasm/pkg/stkcraft_time_bg.wasm',location));return String(new m.Epoch(9007199254740993n).seconds);});if(bundled!==worker)throw Error('Bundler transport failed');console.log('Module worker and esbuild consumer passed');
}finally{await browser?.close();server.close();}
