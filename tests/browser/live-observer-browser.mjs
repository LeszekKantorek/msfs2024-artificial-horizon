import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { observeBrowser } from '../../tools/live/browser.mjs';
const server=spawn(process.execPath,['tests/browser/helpers/browser-fixture-server.mjs'],{stdio:['pipe','pipe','inherit'],env:{...process.env,PFD_FIXTURE_PORT:'0'}});
try {
 const url=await new Promise((resolve,reject)=>{const timer=setTimeout(()=>reject(new Error('Fixture startup timeout')),5000);server.stdout.on('data',data=>{const match=String(data).match(/Browser fixture: (http:\/\/127\.0\.0\.1:\d+)/);if(match){clearTimeout(timer);resolve(match[1]);}});server.once('error',reject);});
 async function run(change, duration=1800) {const signal=new AbortController(),observations=[]; let started=false; const safety=setTimeout(()=>signal.abort(),10000); const summary=await observeBrowser({endpoint:url,source:'demo',signal:signal.signal,onObservation:o=>{observations.push(o); if(!started){started=true; setTimeout(()=>signal.abort(),duration); if(change) setTimeout(()=>server.stdin.write(JSON.stringify(change)+'\n'),500);}}});clearTimeout(safety);return {summary,observations};}
 server.stdin.write(JSON.stringify({pitch_deg:5,roll_deg:10,ias_kt:123,altitude_ft:8500})+'\n');
 const valid=await run({pitch_deg:-8,roll_deg:20,ias_kt:null,altitude_ft:8600});
 assert.equal(valid.summary.check.status,'passed'); assert.ok(valid.observations.some(o=>o.snapshot?.ias_kt===123)); assert.ok(valid.observations.some(o=>o.snapshot?.ias_kt===null && o.snapshot?.altitude_ft===8600));
 const invalid=await run({malformed:true}); assert.equal(invalid.summary.check.status,'failed'); assert.match(invalid.summary.check.reason,/Malformed/);
 server.stdin.write(JSON.stringify({malformed:false,hold:false})+'\n');
 const stale=await run({hold:true},2000); assert.equal(stale.summary.check.status,'failed'); assert.match(stale.summary.check.reason,/expired/i);
 console.log('Live browser receipt checks passed: changing values, optional field loss, invalid and expired samples.');
} finally {server.kill();}
