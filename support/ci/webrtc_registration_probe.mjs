// Independent post-failure probes. These never provide a name answer and must
// not count as evidence that the browser itself consumed an advertisement.
import {createSocket} from 'node:dgram';
import {randomBytes} from 'node:crypto';
import {observeNames} from './webrtc_mdns_observer.mjs';
export function validRegistrationReply(packet,from,name,bind,id){
  return from.address===bind&&from.port===5353&&packet.length>=12&&packet.readUInt16BE(0)===id&&observeNames(packet,new Set([name]),bind).answers>0;
}
export async function probeRegistration(name,bind,sharedSocket){
  const result={};
  for(const mode of ['multicastLegacy','unicastLegacy','multicastShared']){
    const shared=mode==='multicastShared',socket=shared?sharedSocket:createSocket('udp4');
    const id=shared?0:randomBytes(2).readUInt16BE();
    try {
      if(!shared)await new Promise((resolve,reject)=>{socket.once('error',reject);socket.bind(0,bind,resolve);});
      socket.setMulticastInterface(bind);socket.setMulticastLoopback(true);
      const header=Buffer.alloc(12);header.writeUInt16BE(id,0);header.writeUInt16BE(1,4);
      const labels=Buffer.concat([...name.split('.').map(s=>Buffer.concat([Buffer.from([s.length]),Buffer.from(s)])),Buffer.from([0,0,1,0,1])]);
      result[mode]=await new Promise(resolve=>{
        let finished=false;const finish=answered=>{if(finished)return;finished=true;clearTimeout(timer);socket.off('message',receive);resolve({answered});};
        const receive=(packet,from)=>{if(validRegistrationReply(packet,from,name,bind,id))finish(true);};
        const timer=setTimeout(()=>finish(false),2500);socket.on('message',receive);
        socket.send(Buffer.concat([header,labels]),5353,mode==='unicastLegacy'?bind:'224.0.0.251',error=>{if(error)finish(false);});
      });
    } catch {result[mode]={answered:false,ioError:true};}
    finally {if(!shared)socket.close();}
  }
  return result;
}
