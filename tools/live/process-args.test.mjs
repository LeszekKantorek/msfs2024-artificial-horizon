import test from 'node:test';
import assert from 'node:assert/strict';
import {parseArgs} from './run.mjs';
const base=['--exe','app','--url','http://127.0.0.1:8080/events','--output','out'];
test('accepts both limits and passes app arguments unchanged',()=>{assert.deepEqual(parseArgs([...base,'--duration-seconds','30','--max-messages','12','--','--source','demo']),{exe:'app',url:base[3],output:'out',durationSeconds:30,maxMessages:12,appArgs:['--source','demo']});});
test('rejects missing, duplicate, invalid and legacy options',()=>{for(const args of [[],base,[...base,'--source','demo'],[...base,'--max-messages','0'],[...base,'--duration-seconds','2147484'],[...base,'--max-messages','2','--max-messages','3'],['--exe','app','--url','http://localhost/','--output','x','--max-messages','1']]) assert.throws(()=>parseArgs(args));});
