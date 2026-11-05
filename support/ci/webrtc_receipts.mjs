// Compact, append-only checkpoints for the long headed restart matrix.
import { readFile, appendFile } from 'node:fs/promises';
import { createHash, randomUUID } from 'node:crypto';

const fields=['browser','engine','route','offerer','cleanup'];
const key=c=>fields.map(f=>c[f]).join('/');
const assert=(value,message)=>{if(!value)throw Error(message);};
const digest=value=>createHash('sha256').update(JSON.stringify(value)).digest('hex');
export function assertWritablePacket(text) {
  assert(!/Packet closed after the final (?:qualified run|audit)/.test(text),'qualification packet is closed; select a fresh receipt path');
}
function records(text) {
  const result=[];
  for(const match of text.matchAll(/```json\r?\n([\s\S]*?)\r?\n```/g)) {
    try {const row=JSON.parse(match[1]);if(row&&typeof row==='object'&&!Array.isArray(row))result.push(row);}catch{}
  }
  return result;
}
function compact(c) {
  if(c.status!=='pass') {
    const d=c.restartDiagnostics;
    return {...Object.fromEntries(fields.map(f=>[f,c[f]])),status:'fail',phase:c.phase,error:c.error,initialAdvertisement:c.advertisement,resolution:c.resolution,initialStats:c.initialStatsDiagnostics,
      diagnostics:d?{completedRestarts:d.completedRestarts,step:d.step,method:d.method,initiator:d.initiator,iceBefore:d.iceBefore,iceAfter:d.iceAfter,statsObserved:d.statsObserved,
        peers:Object.fromEntries(Object.entries(d.snapshots||{}).map(([name,s])=>[name,{
          connection:s.connectionState,ice:s.iceState,signaling:s.signaling,gathering:s.gathering,
          receivedMessages:s.received.length,channelStates:s.channelStates,errors:(s.errors||[]).slice(0,4).map(e=>e.slice(0,160)),
          localCredentialsChanged:s.localCredentialsChanged,remoteCredentialsChanged:s.remoteCredentialsChanged}]))}:undefined};
  }
  assert(c.restarts.length===2,'restart result missing a round');
  for(const r of c.restarts) {
    for(const f of ['credentialsChangedOnBothPeers','channelIdentityPreserved','certificateIdentityPreserved','staleCandidateRefused','relayAllocationReused'])assert(r[f]===true,'restart invariant missing: '+f);
    assert(r.liveChannelProbes===4&&r.binaryBytesPerPeer===192000,'restart payload evidence missing');
  }
  assert(c.socketPreserved&&c.socketRebound&&c.cleanupVerified&&c.interruptedRestart,'restart cleanup evidence missing');
  const a=c.native,b=c.browserPeer;
  assert(a.connected&&b.connected&&a.remoteAddressesRedacted&&!a.peerReflexiveInRemoteList,'restart observation evidence missing');
  const counts=values=>Object.fromEntries([...new Set(values)].map(v=>[v,values.filter(x=>x===v).length]));
  const codes={native:a.gatheringErrors.map(e=>e.code),browser:b.gatheringErrors.map(e=>e.code)};
  return {...Object.fromEntries(fields.map(f=>[f,c[f]])),status:'pass',seconds:c.seconds,
    pairs:{native:[a.pair.local,a.pair.remote],browser:[b.pair.local,b.pair.remote]},
    nominated:[a.pair.nominated,b.pair.nominated],certificatePairSHA256:digest([a.localFingerprint,b.localFingerprint]),
    refusals:a.refused.length||b.refused.length?{native:counts(a.refused),browser:counts(b.refused)}:undefined,
    gatheringErrorCodes:codes.native.length||codes.browser.length?codes:undefined,
    restarts:c.restarts.map(r=>({method:r.method,initiator:r.initiator,mdnsResolved:r.mdnsResolved,nativeNegotiationEvents:r.nativeNegotiationEvents||undefined,statsSettling:r.statsSettling,advertisement:r.advertisement})),
    dns:c.dns,mdns:c.mdns,advertisement:c.advertisement,initialStatsSettling:c.initialStatsSettling,commonAssertionsVerified:true};
}

