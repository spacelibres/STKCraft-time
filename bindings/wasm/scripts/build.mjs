import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
process.chdir(fileURLToPath(new URL('..', import.meta.url)));
execFileSync('cargo', ['build', '--locked', '--release', '--target', 'wasm32-unknown-unknown'], {stdio:'inherit'});
execFileSync('wasm-bindgen', ['--target','web','--out-dir','pkg','--out-name','stkcraft_time','target/wasm32-unknown-unknown/release/stkcraft_time_wasm.wasm'],{stdio:'inherit'});
