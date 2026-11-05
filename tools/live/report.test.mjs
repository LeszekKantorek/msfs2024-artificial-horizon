import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { writeReport, exitCode } from './report.mjs';
test('writes evidence and classifies outcomes including cleanup', async () => {
 const dir=await mkdtemp(path.join(tmpdir(),'report-'));
 try { const check={id:'stream',status:'passed',expected:'fresh',observed:'fresh',evidence:[],limitations:[]};
 const result=await writeReport({manifest:{source:'demo'},observations:[{kind:'sse'}],checks:[check],cleanup:{graceful:true}},dir);
 assert.equal(result.schemaVersion,1); assert.equal(exitCode(result),0); assert.equal(JSON.parse(await readFile(path.join(dir,'manifest.json'),'utf8')).source,'demo');
 assert.equal((await readFile(path.join(dir,'observations.jsonl'),'utf8')).trim(),'{"kind":"sse"}');
 assert.equal(exitCode({...result,checks:[{status:'unavailable'}]}),2);
 assert.equal(exitCode({...result,checks:[{status:'failed'}]}),1);
 assert.equal(exitCode({...result,cleanup:{graceful:false}}),1);
 assert.equal(exitCode({...result,cancelled:true}),130);
 } finally {await rm(dir,{recursive:true,force:true});}
});
test('report write errors are propagated', async () => {
 const dir=await mkdtemp(path.join(tmpdir(),'report-error-')); const missing=path.join(dir,'missing');
 try { await assert.rejects(writeReport({manifest:{},observations:[],checks:[],cleanup:{graceful:true}},missing)); }
 finally {await rm(dir,{recursive:true,force:true});}
});
