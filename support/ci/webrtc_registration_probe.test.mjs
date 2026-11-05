import test from 'node:test';
import assert from 'node:assert/strict';
import {validRegistrationReply} from './webrtc_registration_probe.mjs';
test('registration probes require the expected transaction, source and positive owned answer',()=>{
  const name='53caef04-4218-4fb0-815d-f91a511bb8d2.local',labels=Buffer.concat([...name.split('.').map(s=>Buffer.concat([Buffer.from([s.length]),Buffer.from(s)])),Buffer.from([0])]);
  const h=Buffer.alloc(12);h.writeUInt16BE(42,0);h.writeUInt16BE(0x8400,2);h.writeUInt16BE(1,6);
  const packet=Buffer.concat([h,labels,Buffer.from([0,1,0x80,1,0,0,0,120,0,4,192,0,2,1])]);
  const from={address:'192.0.2.1',port:5353};
  assert(validRegistrationReply(packet,from,name,'192.0.2.1',42));
  assert(!validRegistrationReply(packet,from,name,'192.0.2.1',43));
  assert(!validRegistrationReply(packet,{...from,port:40000},name,'192.0.2.1',42));
  assert(!validRegistrationReply(packet,from,'foreign.local','192.0.2.1',42));
  packet.writeUInt32BE(0,12+labels.length+4);
  assert(!validRegistrationReply(packet,from,name,'192.0.2.1',42));
});
