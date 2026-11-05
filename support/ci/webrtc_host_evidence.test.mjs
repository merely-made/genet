import test from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import vm from 'node:vm';
const context=vm.createContext({});vm.runInContext(await readFile(new URL('./webrtc_interop_peer.js',import.meta.url),'utf8'),context);
const evidence=(stats)=>JSON.parse(JSON.stringify(context.rtcRemoteHostEvidence(stats,'private.local','192.0.2.1',40000)));
test('host evidence excludes reflexive and local candidates and wrong endpoints',()=>{
  assert.deepEqual(evidence([{type:'remote-candidate',candidateType:'prflx',address:'192.0.2.1',port:40000},{type:'local-candidate',candidateType:'host',address:'private.local',port:40000}]),{hosts:0,samePort:0,named:0,numericEndpoint:0});
  assert.deepEqual(evidence([{type:'remote-candidate',candidateType:'host',address:'private.local',port:40001}]),{hosts:1,samePort:0,named:0,numericEndpoint:0});
});
test('resolved named and numeric host observations yield counts without addresses',()=>{
  const view=evidence(['private.local','192.0.2.1'].map(address=>({type:'remote-candidate',candidateType:'host',address,port:40000})));
  assert.deepEqual(view,{hosts:2,samePort:2,named:1,numericEndpoint:1});
  assert(!JSON.stringify(view).includes('private'));assert(!JSON.stringify(view).includes('192.0.2.1'));
});
