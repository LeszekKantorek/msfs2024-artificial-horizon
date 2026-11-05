import test from 'node:test';
import assert from 'node:assert/strict';
import { SseParser, evaluateSnapshots, parseJournal } from './observe.mjs';

const sample = (sequence, extra = {}) => ({ schema_version: 1, source: 'simconnect', state: 'live', sequence, sample_age_ms: 10, attitude: {pitch_deg: 1, roll_deg: 2}, ...extra });
test('parses split UTF-8, CRLF, multiline and heartbeats', () => {
  const events = []; const p = new SseParser(event => events.push(event));
  const bytes = Buffer.from(': heartbeat\r\nevent: telemetry\r\ndata: {"text":\r\ndata: "é"}\r\n\r\n');
  for (const byte of bytes) p.push(Buffer.from([byte]));
  p.finish(); assert.deepEqual(events, [{event:'telemetry', data:'{"text":\n"é"}'}]);
});
test('rejects interrupted event', () => { const p = new SseParser(() => {}); p.push(Buffer.from('event: telemetry\ndata: {}')); assert.throws(() => p.finish(), /partial/i); });
test('accepts equal values and sequence gaps but rejects regressions and stale live samples', () => {
  assert.equal(evaluateSnapshots([sample(1), sample(3)], 'simconnect').status, 'passed');
  assert.equal(evaluateSnapshots([sample(3), sample(2)], 'simconnect').status, 'failed');
  assert.equal(evaluateSnapshots([sample(1, {sample_age_ms:1000})], 'simconnect').status, 'failed');
  assert.equal(evaluateSnapshots([sample(1, {source:'demo'})], 'simconnect').status, 'failed');
});
test('journal validates session and reports partial records', () => {
 const start = JSON.stringify({session_id:'a',record:0,kind:'session_start',data:{schema_version:5}});
 const result = parseJournal(start+'\n{"partial"');
 assert.equal(result.sessionId, 'a'); assert.equal(result.complete, false);
 assert.throws(() => parseJournal(start+'\n'+JSON.stringify({session_id:'b',record:1,kind:'publication',data:{}})+'\n'), /session/i);
});
test('correlation rejects changed publications and marks missing records incomplete', async () => {
 const {correlatePublications}=await import('./observe.mjs');
 const s=sample(2); const start={session_id:'a',record:0,kind:'session_start',data:{schema_version:5}};
 const pub={session_id:'a',record:1,kind:'publication',data:{snapshot:s}}; const end={session_id:'a',record:2,kind:'session_end',data:{}};
 const journal=parseJournal([start,pub,end].map(JSON.stringify).join('\n')+'\n');
 assert.equal(correlatePublications(journal,[s]).status,'passed');
 assert.equal(correlatePublications(journal,[{...s,attitude:{pitch_deg:4,roll_deg:2}}]).status,'failed');
 assert.equal(correlatePublications(journal,[sample(3)]).status,'inconclusive');
});
test('heartbeat-only tail expires an otherwise fresh window', async () => {
 const {evaluateWindow}=await import('./observe.mjs');
 const records=[{snapshot:sample(1),receivedMonotonicMs:100},{snapshot:sample(2),receivedMonotonicMs:200}];
 assert.equal(evaluateWindow(records,'simconnect',500).status,'passed');
 assert.equal(evaluateWindow(records,'simconnect',1200).status,'failed');
});
test('a new browser subscription resets sequence comparisons', async () => {
 const {evaluateSubscriptions}=await import('./observe.mjs');
 assert.equal(evaluateSubscriptions([{subscription:1,snapshot:sample(9)},{subscription:1,snapshot:sample(10)},{subscription:2,snapshot:sample(1)},{subscription:2,snapshot:sample(2)}],'simconnect').status,'passed');
 assert.equal(evaluateSubscriptions([{subscription:1,snapshot:sample(9)},{subscription:1,snapshot:sample(8)}],'simconnect').status,'failed');
});
test('oversized incomplete SSE text is rejected before unbounded buffering', () => {
 const p=new SseParser(()=>{}); assert.throws(()=>p.push(Buffer.alloc(8*1024*1024+1, 65)),/limit/i);
});
test('a stale final source state is not a passing fresh window', () => {
 assert.equal(evaluateSnapshots([sample(1),sample(2),sample(3,{state:'stale',attitude:null})],'simconnect').status,'unavailable');
});
test('journal reader accepts split UTF-8 records and tracks incomplete tail', async () => {
 const {JournalReader}=await import('./observe.mjs'); const reader=new JournalReader();
 const start=JSON.stringify({session_id:'é',record:0,kind:'session_start',data:{schema_version:5}})+'\n';
 const bytes=Buffer.from(start); for(const byte of bytes) reader.push(Buffer.from([byte]));
 reader.push(Buffer.from('{"record":')); const result=reader.finish();
 assert.equal(result.sessionId,'é'); assert.equal(result.records.length,1); assert.equal(result.complete,false);
});
test('journal polling reads appended bytes once', async () => {
 const {JournalReader}=await import('./observe.mjs'); const {mkdtemp,writeFile,appendFile,rm}=await import('node:fs/promises'); const {tmpdir}=await import('node:os'); const path=(await import('node:path')).default;
 const dir=await mkdtemp(path.join(tmpdir(),'journal-')); const file=path.join(dir,'journal');
 try { const reader=new JournalReader(); await reader.readAvailable(file); await writeFile(file,JSON.stringify({session_id:'x',record:0,kind:'session_start',data:{schema_version:5}})+'\n'); await reader.readAvailable(file); await reader.readAvailable(file); await appendFile(file,JSON.stringify({session_id:'x',record:1,kind:'session_end',data:{}})+'\n'); await reader.readAvailable(file); const result=reader.finish(); assert.equal(result.records.length,2); assert.equal(result.complete,true); }
 finally {await rm(dir,{recursive:true,force:true});}
});
test('a failed diagnostic session is an application failure', async () => {
 const {evaluateJournal}=await import('./observe.mjs');
 assert.equal(evaluateJournal({complete:true,records:[{kind:'session_end',data:{success:false,error:'missing DLL'}}]}).status,'failed');
 assert.equal(evaluateJournal({complete:false,records:[]}).status,'inconclusive');
 assert.equal(evaluateJournal({complete:true,records:[{kind:'session_end',data:{success:true}}]}).status,'passed');
});
test('publication comparison ignores JSON object key order and independent age measurement', async () => {
 const {correlatePublications}=await import('./observe.mjs'); const s=sample(2);
 const publication={attitude:{roll_deg:2,pitch_deg:1},sample_age_ms:0,state:'live',source:'simconnect',sequence:2,schema_version:1};
 assert.equal(correlatePublications({complete:true,records:[{kind:'publication',data:{snapshot:publication}}]},[s]).status,'passed');
});
test('latest reconnected subscription must itself become fresh', async () => {
 const {evaluateSubscriptions}=await import('./observe.mjs');
 assert.equal(evaluateSubscriptions([{subscription:1,snapshot:sample(1)},{subscription:1,snapshot:sample(2)},{subscription:2,snapshot:sample(1)}],'simconnect').status,'unavailable');
});
test('optional fields are assessed independently', async () => {
 const {fieldAvailability}=await import('./observe.mjs');
 const checks=fieldAvailability([sample(1,{altitude_ft:8500}),sample(2,{altitude_ft:8500})]);
 assert.equal(checks.ias_kt.status,'unavailable'); assert.equal(checks.altitude_ft.status,'passed');
});
test('readiness finishes after fresh advancing samples and times out without them', async () => {
 const {waitForReadiness}=await import('./observe.mjs'); const {createServer}=await import('node:http');
 let sequence=0; const server=createServer((req,res)=>{res.writeHead(200,{'content-type':'text/event-stream'}); const timer=setInterval(()=>res.write(`event: telemetry\ndata: ${JSON.stringify(sample(sequence++))}\n\n`),20); res.on('close',()=>clearInterval(timer));});
 await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve)); const endpoint=`http://127.0.0.1:${server.address().port}`;
 try {assert.equal((await waitForReadiness({endpoint,source:'simconnect',timeoutMs:1000,signal:new AbortController().signal,onObservation:()=>{}})).status,'passed');}
 finally {server.closeAllConnections(); await new Promise(resolve=>server.close(resolve));}
});
test('readiness timeout ends with unavailable status', async () => {
 const {waitForReadiness}=await import('./observe.mjs'); const {createServer}=await import('node:http');
 const server=createServer((req,res)=>{res.writeHead(200,{'content-type':'text/event-stream'});res.write(': heartbeat\n\n');}); await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
 try {const result=await waitForReadiness({endpoint:`http://127.0.0.1:${server.address().port}`,source:'demo',timeoutMs:100,signal:new AbortController().signal,onObservation:()=>{}}); assert.equal(result.status,'unavailable');}
 finally {server.closeAllConnections();await new Promise(resolve=>server.close(resolve));}
});
test('age-based serialization expiry leaves publication correlation ambiguous', async () => {
 const {correlatePublications}=await import('./observe.mjs'); const publication=sample(1,{sample_age_ms:900}); const expired=sample(1,{state:'stale',sample_age_ms:1100,attitude:null});
 assert.equal(correlatePublications({complete:true,records:[{kind:'publication',data:{snapshot:publication}}]},[expired]).status,'inconclusive');
});
test('diagnostic reader accepts more than 10000 records within its byte limit', () => {
 const records = [{session_id:'a',record:0,kind:'session_start',data:{schema_version:5}}];
 for(let i=1;i<=10000;i++) records.push({session_id:'a',record:i,kind:'raw',data:{}});
 records.push({session_id:'a',record:10001,kind:'session_end',data:{success:true}});
 const result=parseJournal(records.map(JSON.stringify).join('\n')+'\n');
 assert.equal(result.complete,true);
 assert.equal(result.records.length,10002);
});
test('incremental validation retains errors and checks final freshness across reconnects', async () => {
 const {StreamValidator}=await import('./observe.mjs'); const v=new StreamValidator('simconnect');
 for(let i=0;i<10001;i++) v.add({subscription:1,receivedMonotonicMs:i,snapshot:sample(i)});
 v.add({subscription:1,receivedMonotonicMs:10001,snapshot:sample(2)});
 v.add({subscription:2,receivedMonotonicMs:10002,snapshot:sample(1)});
 v.add({subscription:2,receivedMonotonicMs:10003,snapshot:sample(2)});
 assert.equal(v.result(10003).status,'failed');
 const clean=new StreamValidator('simconnect'); clean.add({subscription:1,receivedMonotonicMs:0,snapshot:sample(1)}); clean.add({subscription:1,receivedMonotonicMs:1,snapshot:sample(2)}); clean.add({subscription:2,receivedMonotonicMs:2,snapshot:sample(1)}); clean.add({subscription:2,receivedMonotonicMs:3,snapshot:sample(2)}); assert.equal(clean.result(3).status,'passed'); assert.equal(clean.result(1000).status,'failed');
});
test('SSE limit applies to a pending frame, not cumulative completed frames', () => {
 let count=0; const p=new SseParser(()=>count++); const frame=Buffer.from(`data: ${'a'.repeat(1024)}\n\n`);
 for(let i=0;i<8200;i++) p.push(frame);
 p.finish(); assert.equal(count,8200);
 const oversized=new SseParser(()=>{});
 assert.throws(()=>oversized.push(Buffer.from(`data: ${'a'.repeat(8*1024*1024)}\n`)),/limit/i);
});
test('clean SSE EOF before cancellation is not a passing observation', async () => {
 const {observeSse}=await import('./observe.mjs'); const {createServer}=await import('node:http');
 const server=createServer((_req,res)=>{res.writeHead(200,{'content-type':'text/event-stream'}); for(const n of [1,2]) res.write(`event: telemetry\ndata: ${JSON.stringify(sample(n))}\n\n`); res.end();});
 await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
 try {const result=await observeSse({endpoint:`http://127.0.0.1:${server.address().port}`,source:'simconnect',signal:new AbortController().signal,onObservation:()=>{}}); assert.notEqual(result.check.status,'passed');}
 finally {server.closeAllConnections(); await new Promise(resolve=>server.close(resolve));}
});
