import {run} from './run.mjs';
import {stat} from 'node:fs/promises';
const [exe,url,output,...appArgs]=process.argv.slice(2);
const task=run({exe,url,output,maxMessages:3,appArgs});
const timer=setInterval(async()=>{try{if((await stat(output)).size){clearInterval(timer);process.emit('SIGINT');}}catch{}},20);
process.exitCode=await task;
clearInterval(timer);
