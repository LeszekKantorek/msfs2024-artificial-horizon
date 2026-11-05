import test from 'node:test';
import assert from 'node:assert/strict';
import { observeBrowser } from './browser.mjs';
const sample = sequence => ({schema_version:1,source:'demo',state:'live',sequence,sample_age_ms:0,attitude:{pitch_deg:sequence,roll_deg:sequence},ias_kt:sequence===1?123:null,altitude_ft:8500+sequence});
test('changing receipts pass without DOM or screenshot APIs', async () => {
 let binding; const signal=new AbortController();
 const page={on(){},exposeBinding:async(_n,cb)=>binding=cb,addInitScript:async()=>{},goto:async()=>{binding(null,{kind:'browser-receipt',subscription:1,data:JSON.stringify(sample(1))});binding(null,{kind:'browser-receipt',subscription:1,data:JSON.stringify(sample(2))});setTimeout(()=>signal.abort(),20);}};
 const browser={newContext:async()=>({newPage:async()=>page}),close:async()=>{}};
 const result=await observeBrowser({endpoint:'http://localhost',source:'demo',signal:signal.signal,onObservation:()=>{},testHooks:{chromium:{launch:async()=>browser}}});
 assert.equal(result.check.status,'passed'); assert.equal(result.receiptCount,2); assert.equal('renderCorrelation' in result,false);
});
test('invalid receipt survives cancellation and cleanup error', async () => {
 let binding; const signal=new AbortController();
 const page={on(){},exposeBinding:async(_n,cb)=>binding=cb,addInitScript:async()=>{},goto:async()=>{binding(null,{kind:'browser-receipt',subscription:1,data:'bad'});signal.abort();}};
 const browser={newContext:async()=>({newPage:async()=>page}),close:async()=>{throw new Error('close failed');}};
 const result=await observeBrowser({endpoint:'http://localhost',source:'demo',signal:signal.signal,onObservation:()=>{},testHooks:{chromium:{launch:async()=>browser}}});
 assert.equal(result.check.status,'failed'); assert.match(result.check.reason,/Malformed/); assert.match(result.cleanupError,/close failed/);
});

function sampleBrowser(sequence) {return {schema_version:1,source:'demo',state:'live',sequence,sample_age_ms:0,attitude:{pitch_deg:0,roll_deg:0}};}
test('browser receipt freshness expires at the end of the window', async () => {
 const {evaluateSubscriptions}=await import('./observe.mjs');
 const receipts=[{subscription:1,receivedMonotonicMs:0,snapshot:sampleBrowser(1)},{subscription:1,receivedMonotonicMs:100,snapshot:sampleBrowser(2)}];
 assert.equal(evaluateSubscriptions(receipts,'demo',500).status,'passed');
 assert.equal(evaluateSubscriptions(receipts,'demo',1100).status,'failed');
});
test('cancellation interrupts browser launch and closes a late browser', async () => {
 let finishLaunch,closed=false; const browser={close:async()=>{closed=true;}};
 const chromium={launch:()=>new Promise(resolve=>{finishLaunch=resolve;})}; const signal=new AbortController();
 const result=observeBrowser({endpoint:'http://localhost',source:'demo',signal:signal.signal,onObservation:()=>{},outputDirectory:'.',testHooks:{chromium}});
 signal.abort(); await assert.rejects(Promise.race([result,new Promise((_,reject)=>setTimeout(()=>reject(new Error('operation hung')),1500))]),/abort|cancel/i);
 finishLaunch(browser); await new Promise(resolve=>setImmediate(resolve)); assert.equal(closed,true);
});
test('browser preserves protocol failure across cancellation and page errors', async () => {
 for(const mode of ['cancel','pageerror']) {
  let binding, pageError, entered; const reached=new Promise(resolve=>entered=resolve); const signal=new AbortController();
  const page={on(_event,cb){pageError=cb;},exposeBinding:async(_name,cb)=>{binding=cb;},addInitScript:async()=>{},goto:async()=>{binding(null,{kind:'browser-receipt',subscription:1,data:'invalid json'}); if(mode==='pageerror') pageError(new Error('page broke')); entered(); if(mode==='cancel') return new Promise(()=>{}); signal.abort();},evaluate:async()=>({available:true})};
  const browser={newContext:async()=>({newPage:async()=>page}),close:async()=>{}};
  const result=observeBrowser({endpoint:'http://localhost',source:'demo',signal:signal.signal,onObservation:()=>{},outputDirectory:'.',testHooks:{chromium:{launch:async()=>browser}}});
  await reached; if(mode==='cancel') signal.abort(); const outcome=await result;
  assert.equal(outcome.check.status,'failed',mode); assert.match(outcome.check.reason,/Malformed/);
 }
});
test('navigation cancellation closes owned browser', async () => {
 let entered; const reached=new Promise(r=>entered=r),controller=new AbortController(); let closed=false;
 const page={on(){},exposeBinding:async()=>{},addInitScript:async()=>{},goto:()=>{entered();return new Promise(()=>{});}};
 const browser={newContext:async()=>({newPage:async()=>page}),close:async()=>{closed=true;}};
 const result=observeBrowser({endpoint:'http://localhost',source:'demo',signal:controller.signal,onObservation:()=>{},testHooks:{chromium:{launch:async()=>browser}}}); await reached;controller.abort();await assert.rejects(result,/cancel/i);assert.equal(closed,true);
});
test('wrong source and sequence regression remain failures on cancellation', async () => {
 for(const samples of [[{...sample(1),source:'simconnect'}],[sample(2),sample(1)]]) {
  let binding; const controller=new AbortController(); const page={on(){},exposeBinding:async(_n,cb)=>binding=cb,addInitScript:async()=>{},goto:async()=>{for(const s of samples)binding(null,{kind:'browser-receipt',subscription:1,data:JSON.stringify(s)});controller.abort();}};
  const browser={newContext:async()=>({newPage:async()=>page}),close:async()=>{}};
  const result=await observeBrowser({endpoint:'http://localhost',source:'demo',signal:controller.signal,onObservation:()=>{},testHooks:{chromium:{launch:async()=>browser}}});assert.equal(result.check.status,'failed');
 }
});
test('expired final receipt remains a failure if navigation is cancelled', async () => {
 let binding; const controller=new AbortController(); let entered; const reached=new Promise(r=>entered=r);
 const page={on(){},exposeBinding:async(_n,cb)=>binding=cb,addInitScript:async()=>{},goto:()=>{binding(null,{kind:'browser-receipt',subscription:1,data:JSON.stringify({...sample(1),sample_age_ms:999})});binding(null,{kind:'browser-receipt',subscription:1,data:JSON.stringify({...sample(2),sample_age_ms:999})});entered();return new Promise(()=>{});}};
 const browser={newContext:async()=>({newPage:async()=>page}),close:async()=>{}};
 const result=observeBrowser({endpoint:'http://localhost',source:'demo',signal:controller.signal,onObservation:()=>{},testHooks:{chromium:{launch:async()=>browser}}}); await reached; await new Promise(r=>setTimeout(r,10));controller.abort();const outcome=await result;assert.equal(outcome.check.status,'failed');assert.match(outcome.check.reason,/expired/);
});
