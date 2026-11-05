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
