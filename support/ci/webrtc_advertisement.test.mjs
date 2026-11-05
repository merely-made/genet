import { test } from 'node:test';
import assert from 'node:assert/strict';
import { validateAdvertisement } from './webrtc_advertisement.mjs';
const address='53caef04-4218-4fb0-815d-f91a511bb8d2.local';
const foundation=address.slice(0,36).replaceAll('-','');
const view=()=>({hosts:[{address,foundation}],statsHosts:[{address}],descriptions:[`c=IN IP4 0.0.0.0\r\na=candidate:${foundation} 1 udp 100 ${address} 1234 typ host\r\n`]});
test('actual UUID host selection and stable restart name are required',()=>{
  const first=validateAdvertisement(view(),{pair:{remote:'host'}},'direct','192.0.2.1');
  assert.equal(validateAdvertisement(view(),{pair:{remote:'host'}},'direct','192.0.2.1',first).nameStable,true);
  assert.throws(()=>validateAdvertisement(view(),{pair:{remote:'prflx'}},'direct','192.0.2.1'),/fallback/);
  assert.throws(()=>validateAdvertisement(view(),{pair:{remote:'host'}},'direct','192.0.2.1',{nameSHA256:'wrong'}),/changed/);
});
test('numeric host, foundation, SDP and stats leaks are refused',()=>{
  for(const mutate of [v=>v.hosts[0].address='192.0.2.1',v=>v.hosts[0].foundation='123',v=>v.statsHosts[0].address='192.0.2.1',v=>v.descriptions=[],v=>v.descriptions=['c=IN IP4 192.0.2.1'],v=>v.descriptions=[`a=candidate:123 1 udp 100 ${address} 1234 typ host`]]){
    const v=view();mutate(v);assert.throws(()=>validateAdvertisement(v,{pair:{remote:'host'}},'direct','192.0.2.1'));
  }
});
test('relay-only requires no host advertisement',()=>{
  assert.equal(validateAdvertisement({hosts:[]},{},'relay','192.0.2.1').relaySuppressed,true);
  assert.throws(()=>validateAdvertisement(view(),{},'relay','192.0.2.1'));
});
