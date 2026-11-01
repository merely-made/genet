(function(g) {
  'use strict';
  const parse = JSON.parse, stringify = JSON.stringify;
  const createHost = __rtc_new, commandHost = __rtc_command;
  const FormOps = g.__formsPrivate;
  for(const name of ['__rtc_new','__rtc_command']) {
    if(!Reflect.deleteProperty(g,name))g[name]=undefined;
  }
  const PCs = new WeakMap(), DCs = new WeakMap(), live = new Map();
  const descs = new WeakMap(), candidates = new WeakMap();
  const transports = new WeakMap(), transportToken={};
  const PromiseCtor = g.Promise, ArrayBufferCtor = g.ArrayBuffer, U8 = g.Uint8Array;
  const utf8 = new TextEncoder(), decoder = new TextDecoder();
  const encode = utf8.encode.bind(utf8), decode = decoder.decode.bind(decoder);
  function error(name, text) { return name === 'TypeError' ? new TypeError(text) : name === 'SyntaxError' ? new DOMException(text, name) : new DOMException(text, name); }
  function unwrap(text) { const r = parse(text); if (!r.ok) throw error(r.name, r.message); return r.result; }
  function pc(value) { const s = PCs.get(value); if (!s) throw new TypeError('Illegal invocation'); return s; }
  function dc(value) { const s = DCs.get(value); if (!s) throw new TypeError('Illegal invocation'); return s; }
  function integer(value, max) { const n = Math.trunc(Number(value)); if (!Number.isFinite(n) || n < 0 || n > max) throw new TypeError('Integer is out of range'); return n; }
  function ushort(value) { const n=Number(value); return !Number.isFinite(n)||n===0?0:((Math.trunc(n)%65536)+65536)%65536; }
  function string(value) { if (typeof value === 'symbol') throw new TypeError('Symbol cannot be converted to DOMString'); return String(value); }
  function usv(value) { return decode(encode(string(value))); }
  function description(value,local=false) {
    if(local&&value==null)return null;
    if (value === null || typeof value !== 'object') throw new TypeError('A session description is required');
    if(local&&value.type===undefined)return {sdp:value.sdp===undefined?'':string(value.sdp)};
    const type = string(value.type);
    if (!['offer','answer','pranswer','rollback'].includes(type)) throw new TypeError('Invalid SDP type');
    return {type, sdp:value.sdp === undefined ? '' : string(value.sdp)};
  }
  class RTCSessionDescription {
    constructor(init) { descs.set(this, description(init)); }
    get type() { const s=descs.get(this); if(!s) throw new TypeError('Illegal invocation'); return s.type; }
    get sdp() { const s=descs.get(this); if(!s) throw new TypeError('Illegal invocation'); return s.sdp; }
    toJSON() { return {type:this.type,sdp:this.sdp}; }
  }
  function candidate(init) {
    init = init == null ? {} : Object(init);
    const s = {candidate:init.candidate === undefined ? '' : string(init.candidate),
      sdpMid:init.sdpMid == null ? null : string(init.sdpMid),
      sdpMLineIndex:init.sdpMLineIndex == null ? null : ushort(init.sdpMLineIndex),
      usernameFragment:init.usernameFragment == null ? null : string(init.usernameFragment)};
    if (s.sdpMid === null && s.sdpMLineIndex === null) throw new TypeError('A media identifier is required');
    const fields={foundation:null,component:null,priority:null,address:null,protocol:null,port:null,type:null,tcpType:null,relatedAddress:null,relatedPort:null,relayProtocol:null,url:null};
    const words=s.candidate.trim().replace(/^a=/,'').split(/\s+/);
    if (/^candidate:/.test(words[0]) && words.length >= 8 && words[6] === 'typ') {
      fields.foundation=words[0].slice(10); fields.component=words[1] === '1' ? 'rtp' : words[1] === '2' ? 'rtcp' : null;
      fields.protocol=words[2].toLowerCase(); fields.priority=Number(words[3]); fields.address=words[4]; fields.port=Number(words[5]); fields.type=words[7];
      for(let i=8;i+1<words.length;i+=2) {
        if(words[i]==='raddr') fields.relatedAddress=words[i+1];
        if(words[i]==='rport') fields.relatedPort=Number(words[i+1]);
        if(words[i]==='tcptype') fields.tcpType=words[i+1];
      }
    }
    return Object.assign(s,fields);
  }
  class RTCIceCandidate {
    constructor(init={}) { candidates.set(this,candidate(init)); }
    toJSON() { const s=candidates.get(this); if(!s) throw new TypeError('Illegal invocation'); return {candidate:s.candidate,sdpMid:s.sdpMid,sdpMLineIndex:s.sdpMLineIndex,usernameFragment:s.usernameFragment}; }
  }
  for(const name of ['candidate','sdpMid','sdpMLineIndex','usernameFragment','foundation','component','priority','address','protocol','port','type','tcpType','relatedAddress','relatedPort','relayProtocol','url']) {
    Object.defineProperty(RTCIceCandidate.prototype,name,{enumerable:true,configurable:true,get(){const s=candidates.get(this);if(!s)throw new TypeError('Illegal invocation');return s[name];}});
  }
  class RTCPeerConnectionIceEvent extends Event {
    constructor(type,init={}) { super(type,init); const c=init.candidate==null?null:init.candidate; if(c!==null&&!candidates.has(c))throw new TypeError('Invalid candidate'); Object.defineProperty(this,'candidate',{value:c,enumerable:true}); Object.defineProperty(this,'url',{value:init.url===undefined?null:string(init.url),enumerable:true}); }
  }
  class RTCDataChannelEvent extends Event {
    constructor(type,init) { if(!init||!DCs.has(init.channel))throw new TypeError('A data channel is required');super(type,init);Object.defineProperty(this,'channel',{value:init.channel,enumerable:true}); }
  }
  function configuration(input) {
    input=input==null?{}:Object(input);
    const out={iceServers:[],certificates:[],iceTransportPolicy:input.iceTransportPolicy===undefined?'all':string(input.iceTransportPolicy),bundlePolicy:input.bundlePolicy===undefined?'balanced':string(input.bundlePolicy),rtcpMuxPolicy:input.rtcpMuxPolicy===undefined?'require':string(input.rtcpMuxPolicy),iceCandidatePoolSize:input.iceCandidatePoolSize===undefined?0:integer(input.iceCandidatePoolSize,255)};
    if(!['all','relay'].includes(out.iceTransportPolicy)||!['balanced','max-compat','max-bundle'].includes(out.bundlePolicy)||out.rtcpMuxPolicy!=='require')throw new TypeError('Invalid RTC configuration');
    if(input.iceServers!==undefined) for(const server of input.iceServers) {
      if(server.urls===undefined)throw new TypeError('ICE server URLs are required');
      const urls=typeof server.urls==='string'?[server.urls]:Array.from(server.urls,string);
      for(const url of urls) {
        if(!/^(stun|stuns|turn|turns):[^\s/?#]+(?::\d+)?(?:\?transport=(?:udp|tcp))?$/.test(url))throw error('SyntaxError','Invalid ICE server URL');
        if(/^turns?:/.test(url)&&(server.username===undefined||server.credential===undefined))throw error('InvalidAccessError','TURN credentials are required');
      }
      const item={urls};if(server.username!==undefined)item.username=string(server.username);if(server.credential!==undefined)item.credential=string(server.credential);out.iceServers.push(item);
    }
    if(input.certificates!==undefined&&Array.from(input.certificates).length)throw new TypeError('A supported RTCCertificate is required');
    return out;
  }
  function apply(s,state) {
    if(!state)return;
    s.state=state;
    for(const name of ['localDescription','currentLocalDescription','pendingLocalDescription','remoteDescription','currentRemoteDescription','pendingRemoteDescription']) {
      const value=state[name],old=s[name];
      if(value==null)s[name]=null;
      else if(!old||old.type!==value.type||old.sdp!==value.sdp)s[name]=new RTCSessionDescription(value);
    }
    for(const meta of state.channels||[]) { const item=s.channels.get(String(meta.handle));if(item)Object.assign(dc(item).meta,meta); }
    if(state.sctp) {
      if(!s.sctp) {s.ice=new RTCIceTransport(transportToken);s.dtls=new RTCDtlsTransport(transportToken);s.sctp=new RTCSctpTransport(transportToken);transports.get(s.dtls).iceTransport=s.ice;transports.get(s.sctp).transport=s.dtls;}
      Object.assign(transports.get(s.sctp),state.sctp);Object.assign(transports.get(s.dtls),state.sctp.dtls);Object.assign(transports.get(s.ice),state.sctp.dtls.ice);
    }
  }
  function call(s,op,data) { const r=unwrap(commandHost(s.id,op,stringify(data===undefined?null:data)));apply(s,r.state);return r.value; }
  function queued(s,op,data) {
    const result=s.chain.then(function(){if(s.closed)throw error('InvalidStateError','Peer connection is closed');return call(s,op,data);});
    s.chain=result.then(function(){},function(){});return result;
  }
  function handler(proto,type) { const slots=new WeakMap();Object.defineProperty(proto,'on'+type,{enumerable:true,configurable:true,get(){return slots.get(this)||null;},set(fn){const old=slots.get(this);if(old)this.removeEventListener(type,old);if(typeof fn==='function'){slots.set(this,fn);this.addEventListener(type,fn);}else slots.delete(this);}}); }
  function transport(value){const s=transports.get(value);if(!s)throw new TypeError('Illegal invocation');return s;}
  class RTCSctpTransport extends EventTarget {constructor(token){if(token!==transportToken)throw new TypeError('Illegal constructor');super();transports.set(this,{});}}
  class RTCDtlsTransport extends EventTarget {
    constructor(token){if(token!==transportToken)throw new TypeError('Illegal constructor');super();transports.set(this,{});}
    getRemoteCertificates(){return (transport(this).certificates||[]).map(v=>new U8(v).buffer);}
  }
  class RTCIceTransport extends EventTarget {
    constructor(token){if(token!==transportToken)throw new TypeError('Illegal constructor');super();transports.set(this,{});}
    getLocalCandidates(){return (transport(this).localCandidates||[]).map(v=>new RTCIceCandidate(v));}
    getRemoteCandidates(){return (transport(this).remoteCandidates||[]).map(v=>new RTCIceCandidate(v));}
    getLocalParameters(){return parse(stringify(transport(this).localParameters||null));}
    getRemoteParameters(){return parse(stringify(transport(this).remoteParameters||null));}
  }
  for(const [type,attributes] of [[RTCSctpTransport,['state','maxMessageSize','maxChannels','transport']],[RTCDtlsTransport,['state','iceTransport']],[RTCIceTransport,['state','gatheringState','role','component']]]) {
    for(const name of attributes)Object.defineProperty(type.prototype,name,{enumerable:true,configurable:true,get(){return transport(this)[name];}});
    handler(type.prototype,'statechange');
  }
  const channelToken={};
  class RTCDataChannel extends EventTarget {
    constructor(token,owner,meta) { if(token!==channelToken)throw new TypeError('Illegal constructor');super();DCs.set(this,{owner,meta:Object.assign({readyState:'connecting',bufferedAmount:0,id:null},meta),binaryType:'arraybuffer',low:0}); }
    get binaryType(){return dc(this).binaryType;}
    set binaryType(value){const s=dc(this);value=string(value);if(value==='arraybuffer'||value==='blob')s.binaryType=value;}
    get bufferedAmountLowThreshold(){return dc(this).low;}
    set bufferedAmountLowThreshold(value){const s=dc(this);s.low=integer(value,4294967295);call(s.owner,'bufferedAmountLowThreshold',{channel:s.meta.handle,value:s.low});}
    send(data) {
      const s=dc(this);if(s.meta.readyState!=='open')throw error('InvalidStateError','Data channel is not open');
      let bytes,text=false;
      if(data instanceof ArrayBufferCtor)bytes=Array.from(new U8(data));
      else if(ArrayBufferCtor.isView(data))bytes=Array.from(new U8(data.buffer,data.byteOffset,data.byteLength));
      else if(FormOps.isBlob(data))bytes=Array.from(FormOps.blob(data).bytes);
      else {text=true;bytes=Array.from(encode(string(data)));}
      call(s.owner,'send',{channel:s.meta.handle,isString:text,data:bytes});
    }
    close(){const s=dc(this);if(s.meta.readyState==='closed'||s.meta.readyState==='closing')return;call(s.owner,'closeDataChannel',{channel:s.meta.handle});s.meta.readyState='closing';}
  }
  for(const name of ['label','ordered','maxPacketLifeTime','maxRetransmits','protocol','negotiated','id','readyState','bufferedAmount'])Object.defineProperty(RTCDataChannel.prototype,name,{enumerable:true,configurable:true,get(){return dc(this).meta[name];}});
  for(const name of ['open','message','closing','close','error','bufferedamountlow'])handler(RTCDataChannel.prototype,name);
  class RTCPeerConnection extends EventTarget {
    constructor(input={}) {
      super();const config=configuration(input),created=unwrap(createHost(stringify(config)));
      const s={id:created.id,closed:false,chain:PromiseCtor.resolve(),channels:new Map(),state:{},config};
      PCs.set(this,s);live.set(s.id,this);apply(s,callState(s));
    }
    createOffer(options={}){const s=pc(this);let value;try{options=options==null?{}:Object(options);value={iceRestart:!!options.iceRestart};}catch(e){return PromiseCtor.reject(e);}return queued(s,'createOffer',value);}
    createAnswer(options={}){return queued(pc(this),'createAnswer',{});}
    setLocalDescription(init){const s=pc(this);let value;try{value=description(init,true);}catch(e){return PromiseCtor.reject(e);}return queued(s,'setLocalDescription',value).then(function(){});}
    setRemoteDescription(init){const s=pc(this);let value;try{value=description(init);}catch(e){return PromiseCtor.reject(e);}return queued(s,'setRemoteDescription',value).then(function(){});}
    addIceCandidate(init){const s=pc(this);let value;try{value=init==null?null:{candidate:init.candidate===undefined?'':string(init.candidate),sdpMid:init.sdpMid==null?null:string(init.sdpMid),sdpMLineIndex:init.sdpMLineIndex==null?null:integer(init.sdpMLineIndex,65535),usernameFragment:init.usernameFragment==null?null:string(init.usernameFragment)};if(value&&value.candidate&&value.sdpMid===null&&value.sdpMLineIndex===null)throw new TypeError('A media identifier is required');}catch(e){return PromiseCtor.reject(e);}return queued(s,'addIceCandidate',value).then(function(){});}
    restartIce(){const s=pc(this);if(s.closed)throw error('InvalidStateError','Peer connection is closed');call(s,'restartIce',null);}
    getConfiguration(){return parse(stringify(pc(this).config));}
    setConfiguration(input={}){const s=pc(this);if(s.closed)throw error('InvalidStateError','Peer connection is closed');const config=configuration(input);call(s,'setConfiguration',config);s.config=config;}
    createDataChannel(label,options={}) {
      const s=pc(this);if(arguments.length===0)throw new TypeError('Label is required');if(s.closed)throw error('InvalidStateError','Peer connection is closed');
      options=options==null?{}:Object(options);
      label=usv(label);const init={label,ordered:options.ordered===undefined?true:!!options.ordered,protocol:options.protocol===undefined?'':usv(options.protocol),negotiated:!!options.negotiated,maxPacketLifeTime:options.maxPacketLifeTime===undefined?null:ushort(options.maxPacketLifeTime),maxRetransmits:options.maxRetransmits===undefined?null:ushort(options.maxRetransmits),id:options.id===undefined?null:integer(options.id,65535)};
      if(encode(init.label).length>65535||encode(init.protocol).length>65535)throw new TypeError('Channel string is too long');
      if(init.maxPacketLifeTime!==null&&init.maxRetransmits!==null)throw new TypeError('Conflicting reliability options');
      if(init.id===65535)throw new TypeError('Reserved channel ID');if(init.negotiated&&init.id===null)throw new TypeError('Negotiated channel ID is required');
      const meta=call(s,'createDataChannel',init),channel=new RTCDataChannel(channelToken,s,meta);s.channels.set(String(meta.handle),channel);return channel;
    }
    getStats(selector=null){const s=pc(this);if(selector!==null)return PromiseCtor.reject(error('InvalidAccessError','Unknown media track'));if(s.closed)return PromiseCtor.resolve(new Map());return queued(s,'getStats',null).then(function(entries){return new Map(entries.map(function(e){return [e.id,e];}));});}
    close(){const s=pc(this);if(s.closed)return;call(s,'close',null);s.closed=true;s.state.signalingState='closed';s.state.connectionState='closed';s.state.iceConnectionState='closed';for(const ch of s.channels.values())dc(ch).meta.readyState='closed';for(const t of [s.sctp,s.dtls,s.ice])if(t)transport(t).state='closed';live.delete(s.id);}
  }
  function callState(s){return unwrap(commandHost(s.id,'snapshot','null')).state;}
  for(const name of ['localDescription','currentLocalDescription','pendingLocalDescription','remoteDescription','currentRemoteDescription','pendingRemoteDescription'])Object.defineProperty(RTCPeerConnection.prototype,name,{enumerable:true,configurable:true,get(){return pc(this)[name];}});
  for(const name of ['signalingState','iceGatheringState','iceConnectionState','connectionState','canTrickleIceCandidates'])Object.defineProperty(RTCPeerConnection.prototype,name,{enumerable:true,configurable:true,get(){return pc(this).state[name];}});
  Object.defineProperty(RTCPeerConnection.prototype,'sctp',{enumerable:true,configurable:true,get(){return pc(this).sctp||null;}});
  for(const name of ['negotiationneeded','icecandidate','icecandidateerror','signalingstatechange','iceconnectionstatechange','icegatheringstatechange','connectionstatechange','datachannel','track'])handler(RTCPeerConnection.prototype,name);
  Object.defineProperty(g,'__rtcDeliver',{configurable:true,value:function(id,text){
    const owner=live.get(id);if(!owner)return;const s=pc(owner),n=parse(text);apply(s,n.state);
    if(n.event==='transportstatechange'){const t=s[n.transport];if(t)t.dispatchEvent(new Event('statechange'));return;}
    if(n.channel){const key=String(n.channel.handle);let channel=s.channels.get(key);if(!channel){channel=new RTCDataChannel(channelToken,s,n.channel);s.channels.set(key,channel);owner.dispatchEvent(new RTCDataChannelEvent('datachannel',{channel}));}Object.assign(dc(channel).meta,n.channel);if(n.event==='open'||n.event==='close'||n.event==='closing'||n.event==='bufferedamountlow')channel.dispatchEvent(new Event(n.event));return;}
    if(n.event==='message'){const channel=s.channels.get(String(n.handle));if(!channel)return;const bytes=new U8(n.data);const value=n.isString?decode(bytes):dc(channel).binaryType==='blob'?FormOps.makeBlob(bytes,''):bytes.buffer;channel.dispatchEvent(new MessageEvent('message',{data:value}));return;}
    if(n.event==='icecandidate'){const value=n.candidate===null?null:new RTCIceCandidate(n.candidate);owner.dispatchEvent(new RTCPeerConnectionIceEvent('icecandidate',{candidate:value}));return;}
    if(n.event)owner.dispatchEvent(new Event(n.event));
  }});
  for(const [name,value] of Object.entries({RTCSessionDescription,RTCIceCandidate,RTCPeerConnectionIceEvent,RTCDataChannelEvent,RTCDataChannel,RTCPeerConnection,RTCSctpTransport,RTCDtlsTransport,RTCIceTransport})) {
    Object.defineProperty(value.prototype,Symbol.toStringTag,{value:name,configurable:true});Object.defineProperty(g,name,{value,writable:true,configurable:true});
  }
})(globalThis);
