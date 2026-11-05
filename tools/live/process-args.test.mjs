import test from 'node:test';
import assert from 'node:assert/strict';
import path from 'node:path';
import { applicationArgs } from './process.mjs';
test('passes a diagnostic journal only to SimConnect', () => {
 const dir=path.resolve('.local/live/example'); const log=path.join(dir,'diagnostic.jsonl');
 const args=applicationArgs({source:'simconnect',runDirectory:dir,logPath:log},1234);
 assert.deepEqual(args,['--source','simconnect','--listen-address','127.0.0.1','--port','1234','--log',log]);
 assert.throws(()=>applicationArgs({source:'demo',runDirectory:dir,logPath:log},1234), /SimConnect/);
 assert.throws(()=>applicationArgs({source:'simconnect',runDirectory:dir,logPath:'relative'},1234), /absolute/);
 assert.ok(!applicationArgs({source:'demo',runDirectory:dir},1234).includes('--log'));
});
test('output cap reports truncation immediately while draining', async () => {
 const {capture}=await import('./process.mjs'); const {PassThrough}=await import('node:stream'); const {mkdtemp,rm,stat}=await import('node:fs/promises'); const {tmpdir}=await import('node:os');
 const dir=await mkdtemp(path.join(tmpdir(),'capture-')); const stream=new PassThrough(); let reached=0; const c=capture(stream,path.join(dir,'stdout'),()=>reached++);
 try {stream.write(Buffer.alloc(8*1024*1024+1)); stream.end(Buffer.from('more')); await c.done; assert.equal(reached,1); assert.equal(c.result().truncated,true); assert.equal((await stat(path.join(dir,'stdout'))).size,8*1024*1024);}
 finally {await rm(dir,{recursive:true,force:true});}
});
