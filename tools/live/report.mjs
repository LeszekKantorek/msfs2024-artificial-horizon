import { writeFile, rename } from 'node:fs/promises';
import path from 'node:path';
async function atomic(directory, name, contents) { const target=path.join(directory,name), temp=`${target}.tmp`; await writeFile(temp,contents); await rename(temp,target); }
export function exitCode(result) {
 if (result.cancelled) return 130;
 if (!result.cleanup?.graceful || result.checks.some(c => c.status === 'failed')) return 1;
 if (result.checks.some(c => c.status !== 'passed')) return 2;
 return 0;
}
export async function writeReport({manifest,observations,checks,cleanup,cancelled=false},directory) {
 const result={schemaVersion:1,manifest,checks,cleanup,cancelled}; result.exitCode=exitCode(result);
 await atomic(directory,'manifest.json',JSON.stringify(manifest,null,2)+'\n');
 await atomic(directory,'observations.jsonl',observations.map(x=>JSON.stringify(x)+'\n').join(''));
 await atomic(directory,'result.json',JSON.stringify(result,null,2)+'\n'); return result;
}
