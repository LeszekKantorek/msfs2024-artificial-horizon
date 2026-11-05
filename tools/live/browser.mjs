import { setTimeout as delay } from 'node:timers/promises';
import { StreamValidator } from './observe.mjs';
import { abortable, openBrowserSession } from '../browser/session.mjs';
export async function observeBrowser({endpoint,source,signal,onObservation,testHooks}) {
 const session=await openBrowserSession({signal,chromium:testHooks?.chromium});
 const validator=new StreamValidator(source); let pageError=null, outcome, failure, cleanupError;
 try {
  const {page}=session;
  page.on('pageerror',error=>{pageError=error.message;});
  await abortable(page.exposeBinding('__liveObserved',(_source,record)=>{record.receivedMonotonicMs=performance.now(); if(record.kind==='browser-receipt'){try{record.snapshot=JSON.parse(record.data);}catch{record.snapshot=null;} validator.add(record);} onObservation(record);}),signal);
  await abortable(page.addInitScript(()=>{const Native=window.EventSource; let next=0; window.EventSource=class extends Native {constructor(...args){super(...args); const subscription=++next; this.addEventListener('telemetry',e=>window.__liveObserved({kind:'browser-receipt',subscription,data:e.data,browserTime:performance.now()}));}};}),signal);
  await abortable(page.goto(endpoint,{waitUntil:'domcontentloaded',timeout:10000}),signal);
  while(!signal.aborted) {try{await delay(200,undefined,{signal});}catch(error){if(!signal.aborted)throw error;}}
  const result=validator.result(performance.now());
  outcome={check:result.status==='failed'?result:pageError?{status:'inconclusive',reason:`Browser error: ${pageError}`}:result,receiptCount:validator.count};
 } catch(error) {const result=validator.result(performance.now()); if(result.status==='failed') outcome={check:result,receiptCount:validator.count}; else failure=error;}
 finally {try{await session.close();}catch(error){cleanupError=error.message;}}
 if(failure){if(cleanupError)failure.cleanupError=cleanupError;throw failure;}
 return {...outcome,cleanupError};
}
