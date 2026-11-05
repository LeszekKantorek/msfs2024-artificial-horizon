import test from 'node:test';
import assert from 'node:assert/strict';
import path from 'node:path';
import { parseArgs } from './run.mjs';
test('requires explicit source and validates limits', () => {
 assert.throws(()=>parseArgs([]),/source/); assert.throws(()=>parseArgs(['--source','demo','--dll',path.resolve('x')]),/dll/i);
 assert.throws(()=>parseArgs(['--source','demo','--duration-ms','0']),/duration/);
 assert.throws(()=>parseArgs(['--source','demo','--exe','relative']),/absolute/);
 assert.equal(parseArgs(['--source','demo']).durationMs,30000);
});
test('protocol errors are classified as application failures', async () => {
 const {observationErrorStatus}=await import('./run.mjs');
 assert.equal(observationErrorStatus(new Error('Malformed telemetry JSON')),'failed');
 assert.equal(observationErrorStatus(new Error('Partial SSE frame')),'failed');
 assert.equal(observationErrorStatus(new Error('SSE text limit reached')),'inconclusive');
 assert.equal(observationErrorStatus(new Error('Playwright unavailable')),'inconclusive');
});
test('build output is retained only to the configured bound', async () => {
 const {boundedText}=await import('./run.mjs'); const output=boundedText(4); output.add(Buffer.from('abc')); output.add(Buffer.from('def')); assert.equal(output.text(),'abcd'); assert.equal(output.truncated(),true);
});
test('retention samples beyond the cap and reports counts', async () => {
 const {boundedObservations}=await import('./run.mjs'); let stopped=0; const bounded=boundedObservations(10,()=>stopped++);
 for(let n=0;n<10001;n++) bounded.add({kind:'sse',n});
 assert.ok(bounded.observations.length<=10); assert.equal(stopped,0);
 assert.deepEqual(bounded.counts(),{observed:10001,retained:bounded.observations.length,discarded:10001-bounded.observations.length});
});
test('observation byte limit samples without exceeding the output bound', async () => {
 const {boundedObservations}=await import('./run.mjs'); let stopped=0; const bounded=boundedObservations(100,()=>stopped++,100);
 for(let n=0;n<100;n++) bounded.add({kind:'sse',n,text:'some text'});
 assert.ok(Buffer.byteLength(bounded.observations.map(x=>JSON.stringify(x)+'\n').join(''))<=100);
 assert.equal(stopped,0); assert.equal(bounded.counts().observed,100); assert.ok(bounded.counts().discarded>0);
});
test('readiness protocol violations are failures, while observation limits are inconclusive', async () => {
 const {observationErrorStatus,runFailureCheck}=await import('./run.mjs');
 for(const message of ['Source mismatch','Sequence regression','Stale live sample','Malformed telemetry envelope']) assert.equal(observationErrorStatus(new Error(message)),'failed',message);
 assert.equal(runFailureCheck(new Error('Observation stopped during readiness'),true).status,'inconclusive');
 assert.equal(runFailureCheck(new Error('Observation stopped during readiness'),true).id,'observation');
 assert.equal(runFailureCheck(new Error('Application exited'),false).status,'failed');
});
test('Demo command reports changing browser receipts without display artifacts', async () => {
 const {execFile}=await import('node:child_process'); const {promisify}=await import('node:util'); const {readFile,readdir,rm}=await import('node:fs/promises');
 assert.ok(process.env.S1_EXE,'Set S1_EXE to the rebuilt application');
 const {stdout}=await promisify(execFile)(process.execPath,['tools/live/run.mjs','--source','demo','--exe',process.env.S1_EXE,'--duration-ms','3000'],{timeout:30000,env:process.env});
 const directory=stdout.match(/Results: (.+)/)?.[1]?.trim(); assert.ok(directory,'command must report its result directory');
 try {
  const report=JSON.parse(await readFile(path.join(directory,'result.json'),'utf8'));
  const observations=(await readFile(path.join(directory,'observations.jsonl'),'utf8')).trim().split('\n').map(JSON.parse);
  const receipts=observations.filter(o=>o.kind==='browser-receipt');
  assert.equal(report.checks.find(c=>c.id==='browser receipt')?.status,'passed');
  assert.equal(report.checks.some(c=>/display/i.test(c.id)),false);
  assert.equal(observations.some(o=>o.kind==='browser-display'),false);
  assert.ok(receipts.length>2); assert.ok(new Set(receipts.map(o=>JSON.stringify([o.snapshot.attitude,o.snapshot.ias_kt,o.snapshot.altitude_ft]))).size>1);
  assert.equal((await readdir(directory)).some(name=>/\.png$/i.test(name)),false);
  assert.equal(report.cleanup.graceful,true); assert.equal(report.cleanup.forced,false); assert.equal(report.cleanup.error,null);
 } finally {await rm(directory,{recursive:true,force:true});}
});
