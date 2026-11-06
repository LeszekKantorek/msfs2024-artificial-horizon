import { createRequire } from 'node:module';
import { mkdir } from 'node:fs/promises';
import { dirname } from 'node:path';

async function closeBrowser(browser) {
 let timer;
 try {await Promise.race([browser.close(),new Promise((_,reject)=>{timer=setTimeout(()=>reject(new Error('Browser cleanup timed out')),1000);})]);}
 finally {clearTimeout(timer);}
}
export async function openBrowserSession({viewport,hasTouch,chromium}={}) {
 if (!chromium) {try {chromium=createRequire(import.meta.url)(process.env.PLAYWRIGHT_MODULE ?? 'playwright').chromium;} catch(error) {throw new Error(`Playwright unavailable; set PLAYWRIGHT_MODULE to the installed module: ${error.message}`);}}
 const browser=await chromium.launch({headless:true,channel:process.env.BROWSER_CHANNEL || undefined,timeout:5000});
 let closed; const close=()=>closed ??=closeBrowser(browser);
 try {
  const options={}; if(viewport!==undefined) options.viewport=viewport; if(hasTouch!==undefined) options.hasTouch=hasTouch;
  const context=await browser.newContext(options); const page=await context.newPage();
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
