// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

// A static origin for the G5 realms headed receipt.
//
// The fixture needs a parent and a child iframe that are *same-origin*: this
// engine gives every `file:` URL an opaque origin (browsing-context `lib.rs`,
// `Origin::of_url`), and an opaque origin is same-origin with nothing, itself
// included. So the fixture cannot be driven from the filesystem at all — the
// parent could not reach `frame.contentDocument`. One loopback http origin is
// the whole of what this server exists to provide; it serves the fixture
// directory and nothing else, and it records each path it served so the
// receipt can show that both child documents were actually fetched.

import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { createServer } from "node:http";
import { basename, join } from "node:path";

const values = new Map();
for (let index = 2; index < process.argv.length; index += 2) {
  values.set(process.argv[index], process.argv[index + 1]);
}
const artifact = values.get("--artifact");
const port = Number(values.get("--port"));
if (!artifact || !Number.isInteger(port)) throw new Error("usage: --artifact DIR --port PORT");

const fixtureRoot = join(process.cwd(), "ports", "ortet", "tests", "native", "realms");
const types = new Map([
  [".html", "text/html; charset=utf-8"],
  [".js", "text/javascript; charset=utf-8"],
  [".css", "text/css; charset=utf-8"],
]);
const served = [];

function record(path) {
  served.push({ sequence: served.length + 1, path });
  writeFileSync(join(artifact, "server-requests.json"), JSON.stringify(served, null, 2));
}

const server = createServer((request, response) => {
  const url = new URL(request.url, `http://127.0.0.1:${port}`);
  // Basename only: this server never walks out of the fixture directory, and a
  // path that is not a fixture is a 404 rather than a filesystem read.
  const name = basename(url.pathname);
  const extension = name.slice(name.lastIndexOf("."));
  const file = join(fixtureRoot, name);
  if (!types.has(extension) || !existsSync(file)) {
    record(`${url.pathname} 404`);
    response.writeHead(404, { "content-type": "text/plain; charset=utf-8", "cache-control": "no-store" });
    response.end("not a fixture");
    return;
  }
  record(url.pathname);
  response.writeHead(200, { "content-type": types.get(extension), "cache-control": "no-store" });
  response.end(readFileSync(file));
});

server.listen(port, "127.0.0.1");
