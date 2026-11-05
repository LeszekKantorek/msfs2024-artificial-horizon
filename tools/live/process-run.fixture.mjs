import http from 'node:http';
import {readFile} from 'node:fs/promises';
import readline from 'node:readline';
const args=process.argv.slice(2); const get=k=>args[args.indexOf(k)+1];
const mode=get('--mode'), port=Number(get('--port')), body=await readFile(get('--body-file'));
if (mode === 'early-exit') process.exit(2);
if (mode === 'stall-startup') { readline.createInterface({input:process.stdin}).on('line', line => { if (line === 'stop') process.exit(0); }); await new Promise(()=>setInterval(()=>{},1000)); }
const server=http.createServer((req,res)=>{res.writeHead(200);res.write(body);});
server.listen(port,'127.0.0.1',()=>console.log(`Listening on 127.0.0.1:${mode === 'wrong-address' ? port+1 : port}`));
readline.createInterface({input:process.stdin}).on('line',line=>{if(line==='stop' && mode!=='ignore-stop') server.close(()=>process.exit(0));});
