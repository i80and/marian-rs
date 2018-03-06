NPM ?= $(shell which npm)
NODE ?= $(shell which node)
MOCHA ?= ./node_modules/.bin/mocha

.PHONY: test debug release

test: debug
	cargo test
	${MOCHA} --timeout 5000 test/integration_test.js

debug:
	cargo build

release:
	cargo build --release

node_modules/.CURRENT: package.json
	${NPM} -s install
	touch $@
