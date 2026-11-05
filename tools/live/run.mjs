import { spawn, execFileSync } from 'node:child_process';
import { mkdir, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { randomUUID } from 'node:crypto';
import { launchOwned } from './process.mjs';
import { observeSse, waitForReadiness, JournalReader, observeJournal, correlatePublications, evaluateJournal, fieldAvailability } from './observe.mjs';
import { observeBrowser } from './browser.mjs';
import { writeReport } from './report.mjs';
export function parseArgs(args) {
 const values={}; const names=new Set(['source','dll','exe','duration-ms','ready-timeout-ms','port']);
 for(let i=0;i<args.length;i+=2) { const k=args[i]?.replace(/^--/,''); if (!args[i]?.startsWith('--') || !names.has(k) || !args[i+1] || values[k]!==undefined) throw new Error(`Invalid option ${args[i]}`); values[k]=args[i+1]; }
 if(!['demo','simconnect'].includes(values.source)) throw new Error('Explicit --source demo or simconnect is required');
 for(const k of ['exe','dll']) if(values[k] && !path.isAbsolute(values[k])) throw new Error(`--${k} must be absolute`);
 if(values.dll && values.source!=='simconnect') throw new Error('--dll requires simconnect');
 function number(k,def,max) { if(values[k]===undefined)return def; if(!/^\d+$/.test(values[k]) || +values[k]<1 || +values[k]>max) throw new Error(`Invalid --${k}`); return +values[k]; }
 return {source:values.source,exe:values.exe,dll:values.dll,durationMs:number('duration-ms',30000,300000),readyTimeoutMs:number('ready-timeout-ms',30000,300000),port:number('port',undefined,65535)};
}
export function boundedText(limit) { const chunks=[]; let size=0,truncated=false; return {add(chunk) {const remaining=Math.max(0,limit-size); if(chunk.length>remaining) truncated=true; if(remaining) chunks.push(chunk.subarray(0,remaining)); size+=chunk.length;},text:()=>Buffer.concat(chunks).toString('utf8'),truncated:()=>truncated}; }
export function boundedObservations(limit,onLimit,byteLimit=8388608) {
 const observations=[]; let observed=0,bytes=0;
 const size=value=>Buffer.byteLength(JSON.stringify(value)+'\n');
 return {observations,add(value) { observed++; const incoming=size(value); if(incoming>byteLimit) return;
  while(observations.length && (observations.length>=limit || bytes+incoming>byteLimit)) {const kept=observations.filter((_,i)=>i%2===0); if(kept.length===observations.length) kept.shift(); observations.splice(0,observations.length,...kept); bytes=observations.reduce((n,item)=>n+size(item),0);}
  observations.push(value); bytes+=incoming;
 },truncated:()=>false,counts:()=>({observed,retained:observations.length,discarded:observed-observations.length})};
}
function build(signal, directory) { return new Promise((resolve,reject) => {
 const child=spawn('cargo',['build','--locked','--release','--target','x86_64-pc-windows-msvc'],{windowsHide:true,signal,env:{...process.env,CARGO_TARGET_DIR:path.resolve('target')}}); const out=boundedText(8388608),err=boundedText(8388608);
 child.stdout.on('data',c=>out.add(c)); child.stderr.on('data',c=>err.add(c));
 const timer=setTimeout(()=>child.kill(),600000);
 child.once('error',reject); child.once('close',async code=>{ clearTimeout(timer); try { await writeFile(path.join(directory,'build-stdout.txt'),out.text()); await writeFile(path.join(directory,'build-stderr.txt'),err.text()); if(out.truncated() || err.truncated()) reject(new Error('Build output limit reached')); else if(code!==0) reject(new Error(`Build failed or timed out: ${code}`)); else resolve(); } catch(e){reject(e);} });
 }); }
function git(args) { try { return execFileSync('git',args,{encoding:'utf8'}).trim(); } catch { return null; } }
const check=(id,status,observed,limitations=[])=>({id,status,expected:id,observed,evidence:[],limitations});
export function observationErrorStatus(error) { return /Malformed telemetry JSON|Malformed telemetry envelope|Partial SSE frame|Source mismatch|Sequence regression|Stale live sample/.test(error.message) ? 'failed' : 'inconclusive'; }
export function runFailureCheck(error,started) { return check(started?'observation':'startup',started?'inconclusive':'failed',error.message); }
export async function run(options) {
 const runId=randomUUID(), directory=path.resolve('.local/live',runId); await mkdir(directory,{recursive:true});
 const controller=new AbortController(); let cancelled=false; const cancel=()=>{cancelled=true;controller.abort();}; process.on('SIGINT',cancel);
 const checks=[]; let cleanup={graceful:false,forced:false,exitCode:null,error:'Application did not start'}, app, journalError;
 const journalReader=options.source==='simconnect' ? new JournalReader() : null;
 const logPath=journalReader ? path.join(directory,'diagnostic.jsonl') : undefined;
 const manifest={schemaVersion:1,runId,source:options.source,configuration:options,startedAt:new Date().toISOString(),revision:git(['rev-parse','HEAD']),dirty:!!git(['status','--porcelain']),platform:process.platform,node:process.version,buildProvenance:options.exe?'external/unverified':'local build'};
 const retained=boundedObservations(10000,()=>controller.abort()); const observations=retained.observations; const record=o=>retained.add(o);
 try {
  let exe=options.exe; if(!exe){await build(controller.signal,directory);exe=path.resolve('target/x86_64-pc-windows-msvc/release/msfs2024-artificial-horizon.exe');}
  const startup = new AbortController();
  const abortStartup = () => startup.abort();
  controller.signal.addEventListener('abort', abortStartup, {once:true});
  try { app=await launchOwned({...options,readyTimeoutMs:5000,exe,runDirectory:directory,logPath,onOutputLimit:()=>controller.abort()},startup.signal); }
  finally { controller.signal.removeEventListener('abort', abortStartup); }
  Object.assign(manifest,{pid:app.pid,childStartedAt:app.startedAt,invocation:app.invocation,endpoint:app.endpoint,artifact:app.artifact});
  if (!options.exe) manifest.artifact.attribution='local build';
  const journalTask=journalReader ? observeJournal(logPath,journalReader,controller.signal).catch(e=>{journalError=e;}) : Promise.resolve();
  try { const readiness=await waitForReadiness({endpoint:app.endpoint,source:options.source,timeoutMs:options.readyTimeoutMs,signal:controller.signal,onObservation:record}); checks.push(check('readiness',readiness.status,readiness.reason)); }
  catch(e) { checks.push(check('readiness',observationErrorStatus(e),e.message)); }
  if (controller.signal.aborted) { await journalTask; throw new Error('Observation stopped during readiness'); }
  const observationStarted=performance.now(); const timer=setTimeout(()=>controller.abort(),options.durationMs);
  const tasks=[observeSse({endpoint:app.endpoint,source:options.source,signal:controller.signal,onObservation:record}),observeBrowser({endpoint:app.endpoint,source:options.source,signal:controller.signal,onObservation:record})];
  const results=await Promise.allSettled(tasks); manifest.observationDurationMs=performance.now()-observationStarted; clearTimeout(timer); controller.abort();
  await journalTask;
  for(const [i,result] of results.entries()) {const id=i?'browser receipt':'SSE telemetry'; if(result.status==='fulfilled') checks.push(check(id,result.value.check.status,result.value.check.reason,[])); else checks.push(check(id,observationErrorStatus(result.reason),result.reason.message)); const cleanupError=result.status==='fulfilled' ? result.value.cleanupError : result.reason.cleanupError; if(i && cleanupError) checks.push(check('browser cleanup','inconclusive',cleanupError));}
  for (const [field, result] of Object.entries(fieldAvailability(observations.filter(o=>o.kind==='sse').map(o=>o.snapshot)))) checks.push(check(`${field} availability`,result.status,result.reason));
 } catch(error) { cleanup=error.cleanup ?? cleanup; checks.push(runFailureCheck(error,!!app)); }
 finally { if(app) cleanup=await app.stop(); process.removeListener('SIGINT',cancel); }
 manifest.observationCounts=retained.counts();
 if(retained.counts().discarded) checks.push(check('observation sampling','inconclusive',`${retained.counts().discarded} observations discarded; correlation covers retained samples only`));
 if(retained.truncated()) checks.push(check('observation retention','inconclusive','Observation cap reached'));
 if(cleanup.stdout?.truncated || cleanup.stderr?.truncated || cleanup.stdout?.error || cleanup.stderr?.error) checks.push(check('child output','inconclusive','Child output incomplete'));
 if(manifest.artifact?.changed) checks.push(check('artifact identity','inconclusive','Executable changed during startup'));
 if(journalReader) {try {if(journalError) throw journalError; await journalReader.readAvailable(logPath); const journal=journalReader.finish(); manifest.diagnosticSessionId=journal.sessionId; const outcome=evaluateJournal(journal); checks.push(check('acquisition',outcome.status,outcome.reason)); for(const [kind,id] of [['sse','diagnostic to SSE'],['browser-receipt','diagnostic to browser receipt']]) { const correlation=correlatePublications(journal,observations.filter(o=>o.kind===kind).map(o=>o.snapshot)); checks.push(check(id,correlation.status,correlation.reason,['Raw acquisition correlation remains time-bounded.', ...(retained.counts().discarded ? ['Correlation covers retained samples only.'] : [])])); }}catch(e){checks.push(check('diagnostic correlation','inconclusive',e.message));}}
 try { const result=await writeReport({manifest,observations,checks,cleanup,cancelled},directory); console.log(`Results: ${directory}`); return result.exitCode; } catch(e) { console.error(`Report failed in ${directory}: ${e.message}`); return cancelled?130:1; }
}
if(process.argv[1] && import.meta.url===pathToFileURL(path.resolve(process.argv[1])).href) { try {process.exitCode=await run(parseArgs(process.argv.slice(2)));} catch(e){console.error(e.message);process.exitCode=1;} }
