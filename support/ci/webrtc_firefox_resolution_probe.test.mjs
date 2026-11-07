import test from 'node:test';
import assert from 'node:assert/strict';
import {resolutionAnswer,currentNativeCandidate} from './webrtc_firefox_resolution_probe.mjs';
import {observeNames} from './webrtc_mdns_observer.mjs';
test('controlled answer is a bounded authoritative positive owned A record',()=>{
  const name='53caef04-4218-4fb0-815d-f91a511bb8d2.local';const answer=resolutionAnswer(name,'192.0.2.1');
  assert(answer.length<128);assert.deepEqual(observeNames(answer,new Set([name]),'192.0.2.1'),{queries:0,answers:1});
  assert.equal(answer.readUInt16BE(4),0);assert.equal(answer.readUInt16BE(6),1);
  assert.throws(()=>resolutionAnswer('unowned.local','192.0.2.1'));assert.throws(()=>resolutionAnswer(name,'::1'));
});

test('replay extracts the current host and generation without constructing regex source',()=>{
  const candidate='candidate:owned 1 udp 123 53caef04-4218-4fb0-815d-f91a511bb8d2.local 50000 typ host';
  assert.deepEqual(currentNativeCandidate('v=0\r\na=ice-ufrag:fresh\r\na='+candidate+'\r\n'),{candidate,sdpMLineIndex:0,usernameFragment:'fresh'});
  assert.throws(()=>currentNativeCandidate('v=0\r\na=ice-ufrag:fresh\r\n'),/lacks a current native host/);
});