export class RestartJournal {
  static async open(file,identity,source,resume) {
    assert(!['2026-11-01','2026-11-03'].some(date=>file.endsWith(date+'_webrtc_conformance_plan_receipts.md')),'baseline receipt is frozen');
    const gate=identity.gate||'E2s';
    const old=await readFile(file,'utf8'),rows=records(old),signature=digest(identity);
    assertWritablePacket(old);
    const start=resume?rows.findLast(r=>r.gate===gate+'-run'&&(resume==='latest'||r.run===resume)):null;
    if(resume)assert(start&&start.signature===signature,'restart resume requires identical source, binary, lock and fixture profile');
    const journal=new RestartJournal();
    Object.assign(journal,{file,identity,gate,signature,run:start?.run||randomUUID(),passed:new Map(),versions:new Map()});
    if(start) {
      for(const r of rows.filter(r=>r.run===start.run)) {
        if(r.gate===gate+'-case'&&r.status==='pass') {
          journal.validateCase(r);assert(!journal.passed.has(key(r)),'duplicate durable case');journal.passed.set(key(r),r);
        }
        if(r.gate===gate+'-browser')journal.versions.set(r.browser,r);
      }
      await journal.append('resume',{source,completed:journal.passed.size});
    } else await journal.append('run',{signature,source,...identity,
      commonAssertions:{channelModes:['ordered reliable','unordered maxRetransmits=0'],
        initialMessagesPerPeer:3,initialBinaryBytesPerPeer:48000,restartsPerCase:2,liveProbesPerRestart:4,
        binaryBytesPerPeerPerRestart:192000,unicodeAndEveryBinaryByteChecked:true,
        changedLocalAndRemoteCredentials:true,sameChannelsAndCertificates:true,staleNativeCandidateRefused:true,
        nativeStatsReferencesAndPrivacyChecked:true,nativeRelayAllocationReused:true,
        certificateEvidence:'SHA256 of JSON [native SHA256 fingerprint,browser SHA256 fingerprint]',
        omittedGatheringErrorCodesMeansNone:true,
        omittedRefusalsAndNegotiationCountsMeanZero:true,
        caseVerifiedMeans:'all common assertions passed and source/native binary unchanged at checkpoint',
        pendingRestartInterruptedBeforeCleanup:true,observableNativeRtcAndDnsEndpointsReleasedAndRebound:true,
        sharedMdnsEvidence:identity.mdns?'Windows owner table may omit shared 5353 binds; pending tickets cancel in headed checks, actual multicast socket/drop behavior has provider tests.':undefined},
      scope:identity.localMdns?'Same-machine selected LAN-interface IPv4 native advertisement, untouched native names, browser named-host selection, privacy and lifecycle. Cross-machine, IPv6 and browser-native responder acceptance remain separate.':identity.mdns?`Windows native remote UUID mDNS resolution on a fixed IPv4 interface; ${identity.mdnsAliasBrowser ? identity.mdnsAliasBrowser+' host names replaced by controlled selected-IPv4 fixture aliases; its own advertisements remain unqualified. Other browser names stay untouched.' : 'controlled numeric-host aliases and untouched browser names.'} UDP STUN/TURN regressions, relay suppression and pending-query cancellation. Local name advertisement, IPv6 multicast, TURN TCP/TLS, public NAT, media and carriers remain open.`:identity.dnsMode==='service'?'Windows native data channels, portless STUN/UDP-TURN through controlled SRV/S-NAPTR at nondefault ports; browser peers use equivalent numeric endpoints. Fixed interface; mDNS, TURN TCP/TLS, public routing, media and carriers remain open.':identity.dns?'Windows native data channels, explicit-port hostname STUN/UDP-TURN through controlled A lookup; browser peers use the equivalent numeric endpoint. Fixed interface; SRV/NAPTR, mDNS, TURN TCP/TLS, media and carriers remain open.':'Windows native data channels with fixed numeric UDP servers'});
    return journal;
  }
  validateCase(c) {
    assert(fields.every(f=>this.identity.choices[f].includes(c[f])),'case outside declared matrix');
    if(this.identity.localMdns&&c.status==='pass'){
      assert(c.advertisement?.private===true,'native advertisement privacy evidence missing');
      assert(c.route==='relay'?c.advertisement.hostCount===0:c.advertisement.hostCount===1,'unexpected advertisement count');
      if(c.route==='direct')assert(c.advertisement.browserNamedHostSelected===true,'named native host was not selected by browser');
      assert(c.restarts.length===2&&c.restarts.every(r=>r.advertisement?.private&&(c.route==='relay'||r.advertisement.nameStable)),'restart advertisement evidence missing');
    }
    if(this.identity.dns&&c.status==='pass'){
      const pending=Number(c.route==='stun')+Number(this.identity.mdns&&c.route!=='relay');
      const shared=c.dns?.sharedMdnsEndpointsObserved||0;
      assert(shared>=0&&shared<=Number(this.identity.mdns&&c.route!=='relay'),'unexpected shared mDNS endpoint count');
      assert(c.dns&&c.dns.pendingAtCleanup===pending&&c.dns.socketsReleased===1+pending+shared,'DNS ownership evidence missing');
      assert(c.route==='direct'?c.dns.queries===0:c.dns.answers>=1&&c.dns.originalUrlAndPrivacyChecked,'DNS exchange/metadata evidence missing');
      if(c.route==='stun')assert(c.dns.held>=1,'DNS cancellation case lacked a pending request');
      if(this.identity.dnsMode==='service'&&c.route!=='direct'){
        assert(c.dns.mode==='service'&&c.dns.types.srv>=1&&c.dns.types.a>=1,'service lookup evidence missing');
        assert(c.route==='relay'?c.dns.types.naptr===1:c.dns.types.naptr===0,'unexpected S-NAPTR ownership');
      }
    }
    if(this.identity.mdns&&c.status==='pass'){
      assert(c.mdns&&c.mdns.namesAndStatsPrivate,'mDNS privacy evidence missing');
      if(c.route!=='relay'&&c.browser===this.identity.mdnsAliasBrowser)assert(c.mdns.aliasedBrowserNames>=1&&c.mdns.answers>=1,'controlled browser-name profile evidence missing');
      if(c.route==='relay')assert(c.mdns.started===0&&c.mdns.resolved===0&&c.mdns.pendingAtCleanup===0,'relay policy issued mDNS');
      else {assert(c.mdns.resolved>=1&&c.mdns.pendingAtCleanup===1&&c.mdns.held>=1,'mDNS resolution/cancellation evidence missing');assert(c.restarts.length===2&&c.restarts.every(r=>r.mdnsResolved>=1),'fresh mDNS restart evidence missing');}
    }
    if(c.gate===this.gate+'-case'&&c.status==='pass')assert(c.verified===true||(c.commonAssertionsVerified&&c.sourceStable),'case lacks checkpoint verification');
  }
  has(c){return this.passed.has(key(c));}
  cases(){return [...this.passed.values()];}
  async browser(version) {
    const prior=this.versions.get(version.browser);
    assert(!prior||prior.version===version.version,'browser build changed during resumed qualification');
    await this.append('browser',version);this.versions.set(version.browser,version);
  }
  async case(c,sourceStable) {
    assert(sourceStable,'source or native binary changed before checkpoint');this.validateCase(c);
    if(c.status==='pass')assert(!this.has(c),'duplicate durable case');
    const row=compact(c);
    if(c.status==='pass') {delete row.commonAssertionsVerified;row.verified=true;}
    await this.append('case',{...row,sourceStable:c.status==='pass'?undefined:sourceStable});
    if(c.status==='pass')this.passed.set(key(c),row);
  }
  async finish(failure,sourceStable) {
    const complete=!failure&&sourceStable&&this.passed.size===this.identity.expectedCases;
    await this.append('run-end',{failure,sourceStable,completedCases:this.passed.size,
      completeMatrix:complete&&this.identity.expectedCases===48,completeSelectedMatrix:complete,
      completedRestarts:this.passed.size*2,interruptedRestarts:this.passed.size,
      versions:[...this.versions.values()].map(v=>({browser:v.browser,version:v.version,headed:v.headed})),
      profileCleanup:'earlier recursive profile cleanup was rejected as blocked by policy; marker-owned caches retained; reviewer not established'});
  }
  async append(kind,data) {
    const old=await readFile(this.file,'utf8');
    assertWritablePacket(old);
    // A killed writer may leave one incomplete block. Preserve it and begin a
    // fresh block; only complete, parsed records can count as passed cases.
    const lastFence=[...old.matchAll(/(?:^|\n)```[^\n]*(?:\n|$)/g)].at(-1)?.[0].trim();
    const repair=lastFence&&lastFence!=='```'?'\n```\n\nInterrupted checkpoint retained; incomplete JSON is not counted.\n':'';
    const row={...data,schema:2,gate:this.gate+'-'+kind,run:this.run,utc:new Date().toISOString()};
    const entry=repair+'\n\n### '+this.gate+' headed '+kind+'\n\n```json\n'+JSON.stringify(row)+'\n```\n';
    assert(Buffer.byteLength(old+entry)<=(this.gate==='E2ah'?340000:this.gate==='E2m'?244000:220000),this.gate+' receipt byte cap exceeded');
    await appendFile(this.file,entry);
  }
}
