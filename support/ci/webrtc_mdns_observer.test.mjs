import test from 'node:test';
import assert from 'node:assert/strict';
import { observeNames } from './webrtc_mdns_observer.mjs';
import { MdnsFixture } from './webrtc_mdns_fixture.mjs';
const name='53caef04-4218-4fb0-815d-f91a511bb8d2.local';
const labels=Buffer.concat([...name.split('.').map(s=>Buffer.concat([Buffer.from([s.length]),Buffer.from(s)])),Buffer.from([0])]);
function query(){const h=Buffer.alloc(12);h.writeUInt16BE(1,4);return Buffer.concat([h,labels,Buffer.from([0,1,0x80,1])]);}
test('observes QU A questions and compressed positive A answers without responding',()=>{
  const watched=new Set([name]);
  assert.deepEqual(observeNames(query(),watched,'192.0.2.1'),{queries:1,answers:0});
  const response=Buffer.concat([query(),Buffer.from([0xc0,12,0,1,0x80,1,0,0,0,120,0,4,192,0,2,1])]);
  response.writeUInt16BE(0x8400,2);response.writeUInt16BE(1,6);
  assert.deepEqual(observeNames(response,watched,'192.0.2.1'),{queries:0,answers:1});
  assert.equal(observeNames(response,watched,'192.0.2.2').answers,0);
  response.writeUInt32BE(0,query().length+6);
  assert.equal(observeNames(response,watched,'192.0.2.1').answers,0);
  const fixture=new MdnsFixture();fixture.watchNative({candidate:{candidate:`candidate:x 1 udp 1 ${name} 5555 typ host`}});
  assert.equal(fixture.nativeNames.has(name),true);assert.equal(fixture.names.has(name),false);
});
test('malformed, looping and oversized DNS input is ignored within bounds',()=>{
  const loop=query();loop[12]=0xc0;loop[13]=12;
  for(const p of [Buffer.alloc(0),Buffer.alloc(1501),loop,...Array.from({length:query().length},(_,n)=>query().subarray(0,n))]){
    assert.deepEqual(observeNames(p,new Set([name]),'192.0.2.1'),{queries:0,answers:0});
  }
});
test('Firefox-sized five-question batches are observed within an eight-question cap',()=>{
  const header=Buffer.alloc(12);header.writeUInt16BE(5,4);
  const packet=Buffer.concat([header,...Array.from({length:5},()=>query().subarray(12))]);
  assert.deepEqual(observeNames(packet,new Set([name]),'192.0.2.1'),{queries:5,answers:0});
  packet.writeUInt16BE(9,4);
  assert.deepEqual(observeNames(packet,new Set([name]),'192.0.2.1'),{queries:0,answers:0});
});
