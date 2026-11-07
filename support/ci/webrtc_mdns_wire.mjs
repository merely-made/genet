import {spawn} from 'node:child_process';
import {once} from 'node:events';
import {createInterface} from 'node:readline';
import {fileURLToPath} from 'node:url';

const script = fileURLToPath(new URL('./webrtc_mdns_wire.py', import.meta.url));

async function within(promise, ms) {
  let timer;
  try {
    return await Promise.race([promise, new Promise((_, reject) => {
      timer = setTimeout(() => reject(Error('mDNS capture helper deadline')), ms);
    })]);
  } finally {
    clearTimeout(timer);
  }
}

export class MdnsWire {
  async start(selected, alternate) {
    this.child = spawn('python', ['-B', script, selected, alternate],
      {stdio: ['pipe', 'pipe', 'ignore'], windowsHide: true});
    this.child.stdin.on('error', () => {});
    this.done = once(this.child, 'close').catch(() => []);
    this.reader = createInterface({input: this.child.stdout})[Symbol.asyncIterator]();
    const ready = await this.read();
    if (!ready.ready) throw Error('mDNS capture helper did not open adapters');
  }

  async read() {
    const {value, done} = await within(this.reader.next(), 10000);
    if (done) throw Error('mDNS capture helper exited');
    const result = JSON.parse(value);
    if (result.error) throw Error(result.error);
    return result;
  }

  async watch(names) {
    const key = [...names].sort().join(' ');
    if (!key || key === this.key) return;
    this.child.stdin.write(JSON.stringify({op: 'watch', names: [...names]}) + '\n');
    await this.read();
    this.key = key;
  }

  async snapshot(fixture) {
    this.child.stdin.write('{"op":"snapshot"}\n');
    return {...await this.read(), socket: fixture.snapshot()};
  }

  async close() {
    if (!this.child) return;
    this.child.stdin.end();
    try {
      await within(this.done, 2000);
    } catch {
      this.child.kill();
      await within(this.done, 2000);
    }
  }
}
