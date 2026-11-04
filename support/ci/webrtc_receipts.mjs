// Compact, append-only checkpoints for the long headed restart matrix.
import { readFile, appendFile } from 'node:fs/promises';
import { createHash, randomUUID } from 'node:crypto';

const fields=['browser','engine','route','offerer','cleanup'];
const key=c=>fields.map(f=>c[f]).join('/');
const assert=(value,message)=>{if(!value)throw Error(message);};
const digest=value=>createHash('sha256').update(JSON.stringify(value)).digest('hex');
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
    return {...Object.fromEntries(fields.map(f=>[f,c[f]])),status:'fail',phase:c.phase,error:c.error,
      diagnostics:d?{step:d.step,method:d.method,initiator:d.initiator,iceBefore:d.iceBefore,iceAfter:d.iceAfter,statsObserved:d.statsObserved,
        peers:Object.fromEntries(Object.entries(d.snapshots||{}).map(([name,s])=>[name,{
          connection:s.connectionState,ice:s.iceState,signaling:s.signaling,gathering:s.gathering,
          receivedMessages:s.received.length,channelStates:s.channelStates,
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
    restarts:c.restarts.map(r=>({method:r.method,initiator:r.initiator,nativeNegotiationEvents:r.nativeNegotiationEvents||undefined,statsSettling:r.statsSettling})),
    dns:c.dns,commonAssertionsVerified:true};
}

export class RestartJournal {
  static async open(file,identity,source,resume) {
    assert(!file.endsWith('2026-11-01_webrtc_conformance_plan_receipts.md'),'baseline receipt is frozen');
    const gate=identity.gate||'E2d';
    const old=await readFile(file,'utf8'),rows=records(old),signature=digest(identity);
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
        pendingRestartInterruptedBeforeCleanup:true,sameNativeRtcSocketThenAllRtcAndDnsSocketsReleasedAndRebound:true},
      scope:identity.dns?'Windows native data channels, explicit-port hostname STUN/UDP-TURN through controlled A lookup; browser peers use the equivalent numeric endpoint. Fixed interface; SRV/NAPTR, mDNS, TURN TCP/TLS, media and carriers remain open.':'Windows native data channels with fixed numeric UDP servers'});
    return journal;
  }
  validateCase(c) {
    assert(fields.every(f=>this.identity.choices[f].includes(c[f])),'case outside declared matrix');
    if(this.identity.dns&&c.status==='pass'){
      const pending=c.route==='stun'?1:0;
      assert(c.dns&&c.dns.pendingAtCleanup===pending&&c.dns.socketsReleased===1+pending,'DNS ownership evidence missing');
      assert(c.route==='direct'?c.dns.queries===0:c.dns.answers>=1&&c.dns.originalUrlAndPrivacyChecked,'DNS exchange/metadata evidence missing');
      if(pending)assert(c.dns.held>=1,'DNS cancellation case lacked a pending request');
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
      profileCleanup:'automatic approval review rejected recursive profile deletion as blocked by policy; marker-owned profile caches retained'});
  }
  async append(kind,data) {
    const old=await readFile(this.file,'utf8');
    // A killed writer may leave one incomplete block. Preserve it and begin a
    // fresh block; only complete, parsed records can count as passed cases.
    const lastFence=[...old.matchAll(/(?:^|\n)```[^\n]*(?:\n|$)/g)].at(-1)?.[0].trim();
    const repair=lastFence&&lastFence!=='```'?'\n```\n\nInterrupted checkpoint retained; incomplete JSON is not counted.\n':'';
    const row={...data,schema:2,gate:this.gate+'-'+kind,run:this.run,utc:new Date().toISOString()};
    const entry=repair+'\n\n### '+this.gate+' headed '+kind+'\n\n```json\n'+JSON.stringify(row)+'\n```\n';
    assert(Buffer.byteLength(old+entry)<=220000,'E2d receipt byte cap exceeded');
    await appendFile(this.file,entry);
  }
}
