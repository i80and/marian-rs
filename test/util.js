'use strict'
/* eslint-env node */
const child_process = require('child_process')
const http = require('http')
const readline = require('readline')

function startServer(serverPath, manifestPath, done) {
    let isDone = false

    const child = child_process.spawn(serverPath, [manifestPath], {
        stdio: [0, 1, 'pipe']
    })

    const rl = readline.createInterface({
        input: child.stderr
    })

    const ctx = {
        child: child,
        host: ''
    }

    rl.on('line', (line) => {
        console.log(`Line: ${line}`)
        if (isDone) { return }

        const match = line.match(/Listening on (\S+)/)
        if (match) {
            ctx.host = match[1]
        } else if (line.match(/Error/)) {
            throw new Error(line)
        }

        isDone = true
        done()
    })

    rl.on('error', (err) => {
        throw err
    })

    rl.on('end', () => {
        rl.close()
    })

    return ctx
}

function request(url, options) {
    if (typeof(url) === 'string') {
        url = new URL(url);
    }

    const request = {
        'hostname': url.hostname,
        'port': parseInt(url.port),
        'path': url.pathname + url.search,
        'method': 'GET'
    }

    Object.assign(request, options)

    return new Promise((resolve, reject) => {
        http.request(request, (res) => {
            res.setEncoding('utf8')
            let data = ''

            res.on('data', (chunk) => { data += chunk })
            res.on('end', () => {
                resolve({
                    response: res,
                    json: data ? JSON.parse(data) : undefined
                })
            })
            res.on('error', (err) => {
                reject(err)
            })
        }).end()
    })
}

exports.request = request
exports.startServer = startServer
