import http.server
from http.server import HTTPServer, BaseHTTPRequestHandler
import socketserver

PORT = 8003

Handler = http.server.SimpleHTTPRequestHandler

Handler.extensions_map[".wasm"] = "application/wasm"

httpd = socketserver.TCPServer(("", PORT), Handler)

print("serving at port", PORT)
httpd.serve_forever()
