# Changelog

## [0.6.0](https://github.com/iExec-Nox/nox-runner/compare/v0.5.0...v0.6.0) (2026-04-24)


### Features

* expose first Nox metrics to Prometheus ([#35](https://github.com/iExec-Nox/nox-runner/issues/35)) ([21fe706](https://github.com/iExec-Nox/nox-runner/commit/21fe706f8c9dfd99ebbf87fbd39c290d4a903c78))
* rework and add new parameters to NATS configuration ([#39](https://github.com/iExec-Nox/nox-runner/issues/39)) ([9bed9e3](https://github.com/iExec-Nox/nox-runner/commit/9bed9e3d65d3d195cb2345e0ccd92257be11edd7))
* support multiple EVM chains ([#41](https://github.com/iExec-Nox/nox-runner/issues/41)) ([6d3c4f4](https://github.com/iExec-Nox/nox-runner/commit/6d3c4f41f1a5e672c0dc25d29f1b1d37b5fafd80))


### Bug Fixes

* configure max_messages_per_batch on NATS consumer and add validation ([#40](https://github.com/iExec-Nox/nox-runner/issues/40)) ([6b8841e](https://github.com/iExec-Nox/nox-runner/commit/6b8841e5765228a544c7c7b2c92d0abce7e0cde5))
* PlaintextToEncrypted event has been removed from NoxCompute ([#38](https://github.com/iExec-Nox/nox-runner/issues/38)) ([654f4c2](https://github.com/iExec-Nox/nox-runner/commit/654f4c21d45135d7877572ecaebf2015000a610c))

## [0.5.0](https://github.com/iExec-Nox/nox-runner/compare/v0.4.0...v0.5.0) (2026-03-28)


### Features

* adapt solidity type retrieval to new Handle specification ([#26](https://github.com/iExec-Nox/nox-runner/issues/26)) ([ce8590b](https://github.com/iExec-Nox/nox-runner/commit/ce8590b6d7168eec94283c7a64b5a28a0339c06b))
* add AUTHORIZATION header based on EIP-712 on Handle Gateway calls ([#21](https://github.com/iExec-Nox/nox-runner/issues/21)) ([4ee642e](https://github.com/iExec-Nox/nox-runner/commit/4ee642ef292dbfc50c9fc68f44daf9047b0691d7))
* add docker release gha ([#33](https://github.com/iExec-Nox/nox-runner/issues/33)) ([4a30ce1](https://github.com/iExec-Nox/nox-runner/commit/4a30ce19d52f1749b8b8392cb67ca67aa33a0882))
* add minimal TCP server for health checks ([#24](https://github.com/iExec-Nox/nox-runner/issues/24)) ([8658bea](https://github.com/iExec-Nox/nox-runner/commit/8658beaf948041d46683cf53d34c5c08c83989f8))
* add WrapPublicHandle support ([#23](https://github.com/iExec-Nox/nox-runner/issues/23)) ([888b14a](https://github.com/iExec-Nox/nox-runner/commit/888b14adc268a33f499a2354592c49b11acc60c7))
* connect to NATS JetStream with a consumer ([#8](https://github.com/iExec-Nox/nox-runner/issues/8)) ([d27ae01](https://github.com/iExec-Nox/nox-runner/commit/d27ae018f6663149c5dd7385236706b5e238c635))
* expose Prometheus metrics on /metrics ([#29](https://github.com/iExec-Nox/nox-runner/issues/29)) ([d7c6f9a](https://github.com/iExec-Nox/nox-runner/commit/d7c6f9a348b70f6f35b805f10487a4cd18ecb574))
* fetch KMS public key on-chain ([#15](https://github.com/iExec-Nox/nox-runner/issues/15)) ([9d60bb3](https://github.com/iExec-Nox/nox-runner/commit/9d60bb3350d6397a61bcc78438479b4c41eaa9a3))
* implement advanced functions related to confidential tokens ([#14](https://github.com/iExec-Nox/nox-runner/issues/14)) ([637c600](https://github.com/iExec-Nox/nox-runner/commit/637c600625dc50c32e3161e4356033d48ca296d1))
* implement arithmetic operations ([#7](https://github.com/iExec-Nox/nox-runner/issues/7)) ([cf7fbc9](https://github.com/iExec-Nox/nox-runner/commit/cf7fbc957501d5ef75d31a9cd1cb5577ed3ea14b))
* implement boolean operations ([#13](https://github.com/iExec-Nox/nox-runner/issues/13)) ([305a60a](https://github.com/iExec-Nox/nox-runner/commit/305a60a221af668f4730f01dc1f424773c1daf32))
* implement handles cache ([#18](https://github.com/iExec-Nox/nox-runner/issues/18)) ([22b848f](https://github.com/iExec-Nox/nox-runner/commit/22b848fa951c8ac493776643c2610a6c8d2d43f4))
* implement PlaintextToEncrypted operation ([#6](https://github.com/iExec-Nox/nox-runner/issues/6)) ([64f668f](https://github.com/iExec-Nox/nox-runner/commit/64f668f0fc1f592aea0b22bc4ab5aaaf729d29bf))
* implement safe arithmetic operations ([#12](https://github.com/iExec-Nox/nox-runner/issues/12)) ([4362bf4](https://github.com/iExec-Nox/nox-runner/commit/4362bf476bc72729782be413ea4b910dc16b31a9))
* initialize application ([#5](https://github.com/iExec-Nox/nox-runner/issues/5)) ([7136f6f](https://github.com/iExec-Nox/nox-runner/commit/7136f6f71b20c16d107b0fef52446b0b9a30ea99))
* initialize project ([#1](https://github.com/iExec-Nox/nox-runner/issues/1)) ([e330819](https://github.com/iExec-Nox/nox-runner/commit/e33081972d7fac1bbad6441c2cf0daa78e1c4c37))
* sign Handle Gateway responses with EIP-712 on /v0/compute endpoints ([#22](https://github.com/iExec-Nox/nox-runner/issues/22)) ([3fafc6a](https://github.com/iExec-Nox/nox-runner/commit/3fafc6a390788806ca8ef00b48df7f40fbe60e8c))
* use random 32 bytes session salt when interacting with Handle Gateway ([#30](https://github.com/iExec-Nox/nox-runner/issues/30)) ([cc56956](https://github.com/iExec-Nox/nox-runner/commit/cc569568f643a920142ee65ac729c80ad7061af8))


### Bug Fixes

* decode plaintext bytes32 to correct SolidityValue in PlaintextToEncrypted operation ([#19](https://github.com/iExec-Nox/nox-runner/issues/19)) ([2f5c577](https://github.com/iExec-Nox/nox-runner/commit/2f5c5770d0a8a1447e10712b41bd8b746d87b1d4))
* refactor code to use format_and_encrypt_result with proper hex serialization ([#10](https://github.com/iExec-Nox/nox-runner/issues/10)) ([b0be0bd](https://github.com/iExec-Nox/nox-runner/commit/b0be0bd1841c9eb1df690376966e96b22f8d88e4))
* return MAX integer value on division by zero ([#27](https://github.com/iExec-Nox/nox-runner/issues/27)) ([cf8bea7](https://github.com/iExec-Nox/nox-runner/commit/cf8bea7b5b726f2973cc3e40b521eced3d91b387))
