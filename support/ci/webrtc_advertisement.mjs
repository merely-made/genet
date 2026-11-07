// Evidence for native advertisement; never rewrites a signaled candidate.
import { createHash } from 'node:crypto';
const assert=(value,message)=>{if(!value)throw Error(message);};
export function validateAdvertisement(view,browserStats,route,bind,before) {
  if(route==='relay') {
    assert(view.hosts.length===0,'relay-only exposed a host registration');
    return {hostCount:0,private:true,relaySuppressed:true};
  }
  assert(view.hosts.length===1,'native registration must expose one host name');
  const names=view.hosts.map(c=>c.address);
  for(const c of view.hosts) {
    assert(/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}\.local$/.test(c.address),'native host is not a private UUID');
    assert(c.foundation===c.address.slice(0,36).replaceAll('-',''),'address-derived foundation exposed');
  }
  assert(view.statsHosts.length>0&&view.statsHosts.every(c=>names.includes(c.address)),'local host stats exposed an address');
  assert(view.descriptions.some(Boolean),'local SDP observation missing');
  for(const sdp of view.descriptions.filter(Boolean)) {
    for(const line of sdp.split(/\r?\n/)) {
      if(line.startsWith('a=candidate:')) {
        const f=line.slice(2).split(/\s+/);
        if(f[7]==='host')assert(names.includes(f[4])&&f[0]==='candidate:'+f[4].slice(0,36).replaceAll('-',''),'SDP host privacy failed');
        const at=f.indexOf('raddr');if(at>=0)assert(f[at+1]!==bind,'SDP related address exposed');
      }
      assert(line!==`c=IN IP4 ${bind}`,'SDP connection address exposed');
    }
  }
  const nameSHA256=createHash('sha256').update(JSON.stringify(names)).digest('hex');
  if(before)assert(before.nameSHA256===nameSHA256,'restart changed the registered name');
  // With only native UUID hosts signaled, a selected browser host candidate
  // demonstrates use of the named candidate instead of a peer-reflexive route.
  if(route==='direct')assert(browserStats.pair?.remote==='host','browser selected a fallback instead of the named native host');
  return {hostCount:1,private:true,nameSHA256,nameStable:!!before,browserNamedHostSelected:route==='direct'};
}
export async function inspectAdvertisement(native,browserStats,route,bind,before) {
  const view=await native.value(`({hosts:__interop.pc.sctp.transport.iceTransport.getLocalCandidates().filter(c=>c.type==='host').map(c=>({address:c.address,foundation:c.foundation})),statsHosts:__interop.stats.localHostViews,descriptions:[__interop.pc.localDescription?.sdp,__interop.pc.currentLocalDescription?.sdp,__interop.pc.pendingLocalDescription?.sdp]})`);
  return validateAdvertisement(view,browserStats,route,bind,before);
}
