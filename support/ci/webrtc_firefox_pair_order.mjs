// Bounded nICEr ordering evidence. Stream labels and pair codewords stay in memory.
import {currentNativeCandidate, resolutionAnswer} from './webrtc_firefox_resolution_probe.mjs';

const iceStates = new Map([[1,'unpaired'],[2,'frozen'],[3,'active'],[4,'connected'],[5,'failed']]);
const pairKind = line => {
  const match = line.match(/\|([^|()]*)\)\.?\s*$/);
  const kind = match?.[1].match(/\b(host|prflx|srflx|relay)\b/)?.[1];
  return kind || 'unknown';
};

export class FirefoxPairOrder {
  constructor(endpoint, previousUfrag = null) {
    this.previousUfrag = previousUfrag;
    this.endpoint = endpoint;
    this.events = [];
    this.omitted = 0;
    this.streams = new Map();
    this.pairs = new Map();
    this.start = performance.now();
  }

  record(event, fields = {}) {
    if (this.events.length < 96) {
      this.events.push({event, ms: Math.round(performance.now() - this.start), ...fields});
    } else {
      this.omitted++;
    }
  }

  number(map, value) {
    if (!value) return undefined;
    if (!map.has(value)) {
      if (map.size >= 96) return undefined;
      map.set(value, map.size + 1);
    }
    return map.get(value);
  }

  push(line, names) {
    const state = line.match(/tried to trickle ICE in inappropriate state (\d+)/);
    const named = [...names].some(name => line.includes(name));
    let event;
    if (/Enqueuing STUN request pre-answer/.test(line)) event = 'queued-check';
    else if (/found pre-answer request/.test(line)) event = 'processed-queued-check';
    else if (/no matching pair\s*$/.test(line)) event = 'unmatched-check';
    else if (/nominated pair is /.test(line)) event = /\b[A-Z]\/mtransport\b/.test(line) ? 'replayed-nomination' : 'nominated';
    else if (/Ensure that check timer is running for new pair/.test(line)) event = 'pair-inserted';
    else if (/cancelling FROZEN\/WAITING pair /.test(line)) event = 'cancelled-waiting-pair';
    else if (/Found a matching pair for received check/.test(line)) event = 'matched-check';
    else if (/state=SUCCEEDED/.test(line)) event = 'succeeded-pair';
    else if (this.previousUfrag && /ICE restart committed, marking old stream as obsolete/.test(line)) event = 'old-stream-obsolete';
    else if (this.previousUfrag && /Setting ICE credentials for/.test(line)) event = 'local-stream-created';
    else if (this.previousUfrag && /dumping r_log ringbuffer/.test(line)) event = 'ice-log-replay';
    else if (this.previousUfrag && /Lower layer experienced an error/.test(line)) event = 'dtls-lower-layer-error';
    else if (this.previousUfrag && /ICE Failed\(/.test(line)) event = 'ice-layer-failed';
    else if (this.previousUfrag && /Marking stream ready/.test(line)) event = 'stream-ready';
    else if (this.previousUfrag && /Marking stream failed/.test(line)) event = 'stream-failed';
    else if (this.previousUfrag && /All pairs are failed, and grace period has elapsed/.test(line)) event = 'checklist-exhausted';
    else if (this.previousUfrag && /starting grace period timer/.test(line)) event = 'grace-start';
    else if (this.previousUfrag && /Trickle grace period is over/.test(line)) event = 'grace-expired';
    else if (this.previousUfrag && /all checks completed success=/.test(line)) event = 'checks-completed';
    else if (state) event = 'trickle-state-refusal';
    else if (named && /parsing trickle ICE candidate /.test(line)) event = 'named-ice-entry';
    else if (named && /parsing trickle candidate /.test(line)) event = 'named-parser-entry';
    else if (named && /Trickle candidate is redundant/.test(line)) event = 'named-redundant';
    else if (named && /Trickle candidate is ignored/.test(line)) event = 'named-unused-component';
    if (!event) return;
    const stream = this.number(this.streams, line.match(/(?:STREAM|stream)\((.*?)\)(?=\/(?:COMP|CAND)|\s*:| tried |$)/)?.[1]);
    const pair = this.number(this.pairs, line.match(/CAND[-_]PAIR\(([^)]+)\)/)?.[1]);
    if (event === 'matched-check' && this.events.some(row => row.event === event && row.pair === pair)) return;
    const replay = /\b[A-Z]\/mtransport\b/.test(line) && /(?:ICE-PEER\(|ICE\()/.test(line);
    if (replay && event !== 'replayed-nomination') return;
    const describesPair = ['nominated','replayed-nomination','pair-inserted'].includes(event);
    const endpoint = describesPair ? line.match(/\|IP4:([\d.]+):(\d+)\/UDP\(/i) : null;
    this.record(event, {
      stream, pair,
      remote: describesPair ? pairKind(line) : undefined,
      component: named ? Number(line.match(/candidate:\S+\s+([12])\s+UDP/i)?.[1]) || undefined : undefined,
      sameNativeEndpoint: endpoint && this.endpoint ?
        endpoint[1] === this.endpoint.address && Number(endpoint[2]) === this.endpoint.port : undefined,
      state: state ? iceStates.get(Number(state[1])) || 'unknown' : undefined,
      successes: event === 'checks-completed' ? Number(line.match(/success=(\d+)/)?.[1]) : undefined,
      failures: event === 'checks-completed' ? Number(line.match(/fail=(\d+)/)?.[1]) : undefined
    });
  }

  get nominated() {
    const before = this.events.findIndex(event => event.event === 'before-answer');
    return before >= 0 && this.events.slice(before + 1).some(event => event.event === 'nominated');
  }

  snapshot() {
    const generations = new Map();
    if (this.previousUfrag) {
      for (const [label, ordinal] of this.streams) {
        if (label.includes(' - ' + this.previousUfrag + ':')) generations.set(ordinal, 'previous');
        else if (this.freshUfrag && label.includes(' - ' + this.freshUfrag + ':')) generations.set(ordinal, 'fresh');
      }
    }
    return {
      events: this.events.map(event => ({...event, generation: generations.get(event.stream)})),
      omitted: this.omitted
    };
  }
}

export async function fixtureAnswers(signal, fixture, bind) {
  const candidate = currentNativeCandidate(signal.description.sdp);
  const name = candidate.candidate.split(/\s+/)[4];
  if (!fixture.nativeNames.has(name)) throw Error('ordering control requires the owned native name');
  const packet = resolutionAnswer(name, bind);
  fixture.socket.setMulticastInterface(bind);
  fixture.socket.setMulticastLoopback(true);
  fixture.socket.setMulticastTTL(255);
  let sent = 0;
  for (let attempt = 0; attempt < 32; attempt++) {
    await new Promise((resolve, reject) => fixture.socket.send(packet,5353,'224.0.0.251',
      error => error ? reject(error) : resolve()));
    sent++;
    await new Promise(resolve => setTimeout(resolve, 25));
  }
  return sent;
}

export function holdBrowserSignal(signal, held) {
  if (signal.description) {
    return {...signal, description: {...signal.description,
      sdp: signal.description.sdp.replace(/^a=(?:candidate:.*|end-of-candidates)\r?\n/gm, '')}};
  }
  if (held.length >= 32) throw Error('held browser restart signal budget');
  held.push(signal);
  return null;
}
