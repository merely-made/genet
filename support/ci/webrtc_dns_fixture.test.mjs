import test from 'node:test';
import assert from 'node:assert/strict';
import { createSocket } from 'node:dgram';
import { DnsFixture } from './webrtc_dns_fixture.mjs';

const name=value=>Buffer.concat([...value.split('.').map(s=>Buffer.concat([Buffer.from([s.length]),Buffer.from(s)])),Buffer.from([0])]);
async function query(socket,port,host,type) {
  const header=Buffer.alloc(12);header.writeUInt16BE(123,0);header.writeUInt16BE(1,4);
  const tail=Buffer.alloc(4);tail.writeUInt16BE(type,0);tail.writeUInt16BE(1,2);
  const packet=Buffer.concat([header,name(host),tail]);
  const response=new Promise((resolve,reject)=>{
    const timer=setTimeout(()=>reject(Error('fixture reply deadline')),1000);
    socket.once('message',data=>{clearTimeout(timer);resolve(data);});
  });
  socket.send(packet,port,'127.0.0.1');
  const reply=await response;
  assert.equal(reply.readUInt16BE(0),123);assert.equal(reply.readUInt16BE(6),1);
  assert.deepEqual(reply.subarray(12,packet.length),packet.subarray(12));
  assert.equal(reply.readUInt16BE(packet.length+2),type);
  const size=reply.readUInt16BE(packet.length+10);
  assert.equal(reply.length,packet.length+12+size);
  return reply.subarray(packet.length+12);
}

test('controlled service DNS advertises the actual nondefault endpoint',async()=>{
  const fixture=new DnsFixture(),socket=createSocket('udp4');
  try {
    const endpoint=await fixture.start('192.0.2.4',39876),port=Number(endpoint.split(':')[1]);
    const naptr=await query(socket,port,'rtc.example.test',35);
    assert.deepEqual(naptr,Buffer.concat([Buffer.alloc(4),Buffer.from([1,83,14]),Buffer.from('RELAY:turn.udp'),Buffer.from([0]),name('_turn._udp.hosted.example.test')]));
    for(const host of ['_turn._udp.hosted.example.test','_stun._udp.rtc.example.test']) {
      const srv=await query(socket,port,host,33);
      assert.equal(srv.readUInt16BE(4),39876);
      assert.deepEqual(srv.subarray(6),name('server.example.test'));
    }
    assert.deepEqual(await query(socket,port,'server.example.test',1),Buffer.from([192,0,2,4]));
    assert.deepEqual(fixture.snapshot(),{queries:4,answers:4,held:0,types:{a:1,srv:2,naptr:1}});
  } finally {socket.close();await fixture.close();}
});
