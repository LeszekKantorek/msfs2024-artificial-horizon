import {spawn} from 'node:child_process';
import {StringDecoder} from 'node:string_decoder';

const wait = (promise, ms) => new Promise(resolve => { const timer = setTimeout(() => resolve(false), ms); promise.then(() => {clearTimeout(timer); resolve(true);}); });
export async function launchOwned({exe, appArgs, url}, signal) {
  const target = new URL(url);
  const child = spawn(exe, appArgs, {shell:false, windowsHide:true, stdio:['pipe','pipe','pipe']});
  let exit, stdinError, stopped;
  child.stdin.on('error', error => { stdinError = error.message; });
  const exited = new Promise(resolve => {
    child.once('error', error => resolve(exit = {code:null,error:error.message}));
    child.once('exit', (code, sig) => resolve(exit = {code,signal:sig}));
  });
  child.stderr.pipe(process.stderr, {end:false});
  const stop = () => stopped ??= (async () => {
    let error = null, forced = false;
    if (!exit) {
      try { child.stdin.end('stop\n'); } catch (e) { error = e.message; }
      if (!await wait(exited, 10000)) {
        forced = true;
        if (!child.kill()) error = 'Failed to terminate owned child';
        if (!await wait(exited, 5000)) error = 'Owned child did not exit';
      }
    }
    return {exitCode:exit?.code ?? null,forced,error:error ?? stdinError ?? exit?.error ?? null};
  })();
  try {
    await new Promise((resolve,reject) => {
      let pending = ''; const decoder = new StringDecoder('utf8');
      const cleanup = () => { child.stdout.off('data', onData); signal?.removeEventListener('abort', onAbort); };
      const fail = e => {cleanup(); reject(e);};
      const onAbort = () => fail(new DOMException('Startup interrupted','AbortError'));
      const onData = chunk => {
        process.stdout.write(chunk); pending += decoder.write(chunk);
        let index; while ((index = pending.indexOf('\n')) !== -1) {
          const line = pending.slice(0,index).trim(); pending = pending.slice(index+1);
          const match = /^Listening on (\S+)$/.exec(line);
          if (match) {
            const expected = `${target.hostname}:${target.port || '80'}`;
            if (match[1] !== expected) return fail(new Error(`Listening address ${match[1]} does not match ${expected}`));
            cleanup(); child.stdout.pipe(process.stdout,{end:false}); resolve(); return;
          }
        }
        if (pending.length > 8192) pending = pending.slice(-8192);
      };
      child.stdout.on('data',onData); exited.then(() => fail(new Error(`Application exited during startup: ${exit?.error ?? exit?.code}`)));
      signal?.addEventListener('abort',onAbort,{once:true}); if (signal?.aborted) onAbort();
    });
    return {stop};
  } catch (error) { error.cleanup = await stop(); throw error; }
}
