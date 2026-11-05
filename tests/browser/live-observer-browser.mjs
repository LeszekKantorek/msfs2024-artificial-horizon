import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { observeBrowser, evaluateStableDisplay } from '../../tools/live/browser.mjs';
const server=spawn(process.execPath,['tests/browser/helpers/browser-fixture-server.mjs'],{stdio:['pipe','pipe','inherit'],env:{...process.env,PFD_FIXTURE_PORT:'0'}});
let url; try {
 url=await new Promise((resolve,reject)=>{ const timer=setTimeout(()=>reject(new Error('Fixture startup timeout')),5000); server.stdout.on('data',data=>{const match=String(data).match(/Browser fixture: (http:\/\/127\.0\.0\.1:\d+)/); if(match){clearTimeout(timer);resolve(match[1]);}}); server.once('error',reject); });
 const dir=await mkdtemp(path.join(tmpdir(),'live-browser-')); const controller=new AbortController(); const observations=[];
 try {server.stdin.write(JSON.stringify({ias_kt:123,altitude_ft:8500})+'\n'); setTimeout(()=>server.stdin.write(JSON.stringify({ias_kt:null})+'\n'),800); setTimeout(()=>controller.abort(),2200); const summary=await observeBrowser({endpoint:url,source:'demo',signal:controller.signal,onObservation:o=>observations.push(o),outputDirectory:dir});
 assert.equal(summary.check.status,'passed'); assert.ok(observations.some(o=>o.kind==='browser-receipt' && o.snapshot.altitude_ft===8500));
 assert.ok(observations.some(o=>o.kind==='browser-display' && o.display.airspeed.state==='available'));
 assert.ok(observations.some(o=>o.kind==='browser-display' && o.display.airspeed.state==='unavailable' && o.display.altitude.label==='Barometric altitude 8500 feet'));
 assert.equal(evaluateStableDisplay(observations).status,'inconclusive');
 assert.equal(summary.renderCorrelation,'inconclusive');
 const stable=[]; const second=new AbortController(); setTimeout(()=>second.abort(),1800);
 await observeBrowser({endpoint:url,source:'demo',signal:second.signal,onObservation:o=>stable.push(o),outputDirectory:dir});
 assert.equal(evaluateStableDisplay(stable).status,'passed');
 } finally {await rm(dir,{recursive:true,force:true});}
} finally {server.kill();}
