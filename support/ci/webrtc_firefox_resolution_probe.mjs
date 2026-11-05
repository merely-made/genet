// Post-failure fixture control only. Never qualifies native advertisement.
import {isIPv4} from 'node:net';
export function resolutionAnswer(name,address){
  if(!/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}\.local$/.test(name)||!isIPv4(address))throw Error('resolution probe requires an owned UUID and IPv4 endpoint');
  const header=Buffer.alloc(12);header.writeUInt16BE(0x8400,2);header.writeUInt16BE(1,6);
  const labels=Buffer.concat([...name.split('.').map(s=>Buffer.concat([Buffer.from([s.length]),Buffer.from(s)])),Buffer.from([0])]);
  return Buffer.concat([header,labels,Buffer.from([0,1,0x80,1,0,0,0,120,0,4,...address.split('.').map(Number)])]);
}
export async function probeFirefoxResolution({browser,native,fixture,bind}){
  const result={control:'post-failure fixture answer; not native advertisement',sent:0};
  const candidate=await native.value(`(()=>{const sdp=__interop.pc.localDescription.sdp;return {candidate:sdp.match(/^a=(candidate:.* typ host.*)$/m)[1].trim(),sdpMLineIndex:0,usernameFragment:sdp.match(/a=ice-ufrag:([^\r\n]+)/)[1]};})()`);
  const name=candidate.candidate.split(/\s+/)[4];if(!fixture.nativeNames.has(name))throw Error('resolution replay name is not observed native ownership');
  const packet=resolutionAnswer(name,bind),before=await browser.ipcTrace.snapshot();
  const transportBefore=browser.firefoxTrace.snapshot();
  await browser.evaluate(`__interop.pc.addIceCandidate(${JSON.stringify(candidate)})`,true);
  fixture.socket.setMulticastInterface(bind);fixture.socket.setMulticastLoopback(true);fixture.socket.setMulticastTTL(255);
  for(let i=0;i<20;i++){
    await new Promise((resolve,reject)=>fixture.socket.send(packet,5353,'224.0.0.251',error=>error?reject(error):resolve()));result.sent++;
    await new Promise(resolve=>setTimeout(resolve,150));
  }
  result.before=before;result.after=await browser.ipcTrace.snapshot();
  const after=browser.firefoxTrace.snapshot();result.transportDelta=Object.fromEntries(['transportParseCalls','transportProcessFailures','transportMissingStreams','transportRedundant','transportUnusedComponent'].map(k=>[k,after[k]-transportBefore[k]]));
  await browser.evaluate('__interop.inspect()');await new Promise(resolve=>setTimeout(resolve,100));
  const host=await native.value('__interop.pc.sctp.transport.iceTransport.getLocalCandidates().filter(c=>c.type==="host").map(c=>({address:c.address,port:c.port}))[0]');
  result.remoteHosts=await browser.value(`__interop.hostEvidence(${JSON.stringify(host.address)},${JSON.stringify(bind)},${host.port})`);
  return result;
}
