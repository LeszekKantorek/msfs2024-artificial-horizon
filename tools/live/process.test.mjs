import test from 'node:test';
import assert from 'node:assert/strict';
import {launchOwned} from './process.mjs';
test('rejects a missing executable and cleans up',async()=>{await assert.rejects(launchOwned({exe:'missing-executable-s2',appArgs:[],url:'http://127.0.0.1:8080/'},AbortSignal.timeout(2000)));});

test('interrupts stalled startup and stops the child',async()=>{const {mkdtemp,writeFile,rm}=await import('node:fs/promises');const os=await import('node:os');const path=await import('node:path');const dir=await mkdtemp(path.join(os.tmpdir(),'stall-'));const body=path.join(dir,'body');await writeFile(body,'x');try{await assert.rejects(launchOwned({exe:process.execPath,url:'http://127.0.0.1:1/',appArgs:[path.resolve('tools/live/process-run.fixture.mjs'),'--mode','stall-startup','--port','1','--body-file',body]},AbortSignal.timeout(200)),{name:'AbortError'});}finally{await rm(dir,{recursive:true,force:true});}});
