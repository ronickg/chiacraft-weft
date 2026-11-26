#!/usr/bin/env python3
"""Run JavaScript inside a running Blockbench (started with --remote-debugging-port, 9333 by
default) and print the result. Blockbench's whole API (Cube, Group, Texture, Codecs, Plugins,
Formats...) is global there.

    .venv/bin/python tools/bb.py 'Formats.hytale_character?.name'
    .venv/bin/python tools/bb.py -f script.js
"""

import json
import os
import subprocess
import sys
import urllib.request

try:
    import websocket
except ImportError:  # no websocket-client: Node's built-in WebSocket does the same
    websocket = None

NODE = r"""
const [url, expr] = [process.argv[1], require("fs").readFileSync(0, "utf8")];
const ws = new WebSocket(url);
ws.onopen = () => ws.send(JSON.stringify({id: 1, method: "Runtime.evaluate",
  params: {expression: expr, awaitPromise: true, returnByValue: true}}));
ws.onmessage = m => { const d = JSON.parse(m.data); if (d.id === 1) { process.stdout.write(JSON.stringify(d)); ws.close(); } };
"""

PORT = int(os.environ.get("BB_PORT", "9333"))


def target():
    tabs = json.load(urllib.request.urlopen(f"http://127.0.0.1:{PORT}/json"))
    page = next(t for t in tabs if t.get("type") == "page")
    return page["webSocketDebuggerUrl"]


def run(js, timeout=300):
    expr = f"(async () => {{ {js} }})()" if "return" in js else js
    if websocket is None:
        r = subprocess.run(["node", "-e", NODE, target()], input=expr, capture_output=True, text=True, timeout=timeout)
        msg = json.loads(r.stdout or "{}")
    else:
        ws = websocket.create_connection(target(), timeout=timeout, suppress_origin=True)
        ws.send(json.dumps({"id": 1, "method": "Runtime.evaluate", "params": {
            "expression": expr, "awaitPromise": True, "returnByValue": True}}))
        while True:
            msg = json.loads(ws.recv())
            if msg.get("id") == 1:
                break
        ws.close()
    res = msg.get("result", {})
    if "exceptionDetails" in res:
        d = res["exceptionDetails"]
        raise SystemExit("JS error: " + (d.get("exception", {}).get("description") or d.get("text", "")))
    return res.get("result", {}).get("value")


if __name__ == "__main__":
    if sys.argv[1] == "-f":
        js = open(sys.argv[2]).read()
    else:
        js = sys.argv[1]
    out = run(js)
    print(out if isinstance(out, str) else json.dumps(out, indent=1))
