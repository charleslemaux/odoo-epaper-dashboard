"""Minimal MCP Streamable-HTTP client for the local Fusion 360 MCP server.

Usage:
  python mcp_client.py list
  python mcp_client.py call <tool_name> <json_args_file_or_inline_json>
"""
import json
import sys
import urllib.request

BASE = "http://127.0.0.1:27182/mcp"
SESSION_FILE = __file__ + ".session"


def post(payload, session_id=None, timeout=120):
    headers = {
        "Content-Type": "application/json",
        "Accept": "application/json, text/event-stream",
    }
    if session_id:
        headers["Mcp-Session-Id"] = session_id
    req = urllib.request.Request(BASE, json.dumps(payload).encode(), headers)
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        sid = resp.headers.get("Mcp-Session-Id")
        ctype = resp.headers.get("Content-Type", "")
        body = resp.read().decode("utf-8", errors="replace")
    if "text/event-stream" in ctype:
        result = None
        for line in body.splitlines():
            if line.startswith("data:"):
                chunk = line[5:].strip()
                if chunk:
                    result = json.loads(chunk)
        return result, sid
    return (json.loads(body) if body.strip() else None), sid


def initialize():
    payload = {
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": {
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": {"name": "claude-code-manual", "version": "1.0"},
        },
    }
    result, sid = post(payload)
    if result and "result" in result:
        info = result["result"].get("serverInfo", {})
        print(f"# server: {info.get('name')} {info.get('version')} "
              f"(proto {result['result'].get('protocolVersion')})", file=sys.stderr)
    post({"jsonrpc": "2.0", "method": "notifications/initialized"}, sid)
    if sid:
        with open(SESSION_FILE, "w") as f:
            f.write(sid)
    return sid


def get_session():
    try:
        with open(SESSION_FILE) as f:
            return f.read().strip()
    except OSError:
        return initialize()


def rpc(method, params, retry=True):
    sid = get_session()
    payload = {"jsonrpc": "2.0", "id": 2, "method": method, "params": params}
    try:
        result, _ = post(payload, sid)
    except urllib.error.HTTPError as exc:
        if retry and exc.code in (400, 404):
            sid = initialize()
            result, _ = post(payload, sid)
        else:
            raise
    if result is None:
        return None
    if "error" in result:
        print("RPC ERROR:", json.dumps(result["error"], indent=2))
        sys.exit(1)
    return result.get("result")


def main():
    cmd = sys.argv[1] if len(sys.argv) > 1 else "list"
    if cmd == "init":
        initialize()
    elif cmd == "list":
        res = rpc("tools/list", {})
        for tool in res.get("tools", []):
            schema = tool.get("inputSchema", {})
            props = schema.get("properties", {})
            req = set(schema.get("required", []))
            args = ", ".join(
                p + ("" if p in req else "?") for p in props)
            print(f"{tool['name']}({args})")
            desc = (tool.get("description") or "").strip().replace("\n", " ")
            print(f"    {desc[:220]}")
    elif cmd == "schema":
        res = rpc("tools/list", {})
        for tool in res.get("tools", []):
            if tool["name"] == sys.argv[2]:
                print(json.dumps(tool, indent=2))
    elif cmd == "run":
        # run a Fusion python script file via the execute tool
        with open(sys.argv[2], encoding="utf-8") as f:
            code = f.read()
        res = rpc("tools/call", {
            "name": "fusion_mcp_execute",
            "arguments": {"featureType": "script", "object": {"script": code}},
        })
        for item in (res or {}).get("content", []):
            if item.get("type") == "text":
                print(item["text"])
        if res and res.get("isError"):
            print("** SCRIPT FAILED **")
            sys.exit(1)
    elif cmd == "shot":
        out = sys.argv[2]
        direction = sys.argv[3] if len(sys.argv) > 3 else "current"
        args = {"queryType": "screenshot", "width": 900, "height": 700,
                "transparentBackground": False, "direction": direction}
        res = rpc("tools/call", {"name": "fusion_mcp_read", "arguments": args})
        import base64
        saved = False
        for item in (res or {}).get("content", []):
            data = None
            if item.get("type") == "image":
                data = item.get("data") or item.get("base64Data")
            elif item.get("type") == "text" and '"base64Data"' in item.get("text", ""):
                data = json.loads(item["text"]).get("base64Data")
            if data:
                with open(out, "wb") as f:
                    f.write(base64.b64decode(data))
                print("saved", out)
                saved = True
        if not saved:
            print("NO IMAGE:", json.dumps(res)[:500])
            sys.exit(1)
    elif cmd == "call":
        name = sys.argv[2]
        raw = sys.argv[3] if len(sys.argv) > 3 else "{}"
        if raw.endswith(".json"):
            with open(raw, encoding="utf-8") as f:
                raw = f.read()
        args = json.loads(raw)
        res = rpc("tools/call", {"name": name, "arguments": args})
        if res is None:
            print("(no response)")
            return
        for item in res.get("content", []):
            if item.get("type") == "text":
                print(item["text"])
            else:
                print(json.dumps(item)[:400])
        if res.get("isError"):
            print("** tool reported isError **")
            sys.exit(1)
    else:
        print("unknown command", cmd)
        sys.exit(2)


if __name__ == "__main__":
    main()
