// Firefox-only, process-local IPC diagnostics. Full profiles never leave Firefox.
export function summarizeMdnsIpc(profile){
  const events=[],counts={querySent:0,queryReceived:0,completionSent:0,completionReceived:0};
  let processes=0,markers=0,omitted=0;
  const origin=profile.meta?.startTime||0;
  function visit(process){
    if(++processes>32)throw Error('Firefox IPC process budget');
    if((process.threads||[]).length>256)throw Error('Firefox IPC thread budget');
    for(const thread of process.threads||[]){
      const table=thread.markers;if(!table?.schema)continue;
      for(const row of table.data||[]){
        if(++markers>200000)throw Error('Firefox IPC marker budget');
        const data=row[table.schema.data];
        if(data?.type!=='IPC'||data.phase!=='endpoint')continue;
        const kind=/PStunAddrsRequest::(?:Msg_)?QueryMDNSHostname(?:$|\b)/.test(data.messageType)?'query':/PStunAddrsRequest::(?:Msg_)?OnMDNSQueryComplete(?:$|\b)/.test(data.messageType)?'completion':null;
        if(!kind||!['sending','receiving'].includes(data.direction)||!['parent','child'].includes(data.side))continue;
        counts[kind+(data.direction==='sending'?'Sent':'Received')]++;
        if(events.length<64){const t=row[table.schema.startTime];events.push({kind,direction:data.direction,side:data.side,ms:Math.round((process.meta?.startTime||origin)-origin+(Number.isFinite(t)?t:0))});}else omitted++;
      }
    }
    for(const child of process.processes||[])visit(child);
  }
  visit(profile);events.sort((a,b)=>a.ms-b.ms);
  return {processes,markers,...counts,events,omitted};
}
export class FirefoxIpcTrace {
  constructor(rpc){this.rpc=rpc;}
  async start(){
    const tree=await this.rpc.call('browsingContext.getTree',{'moz:scope':'chrome'});
    this.context=tree.contexts?.[0]?.context;if(!this.context)throw Error('Firefox diagnostic chrome context unavailable');
    await this.rpc.call('moz:profiler.start',{entries:262144,interval:10,features:['ipcmessages','nostacksampling'],threads:['GeckoMain']});
    this.active=true;
  }
  async snapshot(){
    const expression=`(async()=>JSON.stringify((${summarizeMdnsIpc.toString()})(await Services.profiler.getProfileDataAsync())))()`;
    const result=await this.rpc.call('script.evaluate',{expression,target:{context:this.context},awaitPromise:true});
    if(result.type!=='success'||typeof result.result?.value!=='string')throw Error('Firefox IPC reduction failed');
    return JSON.parse(result.result.value);
  }
  async close(){if(this.active){await this.rpc.call('moz:profiler.stop',{discard:true});this.active=false;}}
}
