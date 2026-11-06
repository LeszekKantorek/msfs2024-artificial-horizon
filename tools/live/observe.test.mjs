import test from 'node:test';
import assert from 'node:assert/strict';
import http from 'node:http';
import {mkdtemp,readFile,open,rm} from 'node:fs/promises';
import {finished} from 'node:stream/promises';
import os from 'node:os';
import path from 'node:path';
import {capture,SseDataParser} from './observe.mjs';
test('records complete data and ignores incomplete frames',async()=>{
 const body=Buffer.concat([Buffer.from(': comment\nretry: 5\nevent: unknown\ndata: {broken\ndata: x\n\n'),Buffer.from([255,254]),Buffer.from('data: unfinished')]);
 const server=http.createServer((req,res)=>res.end(body)); await new Promise(r=>server.listen(0,'127.0.0.1',r));
 const dir=await mkdtemp(path.join(os.tmpdir(),'sse-')); const filename=path.join(dir,'out'); const f=await open(filename,'wx'); const output=f.createWriteStream();
 try {const result=await capture({url:`http://127.0.0.1:${server.address().port}/`,output}); output.end(); await finished(output); assert.equal(result.reason,'eof'); assert.equal(await readFile(filename,'utf8'),'{broken x\n');} finally {server.close(); await rm(dir,{recursive:true,force:true});}
});
test('extracts complete data messages across delimiters',()=>{const p=new SseDataParser();const records=[];for(const s of [': x\r','\nretry: 2\r\n\r','\ndata: x\r','\n\r','\ndata: y\r\r']) records.push(...p.push(Buffer.from(s)));assert.deepEqual(records,['x','y']);});

test('duration stops an idle response after headers',async()=>{const server=http.createServer((req,res)=>{res.writeHead(200);res.flushHeaders();});await new Promise(r=>server.listen(0,'127.0.0.1',r));const dir=await mkdtemp(path.join(os.tmpdir(),'sse-'));const output=(await open(path.join(dir,'out'),'wx')).createWriteStream();try{const result=await capture({url:`http://127.0.0.1:${server.address().port}/`,output,durationSeconds:1});assert.equal(result.reason,'duration');output.end();await finished(output);}finally{server.closeAllConnections();server.close();await rm(dir,{recursive:true,force:true});}});
test('cancellation stops an idle response',async()=>{const server=http.createServer((req,res)=>{res.writeHead(200);res.flushHeaders();});await new Promise(r=>server.listen(0,'127.0.0.1',r));const dir=await mkdtemp(path.join(os.tmpdir(),'sse-'));const output=(await open(path.join(dir,'out'),'wx')).createWriteStream();const controller=new AbortController();try{setTimeout(()=>controller.abort(),100);const result=await capture({url:`http://127.0.0.1:${server.address().port}/`,output,signal:controller.signal});assert.equal(result.reason,'cancelled');output.end();await finished(output);}finally{server.closeAllConnections();server.close();await rm(dir,{recursive:true,force:true});}});

test('reports mid-stream socket failure',async()=>{const server=http.createServer((req,res)=>{res.writeHead(200);res.write('data: incomplete');setTimeout(()=>res.destroy(),30);});await new Promise(r=>server.listen(0,'127.0.0.1',r));const dir=await mkdtemp(path.join(os.tmpdir(),'sse-'));const output=(await open(path.join(dir,'out'),'wx')).createWriteStream();try{await assert.rejects(capture({url:`http://127.0.0.1:${server.address().port}/`,output}));output.end();await finished(output);}finally{server.closeAllConnections();server.close();await rm(dir,{recursive:true,force:true});}});
test('returns HTTP status without recording non-SSE content',async()=>{const body=Buffer.from('error body');const server=http.createServer((req,res)=>{res.writeHead(503);res.end(body);});await new Promise(r=>server.listen(0,'127.0.0.1',r));const dir=await mkdtemp(path.join(os.tmpdir(),'sse-'));const filename=path.join(dir,'out');const output=(await open(filename,'wx')).createWriteStream();try{const result=await capture({url:`http://127.0.0.1:${server.address().port}/`,output});output.end();await finished(output);assert.equal(result.httpStatus,503);assert.equal(await readFile(filename,'utf8'),'');}finally{server.close();await rm(dir,{recursive:true,force:true});}});

test('reports writer failure',async()=>{const {Writable}=await import('node:stream');const output=new Writable({write(chunk,encoding,callback){callback(new Error('disk failure'));}});const server=http.createServer((req,res)=>res.end('data: x\n\n'));await new Promise(r=>server.listen(0,'127.0.0.1',r));try{await assert.rejects(capture({url:`http://127.0.0.1:${server.address().port}/`,output}),/disk failure/);}finally{server.close();}});


test('writes SSE data as newline-delimited records without validation',async()=>{
 const body=Buffer.from(': keepalive\r\nevent: telemetry\r\nretry: 2000\r\ndata: {"sequence":1}\r\n\r\ndata: not-json\n\ndata: {"sequence":2}\n\n');
 const server=http.createServer((req,res)=>{res.write(body.subarray(0,37));setTimeout(()=>res.end(body.subarray(37)),10);});await new Promise(r=>server.listen(0,'127.0.0.1',r));
 const dir=await mkdtemp(path.join(os.tmpdir(),'jsonl-'));const filename=path.join(dir,'out');const output=(await open(filename,'wx')).createWriteStream();
 try{await capture({url:`http://127.0.0.1:${server.address().port}/`,output});output.end();await finished(output);assert.equal(await readFile(filename,'utf8'),'{"sequence":1}\nnot-json\n{"sequence":2}\n');}finally{server.close();await rm(dir,{recursive:true,force:true});}
});


test('stops promptly on HTTP error even when the response remains open',async()=>{
 const server=http.createServer((req,res)=>{res.writeHead(503);res.flushHeaders();});await new Promise(r=>server.listen(0,'127.0.0.1',r));
 const dir=await mkdtemp(path.join(os.tmpdir(),'sse-http-'));const output=(await open(path.join(dir,'out'),'wx')).createWriteStream();const deadline=AbortSignal.timeout(1000);
 try{const result=await capture({url:`http://127.0.0.1:${server.address().port}/`,output,maxMessages:1,signal:deadline});assert.equal(result.httpStatus,503);assert.equal(deadline.aborted,false);output.end();await finished(output);}finally{server.closeAllConnections();server.close();await rm(dir,{recursive:true,force:true});}
});
