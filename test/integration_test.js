#!/usr/bin/env node
/* eslint-env node, mocha */
'use strict'

const assert = require('assert')
const process = require('process')

const testUtil = require('./util.js')

describe('integration', function() {
    this.slow(100)

    let ctx = null

    before('starting server', function(done) {
        ctx = testUtil.startServer('target/debug/marian-rust', 'dir:test/manifests/', done)
    })

    let lastSync
    function testFunctionality() {
        it('should return proper /status document', async () => {
            const result = await testUtil.request(`${ctx.host}/status`)
            assert.strictEqual(result.response.statusCode, 200)
            assert.strictEqual(result.response.headers['content-type'], 'application/json')
            assert.ok(result.json.lastSync.finished)
            lastSync = result.json.lastSync.finished
            assert.deepStrictEqual(result.json.manifests.sort(), ['atlas-master', 'bi-connector-master'])
        })

        it('should return proper results for a normal query', async () => {
            const result = await testUtil.request(`${ctx.host}/search?q=${encodeURIComponent('"connect dialog" compass')}`)
            assert.strictEqual(result.response.statusCode, 200)
            assert.strictEqual(result.response.headers['content-type'], 'application/json')
            assert.deepStrictEqual(result.json, {'results':[{'title':'Connect via Compass — MongoDB Atlas','preview':'The Connect dialog for a cluster provides the details to connect to a cluster via Compass.','url':'https://docs.atlas.mongodb.com/compass-connection/'},{'title':'Connect via Driver — MongoDB Atlas','preview':'The Connect dialog for a cluster provides the details to connect to a cluster with an application using a MongoDB driver.','url':'https://docs.atlas.mongodb.com/driver-connection/'},{'title':'Connect via mongo Shell — MongoDB Atlas','preview':'The Connect dialog for a cluster provides the details to connect to a cluster via the mongo shell.','url':'https://docs.atlas.mongodb.com/mongo-shell-connection/'},{'title':'Connect to a Cluster — MongoDB Atlas','preview':'Atlas provides instructions on connecting to a cluster via the mongo shell, a MongoDB driver, or MongoDB Compass via the Atlas UI.','url':'https://docs.atlas.mongodb.com/connect-to-cluster/'},{'title':'Set up VPC Peering Connection — MongoDB Atlas','preview':'For Atlas clusters deployed on Google Cloud Platform or Microsoft Azure, add the IP addresses of your GCP or Azure services to Atlas group IP whitelist to grant those services access to the cluster.','url':'https://docs.atlas.mongodb.com/security-vpc-peering/'},{'title':'Connect from Tableau Desktop — MongoDB Connector for BI 2.2','preview':'The MongoDB Connector for BI is a named connector in Tableau.','url':'https://docs.mongodb.com/bi-connector/current/connect/tableau/'},{'title':'Load File with mongoimport — MongoDB Atlas','preview':'You can use mongoimport to import data from a JSON or a CSV file into MongoDB Atlas cluster.','url':'https://docs.atlas.mongodb.com/import/mongoimport/'},{'title':'Migrate with mongomirror — MongoDB Atlas','preview':'mongomirror is a utility for migrating data from an existing MongoDB replica set to a MongoDB Atlas replica set. mongomirror does not require you to shut down your existing replica set or applications.','url':'https://docs.atlas.mongodb.com/import/mongomirror/'},{'title':'MongoDB Atlas — MongoDB Atlas','preview':'MongoDB Atlas is a cloud service for running, monitoring, and maintaining MongoDB deployments, including the provisioning of dedicated servers for the MongoDB instances. In addition, Atlas provides the ability to introspect collections, query backups, and migrate data from existing MongoDB replica set into an Atlas cluster.','url':'https://docs.atlas.mongodb.com/'}],'spellingCorrections':{}})
        })

        // Test variants of searchProperty
        it('should properly handle searchProperty', async () => {
            let result = await testUtil.request(`${ctx.host}/search?q=aggregation`)
            assert.strictEqual(result.response.statusCode, 200)
            assert.strictEqual(result.response.headers['content-type'], 'application/json')
            assert.deepStrictEqual(result.json, {'results':[{'title':'Schema Configuration — MongoDB Connector for BI 2.2','preview':'Business intelligence tools connect to a data source and, given a fixed tabular schema, allow the user to visually explore their data. As MongoDB uses a flexible schema, these tools currently cannot use MongoDB as a native data source.','url':'https://docs.mongodb.com/bi-connector/current/schema-configuration/'},{'title':'Supported SQL Functions and Operators — MongoDB Connector for BI 2.2','preview':'MongoDB Connector for BI Version 2.2 is compatible with SQL-99 SELECT statements.','url':'https://docs.mongodb.com/bi-connector/current/supported-operations/'},{'title':'MongoDB Reference — MongoDB Atlas','preview':'For a comprehensive documentation of MongoDB, refer to the MongoDB Manual. The following sections in the manual provide some starting points for developing with MongoDB.','url':'https://docs.atlas.mongodb.com/mongodb-reference/'},{'title':'Command Limitations in Free Tier Clusters — MongoDB Atlas','preview':'Atlas Free Tier clusters do not support all functionality available to other clusters.','url':'https://docs.atlas.mongodb.com/unsupported-commands/'},{'title':'Monitor a Cluster — MongoDB Atlas','preview':'Atlas collects and displays metrics for your servers, databases, and MongoDB processes. Atlas displays three charts in the Clusters view and additional charts in the Metrics view.','url':'https://docs.atlas.mongodb.com/monitor-cluster-metrics/'},{'title':'mongodrdl — MongoDB Connector for BI 2.2','preview':'The mongodrdl command man page.','url':'https://docs.mongodb.com/bi-connector/current/reference/mongodrdl/'},{'title':'Create a Cluster — MongoDB Atlas','preview':'Atlas-managed MongoDB deployments, or “clusters”, can be either a replica set or a sharded cluster.','url':'https://docs.atlas.mongodb.com/create-new-cluster/'},{'title':'Query a Backup Snapshot — MongoDB Atlas','preview':'Atlas provides queryable backups. This functionality allows you to query specific backup snapshot. You can use the queryable backups to:','url':'https://docs.atlas.mongodb.com/query-backup/'},{'title':'FAQ: The MongoDB Connector for BI — MongoDB Connector for BI 2.2','preview':'Changed in version 2.0: Prior to version 2.0, the MongoDB Connector for BI stored its own separate set of credentials.','url':'https://docs.mongodb.com/bi-connector/current/faq/'},{'title':'Release Notes for MongoDB Connector for BI — MongoDB Connector for BI 2.2','preview':'Supports authenticating directly against MongoDB using the new C and JDBC authentication plugins. These plugins support SCRAM-SHA-1 and PLAIN mechanisms and remove the SSL requirement for authentication. The authentication plugins can be found on GitHub:','url':'https://docs.mongodb.com/bi-connector/current/release-notes/'},{'title':'mongosqld — MongoDB Connector for BI 2.2','preview':'The mongosqld command man page.','url':'https://docs.mongodb.com/bi-connector/current/reference/mongosqld/'}],'spellingCorrections':{}})

            const result2 = await testUtil.request(`${ctx.host}/search?q=aggregation&searchProperty=atlas-master,bi-connector-master`)
            assert.deepStrictEqual(result.json, result2.json)

            result = await testUtil.request(`${ctx.host}/search?q=aggregation&searchProperty=bi-connector-master`)
            assert.strictEqual(result.response.statusCode, 200)
            assert.strictEqual(result.response.headers['content-type'], 'application/json')
            assert.deepStrictEqual(result.json, {'results':[{'title':'Schema Configuration — MongoDB Connector for BI 2.2','preview':'Business intelligence tools connect to a data source and, given a fixed tabular schema, allow the user to visually explore their data. As MongoDB uses a flexible schema, these tools currently cannot use MongoDB as a native data source.','url':'https://docs.mongodb.com/bi-connector/current/schema-configuration/'},{'title':'Supported SQL Functions and Operators — MongoDB Connector for BI 2.2','preview':'MongoDB Connector for BI Version 2.2 is compatible with SQL-99 SELECT statements.','url':'https://docs.mongodb.com/bi-connector/current/supported-operations/'},{'title':'mongodrdl — MongoDB Connector for BI 2.2','preview':'The mongodrdl command man page.','url':'https://docs.mongodb.com/bi-connector/current/reference/mongodrdl/'},{'title':'FAQ: The MongoDB Connector for BI — MongoDB Connector for BI 2.2','preview':'Changed in version 2.0: Prior to version 2.0, the MongoDB Connector for BI stored its own separate set of credentials.','url':'https://docs.mongodb.com/bi-connector/current/faq/'},{'title':'Release Notes for MongoDB Connector for BI — MongoDB Connector for BI 2.2','preview':'Supports authenticating directly against MongoDB using the new C and JDBC authentication plugins. These plugins support SCRAM-SHA-1 and PLAIN mechanisms and remove the SSL requirement for authentication. The authentication plugins can be found on GitHub:','url':'https://docs.mongodb.com/bi-connector/current/release-notes/'},{'title':'mongosqld — MongoDB Connector for BI 2.2','preview':'The mongosqld command man page.','url':'https://docs.mongodb.com/bi-connector/current/reference/mongosqld/'}],'spellingCorrections':{}})

            const result3 = await testUtil.request(`${ctx.host}/search?q=aggregation&searchProperty=bi-connector-alias`)
            assert.deepStrictEqual(result.json, result3.json)
        })

        it('should return 304 if index hasn\'t changed', async () => {
            const result = await testUtil.request({
                host: ctx.host,
                path: `/search?q=${encodeURIComponent('quary')}`,
                headers: {
                    'If-Modified-Since': new Date().toUTCString()
                }})
            assert.strictEqual(result.response.statusCode, 304)
        })

        it('should NOT return 304 if index has changed', async () => {
            const result = await testUtil.request({
                host: ctx.host,
                path: `/search?q=${encodeURIComponent('quary')}`,
                headers: {
                    'If-Modified-Since': new Date(0).toUTCString()
                }})
            assert.strictEqual(result.response.statusCode, 200)
        })
    }

    it('should print host to stdout', () => {
        assert.ok(ctx.host)
    })

    testFunctionality()

    it('should return 200 to /refresh', async function() {
        this.slow(5000)
        const result = await testUtil.request({
            method: 'post',
            host: ctx.host,
            path: '/refresh'})
        assert.strictEqual(result.response.statusCode, 200)

        await new Promise((resolve, reject) => {
            const intervalID = setInterval(async () => {
                const result = await testUtil.request({
                    host: ctx.host,
                    path: '/status'})

                try {
                    assert.strictEqual(result.response.statusCode, 200)
                } catch (err) {
                    reject(err)
                    return
                }

                if (result.json.lastSync.finished > lastSync) {
                    clearInterval(intervalID)
                    resolve()
                }
            }, 100)
        })
    })

    after('shutting down', function() {
        process.kill(ctx.child.pid, 'SIGINT')
    })
})
