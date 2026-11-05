// Read-only bounded DNS observation. It never produces a response or a name.
export function observeNames(packet,watched,bind) {
  const result={queries:0,answers:0};
  if(packet.length<12||packet.length>1500)return result;
  const counts=[4,6,8,10].map(i=>packet.readUInt16BE(i));
  if(counts[0]>8||counts.slice(1).reduce((a,b)=>a+b,0)>16)return result;
  function name(start,depth=0) {
    if(depth>8)throw Error();
    const labels=[];let at=start;
    while(true){
      if(at>=packet.length)throw Error();
      const size=packet[at++];if(!size)break;
      if((size&0xc0)===0xc0){if(at>=packet.length)throw Error();const next=name(((size&63)<<8)|packet[at++],depth+1);labels.push(next[0]);break;}
      if(size>63||at+size>packet.length)throw Error();
      labels.push(packet.subarray(at,at+size).toString('ascii'));at+=size;
      if(labels.join('.').length>255)throw Error();
    }
    return [labels.join('.').toLowerCase(),at];
  }
  try {
    let at=12;const response=!!(packet.readUInt16BE(2)&0x8000);
    for(let i=0;i<counts[0];i++){
      let key;[key,at]=name(at);if(at+4>packet.length)throw Error();
      const type=packet.readUInt16BE(at),cls=packet.readUInt16BE(at+2)&0x7fff;at+=4;
      if(!response&&watched.has(key)&&[1,255].includes(type)&&[1,255].includes(cls))result.queries++;
    }
    for(let i=0;i<counts.slice(1).reduce((a,b)=>a+b,0);i++){
      let key;[key,at]=name(at);if(at+10>packet.length)throw Error();
      const type=packet.readUInt16BE(at),cls=packet.readUInt16BE(at+2)&0x7fff,ttl=packet.readUInt32BE(at+4),length=packet.readUInt16BE(at+8);at+=10;
      if(at+length>packet.length)throw Error();
      if(response&&watched.has(key)&&type===1&&cls===1&&ttl>0&&length===4&&[...packet.subarray(at,at+4)].join('.')===bind)result.answers++;
      at+=length;
    }
    return at===packet.length?result:{queries:0,answers:0};
  } catch { return {queries:0,answers:0}; }
}
