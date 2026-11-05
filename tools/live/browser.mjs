import { createRequire } from 'node:module';
import path from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { evaluateSubscriptions, StreamValidator } from './observe.mjs';
function abortable(operation, signal, onLate) {
 return new Promise((resolve,reject)=>{
  let done=false;
  const abort=()=>{ if(!done) {done=true; reject(new Error('Browser observation cancelled'));} };
  signal.addEventListener('abort',abort,{once:true});
  Promise.resolve(operation).then(value=>{if(done){onLate?.(value);return;} done=true;signal.removeEventListener('abort',abort);resolve(value);},error=>{if(done)return;done=true;signal.removeEventListener('abort',abort);reject(error);});
  if(signal.aborted) abort();
 });
}
async function closeBrowser(browser) {
 let timer;
 try { await Promise.race([browser.close(),new Promise((_,reject)=>{timer=setTimeout(()=>reject(new Error('Browser cleanup timed out')),1000);})]); }
 finally { clearTimeout(timer); }
}
export function evaluateStableDisplay(events) {
 const display=events.filter(e=>e.kind==='browser-display').at(-1); const receipts=events.filter(e=>e.kind==='browser-receipt' && e.receivedMonotonicMs <= display?.receivedMonotonicMs);
 if(receipts.length<2 || !display || receipts.at(-1).receivedMonotonicMs-receipts[0].receivedMonotonicMs<400 || display.receivedMonotonicMs<receipts.at(-1).receivedMonotonicMs) return {status:'inconclusive',reason:'No sufficiently long stable receipt window'};
 if(display.display?.available !== true) return {status:'unavailable',reason:'Instrument was not available at display observation'};
 const fields=s=>({state:s?.state,attitude:s?.attitude,ias:s?.ias_kt ?? null,altitude:s?.altitude_ft ?? null}); const expected=fields(receipts[0].snapshot);
 if(expected.state!=='live' || receipts.some(r=>JSON.stringify(fields(r.snapshot))!==JSON.stringify(expected))) return {status:'inconclusive',reason:'Received values changed during display window'};
 if(expected.attitude?.pitch_deg!==0 || expected.attitude?.roll_deg!==0 || (expected.ias!==null && (!Number.isFinite(expected.ias) || expected.ias<0 || expected.ias>=1000)) || (expected.altitude!==null && !Number.isFinite(expected.altitude))) return {status:'inconclusive',reason:'Stable display check does not cover these values'};
 const ias=expected.ias===null?'Indicated airspeed unavailable':`Indicated airspeed ${expected.ias} knots`;
 const altitude=expected.altitude===null?'Barometric altitude unavailable':`Barometric altitude ${expected.altitude} feet`;
 const d=display.display;
 return d?.airspeed?.state===(expected.ias===null?'unavailable':'available') && d?.airspeed?.label===ias && d?.altitude?.state===(expected.altitude===null?'unavailable':'available') && d?.altitude?.label===altitude && d.rotation==='rotate(0 0 0)' && d.pitch==='translate(0 0)' ? {status:'passed',reason:'Stable zero-attitude and instrument availability and labels match'} : {status:'failed',reason:'Stable display differs from received values'};
}
export function evaluateBrowserCheck(receipts,source,now,available) {
 const result=evaluateSubscriptions(receipts,source,now);
 return result.status==='passed' && available!==true ? {status:'unavailable',reason:'Instrument is not available'} : result;
}
export async function observeBrowser({endpoint, source, signal, onObservation, outputDirectory, testHooks}) {
 let chromium; try { chromium=testHooks?.chromium ?? createRequire(import.meta.url)(process.env.PLAYWRIGHT_MODULE ?? 'playwright').chromium; }
 catch (error) { throw new Error(`Playwright unavailable; set PLAYWRIGHT_MODULE to the installed module: ${error.message}`); }
 const browser=await abortable(chromium.launch({headless:true, channel:process.env.BROWSER_CHANNEL || undefined,timeout:5000}),signal,late=>{void closeBrowser(late).catch(()=>{});});
 const validator=new StreamValidator(source); let page; let pageError=null; let available=false; let outcome, failure, cleanupError;
 try {
  const context=await abortable(browser.newContext(),signal); page=await abortable(context.newPage(),signal);
  page.on('pageerror', error => { pageError=error.message; });
  await abortable(page.exposeBinding('__liveObserved', (_source, record) => { record.receivedMonotonicMs=performance.now(); if(record.kind==='browser-receipt') { try { record.snapshot=JSON.parse(record.data); } catch { record.snapshot=null; } validator.add(record); } onObservation(record); }),signal);
  await abortable(page.addInitScript(() => { const Native=window.EventSource; let next=0; window.EventSource=class extends Native { constructor(...args) { super(...args); const subscription=++next; this.addEventListener('telemetry', e => window.__liveObserved({kind:'browser-receipt',subscription,data:e.data, browserTime:performance.now()})); } }; }),signal);
  await abortable(page.goto(endpoint, {waitUntil:'domcontentloaded',timeout:10000}),signal);
  let screenshot=false;
  while (!signal.aborted) {
   const display=await abortable(page.evaluate(() => ({available:document.querySelector('.instrument')?.dataset.available==='true',...Object.fromEntries(['airspeed','altitude'].map(id => {const n=document.getElementById(id); return [id,n ? {state:n.getAttribute('data-state'),label:n.getAttribute('aria-label')} : null]; })),rotation:document.getElementById('world-rotation')?.getAttribute('transform'),pitch:document.getElementById('world-pitch')?.getAttribute('transform')})),signal);
   available=display.available;
   onObservation({kind:'browser-display',receivedMonotonicMs:performance.now(),display,limitation:'Display is not tied to an exact received sequence'});
   if(!screenshot && available) { await abortable(page.screenshot({path:path.join(outputDirectory,'browser.png'),timeout:5000}),signal); screenshot=true; }
   try { await delay(200,undefined,{signal}); } catch (error) { if(!signal.aborted) throw error; }
  }
  const result=validator.result(performance.now());
  outcome={check:result.status==='failed' ? result : pageError ? {status:'inconclusive',reason:`Browser error: ${pageError}`} : result.status==='passed' && available!==true ? {status:'unavailable',reason:'Instrument is not available'} : result,receiptCount:validator.count,renderCorrelation:'inconclusive'};
 } catch(error) {
  if(validator.failure) outcome={check:validator.failure,receiptCount:validator.count,renderCorrelation:'inconclusive'};
  else failure=error;
 } finally { try { await closeBrowser(browser); } catch(error) { cleanupError=error.message; } }
 if(failure) { if(cleanupError) failure.cleanupError=cleanupError; throw failure; }
 return {...outcome,cleanupError};
}
