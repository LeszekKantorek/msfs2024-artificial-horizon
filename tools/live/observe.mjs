import { TextDecoder, isDeepStrictEqual } from 'node:util';
import { open } from 'node:fs/promises';
import { setTimeout as delay } from 'node:timers/promises';
import { isValidSnapshot } from '../../web/telemetry/validation.js';

export class SseParser {
  constructor(onEvent) { this.onEvent = onEvent; this.decoder = new TextDecoder('utf-8', {fatal:true}); this.buffer = ''; this.event = ''; this.data = []; this.pending = false; this.bytes=0; }
  push(bytes) { this.buffer += this.decoder.decode(bytes, {stream:true}); this.lines(); if (this.bytes + Buffer.byteLength(this.buffer) > 8 * 1024 * 1024) throw new Error('SSE text limit reached'); }
  lines() {
    while (true) {
      const m = /\r\n|\r(?!$)|\n/.exec(this.buffer); if (!m) break;
      const line = this.buffer.slice(0,m.index); this.buffer = this.buffer.slice(m.index+m[0].length); this.bytes += Buffer.byteLength(line)+Buffer.byteLength(m[0]); if(this.bytes > 8 * 1024 * 1024) throw new Error('SSE text limit reached');
      if (!line) { if (this.data.length) this.onEvent({event:this.event || 'message', data:this.data.join('\n')}); this.event=''; this.data=[]; this.pending=false; this.bytes=0; continue; }
      if (line.startsWith(':')) continue;
      const index=line.indexOf(':'); const key=index < 0 ? line : line.slice(0,index); let value=index < 0 ? '' : line.slice(index+1); if (value.startsWith(' ')) value=value.slice(1);
      if (key === 'event') { this.event=value; this.pending=true; } else if (key === 'data') { this.data.push(value); this.pending=true; }
    }
  }
  finish() { this.buffer += this.decoder.decode(); if (this.buffer.endsWith('\r')) this.buffer += '\n'; this.lines(); if (this.buffer || this.pending) throw new Error('Partial SSE frame'); }
}
export function evaluateSnapshots(samples, source) {
  let previous = -1, live = 0;
  for (const s of samples) {
    if (!isValidSnapshot(s)) return {status:'failed', reason:'Malformed telemetry envelope'};
    if (s.source !== source) return {status:'failed', reason:'Source mismatch'};
    if (s.sequence <= previous) return {status:'failed', reason:'Sequence regression'};
    previous=s.sequence;
    if (s.state === 'live') { if (s.sample_age_ms >= 1000) return {status:'failed', reason:'Stale live sample'}; live++; }
  }
  return live >= 2 && samples.at(-1)?.state === 'live' ? {status:'passed', reason:'Fresh advancing samples'} : {status:'unavailable', reason:'Fresh advancing samples not observed at end of window'};
}
export function evaluateWindow(records, source, now) {
  const result = evaluateSnapshots(records.map(r => r.snapshot), source);
  const last = records.at(-1);
  if (result.status === 'passed' && last.snapshot.state === 'live' && last.snapshot.sample_age_ms + now - last.receivedMonotonicMs >= 1000) return {status:'failed', reason:'Live sample expired without a fresh update'};
  return result;
}
export function evaluateSubscriptions(records, source, now) {
  const groups = new Map();
  for (const record of records) { if (!groups.has(record.subscription)) groups.set(record.subscription, []); groups.get(record.subscription).push(record); }
  const values = [...groups.values()];
  const results = values.map((group, index) => now !== undefined && index === values.length - 1 ? evaluateWindow(group, source, now) : evaluateSnapshots(group.map(r=>r.snapshot), source));
  return results.find(r => r.status === 'failed') ?? results.at(-1) ?? {status:'unavailable',reason:'Fresh advancing samples not observed'};
}
export class StreamValidator {
  constructor(source) { this.source=source; this.subscription=undefined; this.previous=-1; this.live=0; this.last=null; this.failure=null; this.count=0; }
  add(record) {
    this.count++;
    if(record.subscription!==this.subscription) { this.subscription=record.subscription; this.previous=-1; this.live=0; }
    const s=record.snapshot;
    if(!isValidSnapshot(s)) this.failure ??= {status:'failed',reason:'Malformed telemetry envelope'};
    else if(s.source!==this.source) this.failure ??= {status:'failed',reason:'Source mismatch'};
    else if(s.sequence<=this.previous) this.failure ??= {status:'failed',reason:'Sequence regression'};
    else if(s.state==='live' && s.sample_age_ms>=1000) this.failure ??= {status:'failed',reason:'Stale live sample'};
    if(isValidSnapshot(s)) {this.previous=s.sequence; if(s.state==='live') this.live++;}
    this.last=record;
  }
  result(now) {
    if(this.failure) return this.failure;
    if(this.live<2 || this.last?.snapshot?.state!=='live') return {status:'unavailable',reason:'Fresh advancing samples not observed at end of window'};
    if(this.last.snapshot.sample_age_ms+now-this.last.receivedMonotonicMs>=1000) return {status:'failed',reason:'Live sample expired without a fresh update'};
    return {status:'passed',reason:'Fresh advancing samples'};
  }
}
export function fieldAvailability(samples) {
  const live=samples.filter(s=>s.state==='live');
  return Object.fromEntries(['ias_kt','altitude_ft'].map(field=> {
    const count=live.filter(s=>Number.isFinite(s[field]) && (field!=='ias_kt' || s[field]>=0)).length;
    return [field,{status:count?'passed':'unavailable',reason:`${count}/${live.length} live samples have ${field}; absence alone does not establish native support`}];
  }));
}
export class JournalReader {
  constructor() { this.decoder=new TextDecoder('utf-8',{fatal:true}); this.buffer=''; this.records=[]; this.sessionId=null; this.offset=0; }
  push(bytes) {
    this.offset+=bytes.length; if(this.offset>32*1024*1024) throw new Error('Diagnostic byte limit reached');
    this.buffer+=this.decoder.decode(bytes,{stream:true});
    let at; while((at=this.buffer.indexOf('\n'))>=0) {
      const line=this.buffer.slice(0,at); this.buffer=this.buffer.slice(at+1); if(!line.trim()) continue;
      const r=JSON.parse(line); if(!this.sessionId) { if(r.kind!=='session_start' || r.data?.schema_version!==5 || !r.session_id) throw new Error('Unsupported diagnostic schema or session'); this.sessionId=r.session_id; }
      if(r.session_id!==this.sessionId || r.record!==this.records.length) throw new Error('Diagnostic session or record discontinuity');
      this.records.push(r);
    }
  }
  finish() { this.buffer+=this.decoder.decode(); return {sessionId:this.sessionId,records:this.records,complete:!this.buffer && this.records.at(-1)?.kind==='session_end'}; }
  async readAvailable(filename) {
    let file; try { file=await open(filename,'r'); } catch(e) { if(e.code==='ENOENT') return; throw e; }
    try { const info=await file.stat(); if(info.size<this.offset) throw new Error('Diagnostic file was truncated');
      while(this.offset<info.size) { const bytes=Buffer.alloc(Math.min(65536,info.size-this.offset)); const {bytesRead}=await file.read(bytes,0,bytes.length,this.offset); if(!bytesRead) break; this.push(bytes.subarray(0,bytesRead)); }
    } finally { await file.close(); }
  }
}
export function parseJournal(text) {
  const reader=new JournalReader(); reader.push(Buffer.from(text)); return reader.finish();
}
export async function observeJournal(filename, reader, signal) {
  while(!signal.aborted) { await reader.readAvailable(filename); try { await delay(200,undefined,{signal}); } catch(e) { if(!signal.aborted) throw e; } }
}
export function evaluateJournal(journal) {
  if (!journal.complete) return {status:'inconclusive',reason:'Diagnostic session is incomplete'};
  const end=journal.records.at(-1);
  if (end?.data?.success === false) return {status:'failed',reason:end.data.error ?? 'Acquisition failed'};
  if (end?.data?.success === true) return {status:'passed',reason:'Diagnostic session completed'};
  return {status:'inconclusive',reason:'Diagnostic outcome is missing'};
}
export function correlatePublications(journal, samples) {
  const publications = new Map(journal.records.filter(r => r.kind === 'publication').map(r => [r.data.snapshot.sequence, r.data.snapshot]));
  let matched = 0;
  for (const sample of samples) {
    const publication = publications.get(sample.sequence);
    if (!publication) continue;
    if (publication.state === 'live' && sample.state === 'stale' && sample.sample_age_ms >= 1000) continue;
    // Age is calculated separately at each serialization.
    const {sample_age_ms: ignored, ...actual} = sample;
    const {sample_age_ms: ignoredPublication, ...expected} = publication;
    if (!isDeepStrictEqual(actual, expected)) return {status:'failed', reason:`Publication ${sample.sequence} differs from SSE`};
    matched++;
  }
  return {status:journal.complete && samples.length > 0 && matched === samples.length ? 'passed' : 'inconclusive', reason:`${matched}/${samples.length} SSE sequences matched`};
}
export async function observeSse({endpoint, source, signal, onObservation}) {
  const validator=new StreamValidator(source); const response=await fetch(`${endpoint}/api/v1/events`, {signal}); if (!response.ok || !response.body) throw new Error(`SSE HTTP ${response.status}`);
  const parser=new SseParser(e => { if (e.event !== 'telemetry') return; let snapshot; try { snapshot=JSON.parse(e.data); } catch { throw new Error('Malformed telemetry JSON'); } const record={kind:'sse', receivedMonotonicMs:performance.now(), snapshot}; validator.add(record); onObservation(record); });
  try { for await (const chunk of response.body) parser.push(chunk); parser.finish(); } catch (error) { if (!signal?.aborted) throw error; }
  const result=validator.result(performance.now());
  return {check:result.status==='failed' ? result : !signal?.aborted ? {status:'inconclusive',reason:'SSE stream ended before observation completed'} : result,count:validator.count};
}
export async function waitForReadiness({endpoint,source,timeoutMs,signal,onObservation}) {
  const local=new AbortController(); const abort=()=>local.abort();
  signal?.addEventListener('abort',abort,{once:true}); if(signal?.aborted) abort();
  let ready=false; const validator=new StreamValidator(source); const timer=setTimeout(abort,timeoutMs);
  try {
    await observeSse({endpoint,source,signal:local.signal,onObservation:record=>{
      validator.add(record); onObservation({...record,kind:'readiness-sse'});
      const result=validator.result(performance.now());
      if(result.status==='failed') throw new Error(result.reason);
      if(result.status==='passed') {ready=true; local.abort();}
    }});
    return ready ? {status:'passed',reason:'Fresh advancing telemetry observed'} : {status:'unavailable',reason:'Readiness deadline elapsed without fresh advancing telemetry'};
  } catch(error) { if(local.signal.aborted && !ready) return {status:'unavailable',reason:'Readiness interrupted or timed out'}; throw error; }
  finally {clearTimeout(timer); signal?.removeEventListener('abort',abort);}
}
