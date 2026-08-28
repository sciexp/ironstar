"""Stub identity-token endpoint for the effect-run-context check.

Serves the contract the effects depend on: POST /api/v1/id-token, bearer
request token, JSON body carrying an `audience`, response `{"token": ...}`
whose token is a JWS compact serialization whose payload segment holds the
claims. The signature segment is a fixed placeholder because the resolver
reads claims from the payload rather than verifying them; a request that gets
the method, path, bearer credential, content type, or audience wrong is
rejected here, so a passing check also exercises the client side of the
contract.

Configuration is by environment: STUB_TASK_TOKEN, STUB_AUDIENCE, STUB_CLAIMS
(JSON object merged into the emitted claims), STUB_URL_FILE (the request URL
is written there once the socket is bound, so the caller need not guess a
port).
"""

import base64
import json
import os
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

PATH = "/api/v1/id-token"
TASK_TOKEN = os.environ["STUB_TASK_TOKEN"]
AUDIENCE = os.environ["STUB_AUDIENCE"]
CLAIMS = json.loads(os.environ["STUB_CLAIMS"])
URL_FILE = os.environ["STUB_URL_FILE"]


def b64url(raw: bytes) -> str:
    return base64.urlsafe_b64encode(raw).rstrip(b"=").decode("ascii")


def mint(audience: str) -> str:
    header = {"alg": "RS256", "kid": "stub", "typ": "JWT"}
    claims = dict(CLAIMS)
    claims["aud"] = audience
    return ".".join(
        [
            b64url(json.dumps(header).encode()),
            b64url(json.dumps(claims).encode()),
            b64url(b"stub-signature"),
        ]
    )


class Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def do_POST(self) -> None:
        if self.path != PATH:
            self.fail(404, f"unexpected path {self.path}")
            return
        if self.headers.get("Authorization") != f"Bearer {TASK_TOKEN}":
            self.fail(401, "invalid task token")
            return
        if self.headers.get("Content-Type") != "application/json":
            self.fail(415, "expected application/json")
            return

        length = int(self.headers.get("Content-Length", "0"))
        try:
            body = json.loads(self.rfile.read(length) or b"{}")
        except json.JSONDecodeError:
            self.fail(400, "body is not JSON")
            return
        if body.get("audience") != AUDIENCE:
            self.fail(403, "audience not declared in the effect's idTokenAudiences")
            return

        self.respond(
            200,
            {
                "token": mint(body["audience"]),
                "expires_at": "2026-01-01T00:00:00+00:00",
            },
        )

    def do_GET(self) -> None:
        self.fail(405, "id-token requests are POST")

    def fail(self, status: int, message: str) -> None:
        self.respond(status, {"error": message})

    def respond(self, status: int, payload: dict) -> None:
        body = json.dumps(payload).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *args: object) -> None:
        pass


server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
with open(URL_FILE, "w", encoding="utf-8") as handle:
    handle.write(f"http://127.0.0.1:{server.server_address[1]}{PATH}")
server.serve_forever()
