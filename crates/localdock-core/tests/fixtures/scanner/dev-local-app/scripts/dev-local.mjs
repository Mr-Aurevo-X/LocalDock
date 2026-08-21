import http from 'node:http'

const port = Number(process.env.PORT || 4180)
http.createServer((_req, res) => {
  res.end('ok')
}).listen(port, '127.0.0.1')
