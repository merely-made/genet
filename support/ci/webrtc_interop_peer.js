// Shared application fixture, executed by real Genet and browser WebRTC APIs.
// Test state is deliberately bounded and never reports SDP/ICE credentials.
(function () {
  globalThis.makeInteropPeer = function (config, name, offerer) {
    const pc = new RTCPeerConnection(config), channels = [], signals = [], waiting = [];
    const received = [], sent = [], modes = [], errors = [], refused = [], remoteProtocols=[], gatheringErrors=[];
    const state = { pc, channels, connected: false, remoteSet: false, stats: null, round: 0, pauseAnswer: false, waitingToAnswer: false, negotiations: 0 };
    const knownRemoteGenerations = new Set();
    const credentials = d => [d?.sdp.match(/a=ice-ufrag:([^\r\n]+)/)?.[1],d?.sdp.match(/a=ice-pwd:([^\r\n]+)/)?.[1]];
    const changed = (d,before) => before!==null&&credentials(d).every((value,i)=>value!==before[i]);
    const generation = d => d?.sdp.match(/a=ice-ufrag:([^\r\n]+)/)?.[1];
    const fail = (operation, e) => {
      // Whitelisted diagnostic words identify protocol failures without copying
      // browser SDP diagnostics or generated ICE credentials into receipts.
      const words=String(e?.message||'').match(/\b(?:DTLS|SSL|role|setup|fingerprint|m-line|mid|media|ICE|ufrag|password|application|active|passive)\b/ig)||[];
      errors.push(operation + ':' + (e?.name || 'Error') + ':' + [...new Set(words)].join(','));
    };
    function candidate(c) {
      pc.addIceCandidate(c).catch(e => {
        // mDNS resolution is a named open gate. Keep this refusal visible;
        // connectivity must independently succeed using an observed IP pair.
        if (c && /\s[^ ]+\.local\s/.test(c.candidate)) refused.push('mdns:' + e.name);
        else fail('candidate', e);
      });
    }
    pc.onicecandidate = e => signals.push({candidate:e.candidate?.toJSON() || null,generation:generation(pc.localDescription)});
    pc.onicecandidateerror = e => gatheringErrors.push({code:e.errorCode,text:e.errorText.slice(0,120)});
    pc.onconnectionstatechange = () => { state.connected = pc.connectionState === 'connected'; };
    let restartOnNegotiation = false, beforeLocal = null, beforeRemote = null, staleCandidate = null, beforeChannels = null, beforeNegotiations = 0;
    pc.onnegotiationneeded = () => {
      state.negotiations++;
      if(restartOnNegotiation){restartOnNegotiation=false;local().catch(e=>fail('restart offer',e));}
    };
    function send(channel) {
      channel.send(name + ':text:λ🙂:' + channel.label + ':' + state.round); sent.push(channel.label + ':text');
      if (channel.label === 'reliable') {
        const bytes = new Uint8Array(48000); bytes.fill(90); bytes[0] = 90 + state.round; channel.send(bytes);
        sent.push('reliable:binary');
      }
    }
    function attach(channel) {
      channels.push(channel); channel.binaryType = 'arraybuffer';
      channel.onopen = () => {
        modes.push({label:channel.label,ordered:channel.ordered,maxRetransmits:channel.maxRetransmits});
        send(channel);
      };
      channel.onmessage = e => {
        if (typeof e.data === 'string') received.push(e.data);
        else {
          const bytes = new Uint8Array(e.data);
          if (bytes.length !== 48000 || bytes[0] !== 90 + state.round || !bytes.subarray(1).every(b => b === 90)) errors.push('binary:corrupt');
          received.push('binary:' + bytes.length + ':' + state.round);
        }
      };
      channel.onerror = () => errors.push('channel:error');
    }
    pc.ondatachannel = e => attach(e.channel);
    async function local(options) {
      if(options)await pc.setLocalDescription(await pc.createOffer(options));
      else await pc.setLocalDescription();
      signals.push({description:pc.localDescription.toJSON()});
    }
    state.start = function () {
      if (!offerer) return;
      attach(pc.createDataChannel('reliable'));
      attach(pc.createDataChannel('unreliable', {ordered:false,maxRetransmits:0}));
      local().catch(e => fail('offer', e));
    };
    state.receive = function (signal) {
      if ('description' in signal) {
        pc.setRemoteDescription(signal.description).then(async () => {
          state.remoteSet = true;
          knownRemoteGenerations.add(generation(pc.remoteDescription));
          for (const c of waiting.splice(0)) state.receive(c);
          if (signal.description.type === 'offer') {
            if(state.pauseAnswer)state.waitingToAnswer=true;
            else await local();
          }
        }).catch(e => fail('description', e));
      } else {
        if(signal.candidate)remoteProtocols.push(signal.candidate.candidate.split(/\s+/)[2]);
        if (state.remoteSet && (!signal.generation || signal.generation===generation(pc.remoteDescription))) candidate(signal.candidate);
        else if(!knownRemoteGenerations.has(signal.generation)) {
          if(waiting.length>=64)errors.push('candidate:queue limit');else waiting.push(signal);
        } else refused.push('stale signaling generation');
      }
    };
    state.takeSignals = () => signals.splice(0);
    state.captureRestart = function () {
      beforeLocal=credentials(pc.localDescription);beforeRemote=credentials(pc.remoteDescription);
      beforeChannels=channels.map(channel=>({channel,id:channel.id}));beforeNegotiations=state.negotiations;
      staleCandidate={candidate:'candidate:stale 1 udp 1 192.0.2.1 9 typ host',sdpMLineIndex:0,usernameFragment:generation(pc.remoteDescription)};
      state.pauseAnswer=true;
      state.staleRefused=false;
    };
    state.restart = function (method) {
      if(method==='method'){restartOnNegotiation=true;pc.restartIce();}
      else local({iceRestart:true}).catch(e=>fail('restart option',e));
    };
    state.answer = function () {state.waitingToAnswer=false;local().catch(e=>fail('restart answer',e));};
    state.beginRound = function (round) {state.round=round;received.length=0;sent.length=0;};
    state.sendRound = function () {for(const channel of channels)send(channel);};
    state.rejectStale = function () {
      pc.addIceCandidate(staleCandidate).then(()=>errors.push('stale candidate accepted'),e=>{
        if(e.name==='OperationError')state.staleRefused=true;else fail('stale candidate exception',e);
      });
    };
    state.snapshot = function () {
      return {connected:state.connected,connectionState:pc.connectionState,iceState:pc.iceConnectionState,signaling:pc.signalingState,remoteSet:state.remoteSet,received,sent,modes,errors,refused,remoteProtocols,gatheringErrors,
        channelStates:channels.map(c=>c.readyState),channelBufferedBytes:channels.map(c=>c.bufferedAmount),gathering:pc.iceGatheringState,round:state.round,waitingToAnswer:state.waitingToAnswer,
        localCredentialsChanged:changed(pc.localDescription,beforeLocal),remoteCredentialsChanged:changed(pc.remoteDescription,beforeRemote),
        negotiations:state.negotiations,restartNegotiations:state.negotiations-beforeNegotiations,channelCount:channels.length,staleRefused:!!state.staleRefused,
        channelsPreserved:!!beforeChannels&&channels.length===beforeChannels.length&&beforeChannels.every((old,i)=>old.channel===channels[i]&&old.id===channels[i].id)};
    };
    state.inspect = function () {
      state.stats=null;
      pc.getStats().then(report => {
        const values = Array.from(report.values());
        const transport = values.find(s=>s.type==='transport' && s.selectedCandidatePairId);
        const pair = transport ? report.get(transport.selectedCandidatePairId) :
          values.find(s=>s.type==='candidate-pair' && s.state==='succeeded' && (s.selected || s.nominated));
        const localCandidate = pair && report.get(pair.localCandidateId);
        const remoteCandidate = pair && report.get(pair.remoteCandidateId);
        const fingerprint = description => description?.sdp.match(/a=fingerprint:sha-256 ([^\r\n]+)/i)?.[1].replace(/:/g,'').toLowerCase();
        const certificate = values.find(s=>s.type==='certificate' && s.id===transport?.remoteCertificateId);
        state.stats = {localHostViews:values.filter(s=>s.type==='local-candidate'&&s.candidateType==='host').map(s=>({address:s.address})),localEndpoint:localCandidate ? localCandidate.address+':'+localCandidate.port : null,pair:pair ? {state:pair.state,nominated:pair.nominated,
          local:localCandidate?.candidateType,remote:remoteCandidate?.candidateType,
          localProtocol:localCandidate?.protocol,remoteProtocol:remoteCandidate?.protocol}:null,
          localFingerprint:fingerprint(pc.localDescription),remoteFingerprint:fingerprint(pc.remoteDescription),
          remoteCertificates:typeof pc.sctp.transport.getRemoteCertificates==='function' ?
            pc.sctp.transport.getRemoteCertificates().map(c=>Array.from(new Uint8Array(c))) : [],
          statsRemoteFingerprint:certificate?.fingerprint?.replace(/:/g,'').toLowerCase() || null,
          candidateReferencesValid:values.filter(s=>s.type==='candidate-pair').every(s=>report.has(s.localCandidateId)&&report.has(s.remoteCandidateId))&&values.filter(s=>s.type==='transport'&&s.selectedCandidatePairId).every(s=>report.has(s.selectedCandidatePairId)),
          remoteAddressesRedacted:values.filter(s=>s.type==='remote-candidate').every(s=>s.address===null&&!('url' in s)&&!('relayProtocol' in s)),
          peerReflexiveInRemoteList:typeof pc.sctp.transport.iceTransport.getRemoteCandidates==='function' ? pc.sctp.transport.iceTransport.getRemoteCandidates().some(c=>c.type==='prflx') : null,
          pairKeys:pair?{local:pair.localCandidateId,remote:pair.remoteCandidateId}:null,
          candidateKeys:values.filter(s=>s.type==='local-candidate'||s.type==='remote-candidate').map(s=>({id:s.id,type:s.type,candidateType:s.candidateType,protocol:s.protocol}))};
      }).catch(e => fail('stats', e));
    };
    state.close = function () {
      pc.close(); return {state:pc.signalingState,channels:channels.map(c=>c.readyState)};
    };
    return state;
  };
})();
