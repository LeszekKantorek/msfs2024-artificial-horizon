import test from 'node:test';
import assert from 'node:assert/strict';
import {launchOwned} from './process.mjs';
test('reports a missing executable after spawn',async()=>{const app=await launchOwned({exe:'missing-executable-s2',appArgs:[]}); await new Promise(resolve=>app.signal.addEventListener('abort',resolve,{once:true})); const result=await app.stop(); assert.ok(result.error);});

test('reports an owned child that exits successfully before stop',async()=>{const app=await launchOwned({exe:process.execPath,appArgs:['-e',"setTimeout(()=>process.exit(0),30)"] });await new Promise(r=>setTimeout(r,200));const result=await app.stop();assert.match(result.error,/exit/i);});
