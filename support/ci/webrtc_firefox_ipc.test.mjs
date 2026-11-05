import test from 'node:test';
import assert from 'node:assert/strict';
import {summarizeMdnsIpc} from './webrtc_firefox_ipc.mjs';
const row=(method,direction='sending',side='child',phase='endpoint')=>[1,{type:'IPC',messageType:'PStunAddrsRequest::Msg_'+method,direction,side,phase,secret:'private.local 192.0.2.1 pwd'}];
const process=(data,offset=0)=>({meta:{startTime:1000+offset},threads:[{markers:{schema:{startTime:0,data:1},data}}]});
test('IPC reduction separates request, completion, direction and process clocks',()=>{
  const input=process([row('QueryMDNSHostname'),row('OnMDNSQueryComplete','receiving')]);
  input.processes=[process([row('QueryMDNSHostname','receiving','parent'),row('OnMDNSQueryComplete','sending','parent'),row('Other'),row('QueryMDNSHostname','sending','child','transferStart')],10)];
  const view=summarizeMdnsIpc(input);for(const key of ['querySent','queryReceived','completionSent','completionReceived'])assert.equal(view[key],1);
  assert.deepEqual(view.events.map(e=>e.ms),[1,1,11,11]);assert(!JSON.stringify(view).includes('private.local'));assert(!JSON.stringify(view).includes('pwd'));
});
test('IPC output and traversal are bounded',()=>{
  const view=summarizeMdnsIpc(process(Array.from({length:70},()=>row('QueryMDNSHostname'))));assert.equal(view.events.length,64);assert.equal(view.omitted,6);
  assert.throws(()=>summarizeMdnsIpc({...process([]),processes:Array.from({length:33},()=>process([]))}),/process budget/);
});
