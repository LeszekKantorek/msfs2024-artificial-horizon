import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';

test('session forwards launch and context options and closes once', async () => {
 const {openBrowserSession}=await import('./session.mjs'); let options, contextOptions, closes=0; const page={};
 const browser={newContext:async o=>(contextOptions=o,{newPage:async()=>page}),close:async()=>closes++};
 const session=await openBrowserSession({viewport:{width:320,height:480},hasTouch:true,chromium:{launch:async o=>(options=o,browser)}});
 assert.equal(options.headless,true); assert.equal(options.timeout,5000); assert.deepEqual(contextOptions,{viewport:{width:320,height:480},hasTouch:true}); assert.equal(session.page,page);
 await session.close(); await session.close(); assert.equal(closes,1);
});
test('cancelled launch closes its late handle', async () => {
 const {openBrowserSession}=await import('./session.mjs'); let finish,closed=false; const controller=new AbortController();
 const result=openBrowserSession({signal:controller.signal,chromium:{launch:()=>new Promise(r=>finish=r)}}); controller.abort(); await assert.rejects(result,error=>{assert.match(error.message,/cancel/i);assert.match(error.cleanupError,/cleanup unconfirmed/);return true;}); finish({close:async()=>closed=true}); await new Promise(r=>setImmediate(r)); assert.equal(closed,true);
});
test('screenshot creates parent and writes only when requested', async () => {
 const {saveScreenshot}=await import('./session.mjs'); const dir=await mkdtemp(path.join(tmpdir(),'browser-session-')); const target=path.join(dir,'nested','image.png');
 try {await saveScreenshot({screenshot:async ({path,timeout})=>{assert.equal(timeout,5000); await (await import('node:fs/promises')).writeFile(path,'image');}},{path:target}); assert.equal(await readFile(target,'utf8'),'image');} finally {await rm(dir,{recursive:true,force:true});}
});
test('missing module is explicit and channel reaches launcher', async () => {
 const {openBrowserSession}=await import('./session.mjs'); const previous=process.env.PLAYWRIGHT_MODULE, channel=process.env.BROWSER_CHANNEL;
 try {process.env.PLAYWRIGHT_MODULE='missing-playwright-for-session';await assert.rejects(openBrowserSession(),/PLAYWRIGHT_MODULE/);process.env.BROWSER_CHANNEL='test-channel';let launch; const session=await openBrowserSession({chromium:{launch:async opts=>(launch=opts,{newContext:async()=>({newPage:async()=>({})}),close:async()=>{}})}});assert.equal(launch.channel,'test-channel');await session.close();}
 finally {if(previous===undefined)delete process.env.PLAYWRIGHT_MODULE;else process.env.PLAYWRIGHT_MODULE=previous;if(channel===undefined)delete process.env.BROWSER_CHANNEL;else process.env.BROWSER_CHANNEL=channel;}
});
test('late launch cleanup failure is reported to the caller', async () => {
 const {openBrowserSession}=await import('./session.mjs');let finish;const controller=new AbortController();
 const result=openBrowserSession({signal:controller.signal,chromium:{launch:()=>new Promise(r=>finish=r)}}); controller.abort(); finish({close:async()=>{throw new Error('late close failed');}});
 await assert.rejects(result,error=>{assert.match(error.cleanupError ?? '',/late close failed/);return true;});
});
test('process cleanup runs when browser close fails and preserves primary failure', async () => {
 const {cleanupBrowserAndProcess}=await import('./session.mjs'); let stopped=false; const messages=[];
 const primary=new Error('assertion failed');
 await cleanupBrowserAndProcess({session:{close:async()=>{throw new Error('close failed');}},stop:async()=>{stopped=true;},primaryError:primary,reportError:e=>messages.push(e.message)});
 assert.equal(stopped,true); assert.deepEqual(messages,['close failed']);
 await assert.rejects(cleanupBrowserAndProcess({session:{close:async()=>{throw new Error('close failed');}},stop:async()=>{}}),/cleanup/i);
});
