#!/usr/bin/env python3
"""Run JavaScript inside a running Blockbench (started with --remote-debugging-port, 9333 by
default) and print the result. Blockbench's whole API (Cube, Group, Texture, Codecs, Plugins,
Formats...) is global there.

    .venv/bin/python tools/bb.py 'Formats.hytale_character?.name'
    .venv/bin/python tools/bb.py -f script.js
"""

import json
import os
import sys
import urllib.request

import websocket

PORT = int(os.environ.get("BB_PORT", "9333"))


def target():
    tabs = json.load(urllib.request.urlopen(f"http://127.0.0.1:{PORT}/json"))
    page = next(t for t in tabs if t.get("type") == "page")
    return page["webSocketDebuggerUrl"]


def run(js, timeout=300):
    ws = websocket.create_connection(target(), timeout=timeout, suppress_origin=True)
    expr = f"(async () => {{ {js} }})()" if "return" in js else js
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
