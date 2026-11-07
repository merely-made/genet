// Evaluated in the owned Firefox fixture. Keep addresses and credentials local.
export function watchFirefoxClose() {
  const peer = globalThis.__interop;
  const pc = peer.pc;
  const start = performance.now();
  const rows = [];
  const attached = new WeakSet();
  const ufrag = description => description?.sdp.match(/^a=ice-ufrag:([^\r\n]+)/m)?.[1];
  const previousLocal = ufrag(pc.localDescription);
  const previousRemote = ufrag(pc.remoteDescription);
  const initialDtls = pc.sctp?.transport;
  const initialIce = initialDtls?.iceTransport;
  let omitted = 0;
  const generation = (value, old, current) => !value ? undefined :
    value === old ? 'previous' : value === current ? 'fresh' : 'other';

  function record(event, channel) {
    if (rows.length >= 64) {
      omitted++;
      return;
    }
    const sctp = pc.sctp;
    const dtls = sctp?.transport;
    const ice = dtls?.iceTransport;
    const pair = ice?.getSelectedCandidatePair?.();
    const local = ufrag(pc.localDescription);
    const remote = ufrag(pc.remoteDescription);
    rows.push({
      event,
      ms: Math.round(performance.now() - start),
      signaling: pc.signalingState,
      pc: pc.connectionState,
      ice: pc.iceConnectionState,
      transportIce: ice?.state,
      dtls: dtls?.state,
      sctp: sctp?.state,
      sameIce: ice === initialIce,
      sameDtls: dtls === initialDtls,
      local: generation(local, previousLocal, local),
      remote: generation(remote, previousRemote, remote),
      selected: pair ? {
        remoteType: pair.remote?.type,
        local: generation(pair.local?.usernameFragment, previousLocal, local),
        remote: generation(pair.remote?.usernameFragment, previousRemote, remote)
      } : undefined,
      channels: peer.channels.map(ch => ch.readyState),
      channel: channel ? peer.channels.indexOf(channel) : undefined
    });
  }
  function attach(target, events) {
    if (!target || attached.has(target)) return;
    attached.add(target);
    for (const [event, label] of events) {
      target.addEventListener(event, () => record(label));
    }
  }
  function transports() {
    attach(pc.sctp, [['statechange','sctp-state']]);
    attach(pc.sctp?.transport, [['statechange','dtls-state']]);
    attach(pc.sctp?.transport?.iceTransport, [
      ['statechange','ice-transport-state'], ['selectedcandidatepairchange','selected-pair']
    ]);
  }
  pc.addEventListener('signalingstatechange', () => {
    transports();
    record('signaling-state');
  });
  attach(pc, [['iceconnectionstatechange','ice-state'],['connectionstatechange','connection-state']]);
  transports();
  for (const channel of peer.channels) {
    for (const event of ['closing','close','error']) {
      channel.addEventListener(event, () => record('channel-' + event, channel));
    }
  }
  peer.closeTraceMark = event => record(event);
  peer.closeTrace = () => ({rows, omitted});
  record('start');
}
