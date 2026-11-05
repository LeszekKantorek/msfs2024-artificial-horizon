import http from 'node:http';
import {readFile, writeFile} from 'node:fs/promises';
import readline from 'node:readline';
const args=process.argv.slice(2); const get=k=>args.includes(k) ? args[args.indexOf(k)+1] : undefined;
const mode=get('--mode'), port=Number(get('--port')), body=await readFile(get('--body-file'));
const marker=get('--stop-marker');
const stopped=async()=>{if(marker) await writeFile(marker,'stop');};
if (mode === 'early-exit') process.exit(2);
if (mode === 'stall-startup') { readline.createInterface({input:process.stdin}).on('line', async line => { if (line === 'stop') { await stopped(); process.exit(Number(get('--stop-code') ?? 0)); } }); await new Promise(()=>setInterval(()=>{},1000)); }
const server=http.createServer((req,res)=>{const status=Number(get('--status') ?? 200);if(get('--request-marker')) void writeFile(get('--request-marker'),'request');if(get('--no-headers') === 'yes') return;res.writeHead(status);if(status >= 400) res.end(body);else {res.write(body);if(get('--disconnect') === 'yes') setTimeout(()=>res.destroy(),30);}});
server.listen(port,'127.0.0.1',()=>console.log(`Listening on 127.0.0.1:${mode === 'wrong-address' ? port+1 : port}`));
readline.createInterface({input:process.stdin}).on('line',async line=>{if(line==='stop' && mode!=='ignore-stop') {await stopped();server.close(()=>process.exit(Number(get('--stop-code') ?? 0)));}});
