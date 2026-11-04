import test from 'node:test';
import assert from 'node:assert/strict';
import { createSocket } from 'node:dgram';
import { MdnsFixture } from './webrtc_mdns_fixture.mjs';

const name=value=>Buffer.concat([...value.split('.').map(s=>Buffer.concat([Buffer.from([s.length]),Buffer.from(s)])),Buffer.from([0])]);
test('mDNS fixture maps numeric hosts, preserves browser names, and answers one-shot queries',async()=>{
  const fixture=new MdnsFixture(),client=createSocket('udp4');
  try {
    await fixture.start('127.0.0.1');
    const original={candidate:{candidate:'candidate:f 1 udp 123456 127.0.0.1 40000 typ host',sdpMLineIndex:0}};
    const mapped=fixture.rewrite(original),host=mapped.candidate.candidate.split(' ')[4];
    assert.match(host,/^[\da-f]{8}-[\da-f]{4}-4[\da-f]{3}-[89ab][\da-f]{3}-[\da-f]{12}\.local$/);
    assert.equal(fixture.rewrite(original).candidate.candidate,mapped.candidate.candidate);
    assert.match(original.candidate.candidate,/127\.0\.0\.1/);
    const browser={candidate:{candidate:'candidate:f 1 udp 123456 12345678-1234-4234-8234-123456789abc.local 40000 typ host'}};
    assert.deepEqual(fixture.rewrite(browser),browser);
    await new Promise(resolve=>client.bind(0,'127.0.0.1',resolve));client.setMulticastInterface('127.0.0.1');client.setMulticastLoopback(true);
    const header=Buffer.alloc(12);header.writeUInt16BE(123,0);header.writeUInt16BE(1,4);
    const packet=Buffer.concat([header,name(host),Buffer.from([0,1,0,1])]);
    const response=new Promise((resolve,reject)=>{const timer=setTimeout(()=>reject(Error('mDNS fixture deadline')),1000);client.once('message',(data,from)=>{clearTimeout(timer);resolve({data,from});});});
    client.send(packet,5353,'224.0.0.251');const {data,from}=await response;
    assert.equal(from.port,5353);assert.equal(data.readUInt16BE(0),123);assert.equal(data.readUInt16BE(2),0x8400);
    assert.deepEqual(data.subarray(-4),Buffer.from([127,0,0,1]));
    assert.deepEqual(fixture.snapshot(),{queries:1,answers:1,held:0,controlledNames:1,browserNames:1,aliasedBrowserNames:0});
    assert.deepEqual(fixture.rewrite(browser,true),browser,'unknown socket family stays untouched');
    fixture.ipv4Ports.add(40000);
    const controlled=fixture.rewrite(browser,true);
    assert.notEqual(controlled.candidate.candidate,browser.candidate.candidate);
    assert.equal(fixture.rewrite(browser,true).candidate.candidate,controlled.candidate.candidate);
    assert.equal(fixture.snapshot().aliasedBrowserNames,1);
  } finally {client.close();await fixture.close();}
});
