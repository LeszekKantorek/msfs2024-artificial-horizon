import { createRequire } from 'node:module';
import { mkdir } from 'node:fs/promises';
import { dirname } from 'node:path';

export function abortable(operation, signal, onLate) {
 if (!signal) return Promise.resolve(operation);
 return new Promise((resolve,reject)=>{
  let done=false;
  const abort=()=>{if(!done){done=true;reject(new Error('Browser observation cancelled'));}};
  signal.addEventListener('abort',abort,{once:true});
  Promise.resolve(operation).then(value=>{if(done){onLate?.(value);return;} done=true;signal.removeEventListener('abort',abort);resolve(value);},error=>{if(done)return;done=true;signal.removeEventListener('abort',abort);reject(error);});
  if(signal.aborted) abort();
 });
}
async function closeBrowser(browser) {
 let timer;
 try {await Promise.race([browser.close(),new Promise((_,reject)=>{timer=setTimeout(()=>reject(new Error('Browser cleanup timed out')),1000);})]);}
 finally {clearTimeout(timer);}
}
export async function openBrowserSession({signal,viewport,hasTouch,chromium}={}) {
 if (!chromium) {try {chromium=createRequire(import.meta.url)(process.env.PLAYWRIGHT_MODULE ?? 'playwright').chromium;} catch(error) {throw new Error(`Playwright unavailable; set PLAYWRIGHT_MODULE to the installed module: ${error.message}`);}}
 const launch=Promise.resolve(chromium.launch({headless:true,channel:process.env.BROWSER_CHANNEL || undefined,timeout:5000}));
 let browser;
 try {browser=await abortable(launch,signal);}
 catch(error) {
  if(!signal?.aborted) throw error;
  let timer, timedOut=false;
  const cleanup=launch.then(handle=>handle.close());
  try {await Promise.race([cleanup,new Promise((_,reject)=>{timer=setTimeout(()=>{timedOut=true;reject(new Error('Browser launch pending; cleanup unconfirmed'));},1000);})]);}
  catch(cleanupError) {error.cleanupError=cleanupError.message;}
  finally {clearTimeout(timer); if(timedOut) void cleanup.catch(()=>{});}
  throw error;
 }
 let closed; const close=()=>closed ??=closeBrowser(browser);
 try {
  const options={}; if(viewport!==undefined) options.viewport=viewport; if(hasTouch!==undefined) options.hasTouch=hasTouch;
  const context=await abortable(browser.newContext(options),signal); const page=await abortable(context.newPage(),signal);
  return {browser,context,page,close};
 } catch(error) {try {await close();} catch(cleanup) {error.cleanupError=cleanup.message;} throw error;}
}
export async function saveScreenshot(page,{path,timeoutMs=5000}) {
 await mkdir(dirname(path),{recursive:true}); await page.screenshot({path,timeout:timeoutMs});
}
export async function cleanupBrowserAndProcess({session,stop,primaryError,reportError=console.error}) {
 const errors=[];
 try { if(session) await session.close(); } catch(error) { errors.push(error); }
 try { await stop(); } catch(error) { errors.push(error); }
 if(primaryError) { for(const error of errors) reportError(error); }
 else if(errors.length) throw new AggregateError(errors,'Browser or process cleanup failed');
}
