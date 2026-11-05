import test from 'node:test';
import assert from 'node:assert/strict';
import { observeBrowser } from './browser.mjs';
test('browser dependency failure remains explicit', async () => {
 const previous=process.env.PLAYWRIGHT_MODULE; process.env.PLAYWRIGHT_MODULE='missing-playwright-for-live-observer';
 try { await assert.rejects(observeBrowser({endpoint:'http://127.0.0.1:1',source:'demo',signal:new AbortController().signal,onObservation:()=>{},outputDirectory:'.'}), /PLAYWRIGHT_MODULE/); }
 finally { if(previous===undefined) delete process.env.PLAYWRIGHT_MODULE; else process.env.PLAYWRIGHT_MODULE=previous; }
});
test('stable browser samples support display checks while varying samples remain ambiguous', async () => {
 const {evaluateStableDisplay}=await import('./browser.mjs');
 const s={...sampleBrowser(1),ias_kt:null,altitude_ft:8500};
 const events=[{kind:'browser-receipt',receivedMonotonicMs:0,snapshot:s},{kind:'browser-receipt',receivedMonotonicMs:500,snapshot:{...s,sequence:2}},{kind:'browser-display',receivedMonotonicMs:501,display:{available:true,airspeed:{state:'unavailable',label:'Indicated airspeed unavailable'},altitude:{state:'available',label:'Barometric altitude 8500 feet'},rotation:'rotate(0 0 0)',pitch:'translate(0 0)'}}];
 assert.equal(evaluateStableDisplay(events).status,'passed');
 const wrong=structuredClone(events); wrong[2].display.altitude.label='Barometric altitude 7000 feet'; assert.equal(evaluateStableDisplay(wrong).status,'failed');
 const variable=structuredClone(events); variable[1].snapshot.altitude_ft=8501; assert.equal(evaluateStableDisplay(variable).status,'inconclusive');
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
 signal.abort(); await assert.rejects(Promise.race([result,new Promise((_,reject)=>setTimeout(()=>reject(new Error('operation hung')),500))]),/abort|cancel/i);
 finishLaunch(browser); await new Promise(resolve=>setImmediate(resolve)); assert.equal(closed,true);
});
test('cancellation interrupts pending browser operations and closes the browser', async () => {
 for (const stalled of ['goto','evaluate','screenshot']) {
  let closed=false,entered; const reached=new Promise(resolve=>entered=resolve);
  const page={on(){},exposeBinding:async()=>{},addInitScript:async()=>{},goto:async()=>{if(stalled==='goto'){entered();return new Promise(()=>{});}},evaluate:async()=>{if(stalled==='evaluate'){entered();return new Promise(()=>{});} return {available:true};},screenshot:async()=>{if(stalled==='screenshot'){entered();return new Promise(()=>{});}}};
  const browser={newContext:async()=>({newPage:async()=>page}),close:async()=>{closed=true;}};
  const signal=new AbortController(); const result=observeBrowser({endpoint:'http://localhost',source:'demo',signal:signal.signal,onObservation:()=>{},outputDirectory:'.',testHooks:{chromium:{launch:async()=>browser}}});
  await reached; signal.abort(); await assert.rejects(Promise.race([result,new Promise((_,reject)=>setTimeout(()=>reject(new Error('operation hung')),500))]),/abort|cancel/i); assert.equal(closed,true,stalled);
 }
});
test('a covered instrument cannot pass on retained SVG attributes', async () => {
 const {evaluateBrowserCheck,evaluateStableDisplay}=await import('./browser.mjs');
 const receipts=[{subscription:1,receivedMonotonicMs:0,snapshot:sampleBrowser(1)},{subscription:1,receivedMonotonicMs:500,snapshot:sampleBrowser(2)}];
 assert.equal(evaluateBrowserCheck(receipts,'demo',600,false).status,'unavailable');
 const events=[...receipts.map(r=>({...r,kind:'browser-receipt'})),{kind:'browser-display',receivedMonotonicMs:600,display:{available:false,airspeed:{label:'Indicated airspeed unavailable'},altitude:{label:'Barometric altitude unavailable'},rotation:'rotate(0 0 0)',pitch:'translate(0 0)'}}];
 assert.notEqual(evaluateStableDisplay(events).status,'passed');
});
test('stable display rejects wrong local availability even when labels match', async () => {
 const {evaluateStableDisplay}=await import('./browser.mjs');
 const sample={...sampleBrowser(1),ias_kt:null,altitude_ft:8500};
 const events=[{kind:'browser-receipt',receivedMonotonicMs:0,snapshot:sample},{kind:'browser-receipt',receivedMonotonicMs:500,snapshot:{...sample,sequence:2}},{kind:'browser-display',receivedMonotonicMs:501,display:{available:true,airspeed:{state:'unavailable',label:'Indicated airspeed unavailable'},altitude:{state:'available',label:'Barometric altitude 8500 feet'},rotation:'rotate(0 0 0)',pitch:'translate(0 0)'}}];
 for(const [field,state] of [['airspeed','available'],['altitude','unavailable']]) {const bad=structuredClone(events);bad[2].display[field].state=state;assert.equal(evaluateStableDisplay(bad).status,'failed',field);}
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
test('page error does not mask a known browser protocol failure on normal loop exit', async () => {
 let binding, pageError; const signal=new AbortController();
 const page={on(_event,cb){pageError=cb;},exposeBinding:async(_name,cb)=>{binding=cb;},addInitScript:async()=>{},goto:async()=>{binding(null,{kind:'browser-receipt',subscription:1,data:'bad'});pageError(new Error('page broke'));},evaluate:async()=>{setTimeout(()=>signal.abort(),5);return {available:false};}};
 const browser={newContext:async()=>({newPage:async()=>page}),close:async()=>{}};
 const outcome=await observeBrowser({endpoint:'http://localhost',source:'demo',signal:signal.signal,onObservation:()=>{},outputDirectory:'.',testHooks:{chromium:{launch:async()=>browser}}});
 assert.equal(outcome.check.status,'failed'); assert.match(outcome.check.reason,/Malformed/);
});
test('browser reports invalid receipt and cleanup failure separately', async () => {
 let binding; const signal=new AbortController();
 const page={on(){},exposeBinding:async(_name,cb)=>{binding=cb;},addInitScript:async()=>{},goto:async()=>{binding(null,{kind:'browser-receipt',subscription:1,data:'bad'});signal.abort();}};
 const browser={newContext:async()=>({newPage:async()=>page}),close:async()=>{throw new Error('close failed');}};
 const result=await observeBrowser({endpoint:'http://localhost',source:'demo',signal:signal.signal,onObservation:()=>{},outputDirectory:'.',testHooks:{chromium:{launch:async()=>browser}}});
 assert.equal(result.check.status,'failed'); assert.match(result.check.reason,/Malformed/); assert.match(result.cleanupError,/close failed/);
});
