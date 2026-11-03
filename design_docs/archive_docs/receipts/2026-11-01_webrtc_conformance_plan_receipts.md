# WebRTC conformance receipts

Retained cap: 1 MB for this file. Native builds reuse the stable Genet target.

## 2026-11-01: before RTC bindings

Three selected unmodified WPT source files, each on both engines, using the
real runner. Five-second drive deadline and 25-second subprocess cap. This is
a diagnostic selection, not a complete WebRTC WPT score.

```json
{
  "schema": 1,
  "phase": "before-rtc",
  "source_revision": "e0a06bb5b231ab15a4384023c850a38cc0e7a6ac",
  "compiler": "rustc 1.99.1 (58cf8f9c3 2026-10-13)\nbinary: rustc\ncommit-hash: 58cf8f9c35b355199727627d07ae34a4e2846d67\ncommit-date: 2026-10-13\nhost: x86_64-pc-windows-msvc\nrelease: 1.99.1\nLLVM version: 23.1.1\n",
  "binary_sha256": "48c40b2d322ac145c0940b7ee1de4e252a28d14e145187fcca2af555e120c085",
  "recorded_at_utc": "2026-11-01T21:39:39.575882+00:00",
  "source_sha256": {
    "Cargo.lock": "6f2805829eaa3e3f3b3aa737520e910d30b11439a625cef9957a06a26f8c140d",
    "tests/wpt/tests/resources/testharness.js": "d2399236c2a09c429804ff2299ad6629e17e2b53f17a74dd341e936adb11ae3e",
    "tests/wpt/tests/webrtc/RTCPeerConnection-helper.js": "9709abd9c5a024c8bdbedd1cfab8a8f2c47b9358a987905cedc2a770ded69d2d",
    "tests/wpt/tests/webrtc/RTCPeerConnection-constructor.html": "31cf6a8f9f9d9c36d4a00a6ca1c224b493b6ee39b848dc4e380ce5b96988d074",
    "tests/wpt/tests/webrtc/RTCIceCandidate-constructor.html": "9bab537b6f3618bda3410d37bba9d63bbdec6116ebe9958759e67fb9fe7d5449",
    "tests/wpt/tests/webrtc/RTCPeerConnection-createDataChannel.html": "0356665f81d466804a4fd06d73075cc03767a5b4944e3f7d62e1da002c49c71f"
  },
  "outcomes": [
    {
      "detail": null,
      "failures": [
        "        [1] RTCPeerConnection.length RTCPeerConnection is not defined",
        "        [1] new RTCPeerConnection() RTCPeerConnection is not defined",
        "        [1] new RTCPeerConnection(null) RTCPeerConnection is not defined",
        "        [1] new RTCPeerConnection(undefined) RTCPeerConnection is not defined",
        "        [1] new RTCPeerConnection({}) RTCPeerConnection is not defined",
        "        [1] new RTCPeerConnection({ certificates: null }) assert_throws_js: function \"function() {\n        eval(expr);\n      }\" threw object \"ReferenceError: RTCPeerConnection is not defined\" (\"ReferenceError\") expected instance of function \"function TypeError() { [native code] }\" (\"TypeError\")",
        "        [1] new RTCPeerConnection({ certificates: undefined }) RTCPeerConnection is not defined",
        "        [1] new RTCPeerConnection({ certificates: [] }) RTCPeerConnection is not defined",
        "        [1] new RTCPeerConnection({ certificates: [null] }) assert_throws_js: function \"function() {\n        eval(expr);\n      }\" threw object \"ReferenceError: RTCPeerConnection is not defined\" (\"ReferenceError\") expected instance of function \"function TypeError() { [native code] }\" (\"TypeError\")",
        "        [1] new RTCPeerConnection({ certificates: [undefined] }) assert_throws_js: function \"function() {\n        eval(expr);\n      }\" threw object \"ReferenceError: RTCPeerConnection is not defined\" (\"ReferenceError\") expected instance of function \"function TypeError() { [native code] }\" (\"TypeError\")",
        "        [1] new RTCPeerConnection({ iceCandidatePoolSize: toNumberThrows }) assert_throws_js: function \"function() {\n        eval(expr);\n      }\" threw object \"ReferenceError: RTCPeerConnection is not defined\" (\"ReferenceError\") expected instance of function \"function TypeError() { [native code] }\" (\"TypeError\")",
        "        [1] localDescription initial value RTCPeerConnection is not defined",
        "        [1] currentLocalDescription initial value RTCPeerConnection is not defined",
        "        [1] pendingLocalDescription initial value RTCPeerConnection is not defined",
        "        [1] remoteDescription initial value RTCPeerConnection is not defined",
        "        [1] currentRemoteDescription initial value RTCPeerConnection is not defined",
        "        [1] pendingRemoteDescription initial value RTCPeerConnection is not defined",
        "        [1] signalingState initial value RTCPeerConnection is not defined",
        "        [1] iceGatheringState initial value RTCPeerConnection is not defined",
        "        [1] iceConnectionState initial value RTCPeerConnection is not defined",
        "        [1] connectionState initial value RTCPeerConnection is not defined",
        "        [1] canTrickleIceCandidates initial value RTCPeerConnection is not defined"
      ],
      "reason": null,
      "status": "fail",
      "subtests": [
        {
          "name": "RTCPeerConnection.length",
          "status": "fail"
        },
        {
          "name": "new RTCPeerConnection()",
          "status": "fail"
        },
        {
          "name": "new RTCPeerConnection(null)",
          "status": "fail"
        },
        {
          "name": "new RTCPeerConnection(undefined)",
          "status": "fail"
        },
        {
          "name": "new RTCPeerConnection({})",
          "status": "fail"
        },
        {
          "name": "new RTCPeerConnection({ certificates: null })",
          "status": "fail"
        },
        {
          "name": "new RTCPeerConnection({ certificates: undefined })",
          "status": "fail"
        },
        {
          "name": "new RTCPeerConnection({ certificates: [] })",
          "status": "fail"
        },
        {
          "name": "new RTCPeerConnection({ certificates: [null] })",
          "status": "fail"
        },
        {
          "name": "new RTCPeerConnection({ certificates: [undefined] })",
          "status": "fail"
        },
        {
          "name": "new RTCPeerConnection({ iceCandidatePoolSize: toNumberThrows })",
          "status": "fail"
        },
        {
          "name": "localDescription initial value",
          "status": "fail"
        },
        {
          "name": "currentLocalDescription initial value",
          "status": "fail"
        },
        {
          "name": "pendingLocalDescription initial value",
          "status": "fail"
        },
        {
          "name": "remoteDescription initial value",
          "status": "fail"
        },
        {
          "name": "currentRemoteDescription initial value",
          "status": "fail"
        },
        {
          "name": "pendingRemoteDescription initial value",
          "status": "fail"
        },
        {
          "name": "signalingState initial value",
          "status": "fail"
        },
        {
          "name": "iceGatheringState initial value",
          "status": "fail"
        },
        {
          "name": "iceConnectionState initial value",
          "status": "fail"
        },
        {
          "name": "connectionState initial value",
          "status": "fail"
        },
        {
          "name": "canTrickleIceCandidates initial value",
          "status": "fail"
        }
      ],
      "subtests_passed": 0,
      "subtests_total": 22,
      "engine": "boa",
      "test": "webrtc/RTCPeerConnection-constructor.html",
      "exit_code": 0,
      "command": [
        "C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe",
        "testharness-one",
        "webrtc/RTCPeerConnection-constructor.html",
        "--test-path",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCPeerConnection-constructor.html",
        "--tests-root",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests",
        "--engine",
        "boa",
        "--drive-deadline",
        "5"
      ]
    },
    {
      "detail": null,
      "failures": [
        "        [1] new RTCIceCandidate() assert_throws_js: function \"() => new RTCIceCandidate()\" threw object \"ReferenceError: RTCIceCandidate is not defined\" (\"ReferenceError\") expected instance of function \"function TypeError() { [native code] }\" (\"TypeError\")",
        "        [1] new RTCIceCandidate({}) assert_throws_js: function \"() => new RTCIceCandidate({})\" threw object \"ReferenceError: RTCIceCandidate is not defined\" (\"ReferenceError\") expected instance of function \"function TypeError() { [native code] }\" (\"TypeError\")",
        "        [1] new RTCIceCandidate({ ... }) with manually filled default values assert_throws_js: function \"() => new RTCIceCandidate({\n        candidate: '',\n        sdpMid: null,\n        sdpMLineIndex: null,\n        usernameFragment: undefined\n      })\" threw object \"ReferenceError: RTCIceCandidate is not defined\" (\"ReferenceError\") expected instance of function \"function TypeError() { [native code] }\" (\"TypeError\")",
        "        [1] new RTCIceCandidate({ sdpMid: null, sdpMLineIndex: null }) assert_throws_js: function \"() => new RTCIceCandidate({\n        sdpMid: null,\n        sdpMLineIndex: null\n      })\" threw object \"ReferenceError: RTCIceCandidate is not defined\" (\"ReferenceError\") expected instance of function \"function TypeError() { [native code] }\" (\"TypeError\")",
        "        [1] new RTCIceCandidate({ candidate: '' }) assert_throws_js: function \"() => new RTCIceCandidate({\n        candidate: ''\n      })\" threw object \"ReferenceError: RTCIceCandidate is not defined\" (\"ReferenceError\") expected instance of function \"function TypeError() { [native code] }\" (\"TypeError\")",
        "        [1] new RTCIceCandidate({ candidate: null }) assert_throws_js: function \"() => new RTCIceCandidate({\n        candidate: null\n      })\" threw object \"ReferenceError: RTCIceCandidate is not defined\" (\"ReferenceError\") expected instance of function \"function TypeError() { [native code] }\" (\"TypeError\")",
        "        [1] new RTCIceCandidate({ ... }) with valid candidate string only assert_throws_js: function \"() => new RTCIceCandidate({\n        candidate: candidateString\n      })\" threw object \"ReferenceError: RTCIceCandidate is not defined\" (\"ReferenceError\") expected instance of function \"function TypeError() { [native code] }\" (\"TypeError\")",
        "        [1] new RTCIceCandidate({ sdpMid: 'audio' }) RTCIceCandidate is not defined",
        "        [1] new RTCIceCandidate({ sdpMLineIndex: 0 }) RTCIceCandidate is not defined",
        "        [1] new RTCIceCandidate({ sdpMid: 'audio', sdpMLineIndex: 0 }) RTCIceCandidate is not defined",
        "        [1] new RTCIceCandidate({ candidate: '', sdpMid: 'audio' } RTCIceCandidate is not defined",
        "        [1] new RTCIceCandidate({ candidate: null, sdpMLineIndex: 0 } RTCIceCandidate is not defined",
        "        [1] new RTCIceCandidate({ ... }) with valid candidate string and sdpMid RTCIceCandidate is not defined",
        "        [1] new RTCIceCandidate({ ... }) with invalid candidate string and sdpMid RTCIceCandidate is not defined",
        "        [1] new RTCIceCandidate({ ... }) with nondefault values for most fields RTCIceCandidate is not defined",
        "        [1] new RTCIceCandidate({ ... }) with nondefault values for most fields, tcp candidate RTCIceCandidate is not defined",
        "        [1] new RTCIceCandidate({ ... }) with nondefault values for all fields RTCIceCandidate is not defined",
        "        [1] new RTCIceCandidate({ ... }) with nondefault values for all fields, tcp candidate RTCIceCandidate is not defined",
        "        [1] new RTCIceCandidate({ relayProtocol, url }) cloned vs signaled RTCIceCandidate is not defined",
        "        [1] new RTCIceCandidate({ ... }) with invalid sdpMid RTCIceCandidate is not defined",
        "        [1] new RTCIceCandidate({ ... }) with invalid sdpMLineIndex RTCIceCandidate is not defined"
      ],
      "reason": null,
      "status": "fail",
      "subtests": [
        {
          "name": "new RTCIceCandidate()",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({})",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with manually filled default values",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ sdpMid: null, sdpMLineIndex: null })",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ candidate: '' })",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ candidate: null })",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with valid candidate string only",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ sdpMid: 'audio' })",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ sdpMLineIndex: 0 })",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ sdpMid: 'audio', sdpMLineIndex: 0 })",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ candidate: '', sdpMid: 'audio' }",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ candidate: null, sdpMLineIndex: 0 }",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with valid candidate string and sdpMid",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with invalid candidate string and sdpMid",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for most fields",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for most fields, tcp candidate",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for all fields",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for all fields, tcp candidate",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ relayProtocol, url }) cloned vs signaled",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with invalid sdpMid",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with invalid sdpMLineIndex",
          "status": "fail"
        }
      ],
      "subtests_passed": 0,
      "subtests_total": 21,
      "engine": "boa",
      "test": "webrtc/RTCIceCandidate-constructor.html",
      "exit_code": 0,
      "command": [
        "C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe",
        "testharness-one",
        "webrtc/RTCIceCandidate-constructor.html",
        "--test-path",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCIceCandidate-constructor.html",
        "--tests-root",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests",
        "--engine",
        "boa",
        "--drive-deadline",
        "5"
      ]
    },
    {
      "detail": null,
      "failures": [
        "        [1] createDataChannel with no argument should throw TypeError RTCPeerConnection is not defined",
        "        [1] createDataChannel with closed connection should throw InvalidStateError RTCPeerConnection is not defined",
        "        [1] createDataChannel attribute default values RTCPeerConnection is not defined",
        "        [1] createDataChannel with provided parameters should initialize attributes to provided values RTCPeerConnection is not defined",
        "        [1] createDataChannel with label \"foo\" should succeed RTCPeerConnection is not defined",
        "        [1] createDataChannel with label null should succeed RTCPeerConnection is not defined",
        "        [1] createDataChannel with label undefined should succeed RTCPeerConnection is not defined",
        "        [1] createDataChannel with label lone surrogate should succeed RTCPeerConnection is not defined",
        "        [1] createDataChannel with ordered false should succeed RTCPeerConnection is not defined",
        "        [1] createDataChannel with ordered null/undefined should succeed RTCPeerConnection is not defined",
        "        [1] createDataChannel with maxPacketLifeTime 0 should succeed RTCPeerConnection is not defined",
        "        [1] createDataChannel with maxRetransmits 0 should succeed RTCPeerConnection is not defined",
        "        [1] createDataChannel with both maxPacketLifeTime and maxRetransmits undefined should succeed RTCPeerConnection is not defined",
        "        [1] createDataChannel with both maxPacketLifeTime and maxRetransmits should throw TypeError RTCPeerConnection is not defined",
        "        [1] createDataChannel with protocol \"foo\" should succeed RTCPeerConnection is not defined",
        "        [1] createDataChannel with protocol null should succeed RTCPeerConnection is not defined",
        "        [1] createDataChannel with protocol undefined should succeed RTCPeerConnection is not defined",
        "        [1] createDataChannel with protocol lone surrogate should succeed RTCPeerConnection is not defined",
        "        [1] createDataChannel with id 0 and negotiated not set should succeed, but not set the channel's id RTCPeerConnection is not defined",
        "        [1] createDataChannel with id 1 and negotiated not set should succeed, but not set the channel's id RTCPeerConnection is not defined",
        "        [1] createDataChannel with id 65534 and negotiated not set should succeed, but not set the channel's id RTCPeerConnection is not defined",
        "        [1] createDataChannel with id 65535 and negotiated not set should succeed, but not set the channel's id RTCPeerConnection is not defined",
        "        [1] createDataChannel with id 0 and negotiated true should succeed, and set the channel's id RTCPeerConnection is not defined",
        "        [1] createDataChannel with id 1 and negotiated true should succeed, and set the channel's id RTCPeerConnection is not defined",
        "        [1] createDataChannel with id 65534 and negotiated true should succeed, and set the channel's id RTCPeerConnection is not defined",
        "        [1] createDataChannel with id -1 and negotiated not set should throw TypeError RTCPeerConnection is not defined",
        "        [1] createDataChannel with id 65536 and negotiated not set should throw TypeError RTCPeerConnection is not defined",
        "        [1] createDataChannel with id -1 should throw TypeError RTCPeerConnection is not defined",
        "        [1] createDataChannel with id 65535 should throw TypeError RTCPeerConnection is not defined",
        "        [1] createDataChannel with id 65536 should throw TypeError RTCPeerConnection is not defined",
        "        [1] createDataChannel with too long label should throw TypeError RTCPeerConnection is not defined",
        "        [1] createDataChannel with too long label (2 byte unicode) should throw TypeError RTCPeerConnection is not defined",
        "        [1] createDataChannel with same label used twice should not throw RTCPeerConnection is not defined",
        "        [1] createDataChannel with negotiated true and id should succeed RTCPeerConnection is not defined",
        "        [1] createDataChannel with too long protocol should throw TypeError RTCPeerConnection is not defined",
        "        [1] createDataChannel with too long protocol (2 byte unicode) should throw TypeError RTCPeerConnection is not defined",
        "        [1] createDataChannel with maximum length label and protocol should succeed RTCPeerConnection is not defined",
        "        [1] createDataChannel with negotiated false should succeed RTCPeerConnection is not defined",
        "        [1] createDataChannel with negotiated false and id 42 should ignore the id RTCPeerConnection is not defined",
        "        [1] createDataChannel with negotiated true and id not defined should throw TypeError RTCPeerConnection is not defined",
        "        [1] Channels created (after SCTP connected) should have id assigned promise_test: Unhandled rejection with value: object \"ReferenceError: RTCPeerConnection is not defined\"",
        "        [1] Reusing a data channel id that is in use should throw OperationError RTCPeerConnection is not defined",
        "        [1] Reusing a data channel id that is in use (after setRemoteDescription) should throw OperationError promise_test: Unhandled rejection with value: object \"ReferenceError: RTCPeerConnection is not defined\"",
        "        [1] Reusing a data channel id that is in use (after setRemoteDescription, negotiated via DCEP) should throw OperationError promise_test: Unhandled rejection with value: object \"ReferenceError: RTCPeerConnection is not defined\"",
        "        [1] New datachannel should be in the connecting state after creation (after connection establishment) promise_test: Unhandled rejection with value: object \"ReferenceError: RTCPeerConnection is not defined\"",
        "        [1] addTrack, then creating datachannel, should negotiate properly promise_test: Unhandled rejection with value: object \"ReferenceError: RTCPeerConnection is not defined\"",
        "        [1] addTrack, then creating datachannel, should negotiate properly when max-bundle is used promise_test: Unhandled rejection with value: object \"ReferenceError: RTCPeerConnection is not defined\"",
        "        [1] Renegotiation with audio/video and datachannel should work properly promise_test: Unhandled rejection with value: object \"ReferenceError: RTCPeerConnection is not defined\"",
        "        [1] New negotiated datachannel should be in the connecting state after creation (after connection establishment) promise_test: Unhandled rejection with value: object \"ReferenceError: RTCPeerConnection is not defined\"",
        "        [1] addTrack, then creating negotiated datachannel, should negotiate properly promise_test: Unhandled rejection with value: object \"ReferenceError: RTCPeerConnection is not defined\"",
        "        [1] addTrack, then creating negotiated datachannel, should negotiate properly when max-bundle is used promise_test: Unhandled rejection with value: object \"ReferenceError: RTCPeerConnection is not defined\"",
        "        [1] Renegotiation with audio/video and negotiated datachannel should work properly promise_test: Unhandled rejection with value: object \"ReferenceError: RTCPeerConnection is not defined\""
      ],
      "reason": null,
      "status": "fail",
      "subtests": [
        {
          "name": "createDataChannel with no argument should throw TypeError",
          "status": "fail"
        },
        {
          "name": "createDataChannel with closed connection should throw InvalidStateError",
          "status": "fail"
        },
        {
          "name": "createDataChannel attribute default values",
          "status": "fail"
        },
        {
          "name": "createDataChannel with provided parameters should initialize attributes to provided values",
          "status": "fail"
        },
        {
          "name": "createDataChannel with label \"foo\" should succeed",
          "status": "fail"
        },
        {
          "name": "createDataChannel with label null should succeed",
          "status": "fail"
        },
        {
          "name": "createDataChannel with label undefined should succeed",
          "status": "fail"
        },
        {
          "name": "createDataChannel with label lone surrogate should succeed",
          "status": "fail"
        },
        {
          "name": "createDataChannel with ordered false should succeed",
          "status": "fail"
        },
        {
          "name": "createDataChannel with ordered null/undefined should succeed",
          "status": "fail"
        },
        {
          "name": "createDataChannel with maxPacketLifeTime 0 should succeed",
          "status": "fail"
        },
        {
          "name": "createDataChannel with maxRetransmits 0 should succeed",
          "status": "fail"
        },
        {
          "name": "createDataChannel with both maxPacketLifeTime and maxRetransmits undefined should succeed",
          "status": "fail"
        },
        {
          "name": "createDataChannel with both maxPacketLifeTime and maxRetransmits should throw TypeError",
          "status": "fail"
        },
        {
          "name": "createDataChannel with protocol \"foo\" should succeed",
          "status": "fail"
        },
        {
          "name": "createDataChannel with protocol null should succeed",
          "status": "fail"
        },
        {
          "name": "createDataChannel with protocol undefined should succeed",
          "status": "fail"
        },
        {
          "name": "createDataChannel with protocol lone surrogate should succeed",
          "status": "fail"
        },
        {
          "name": "createDataChannel with id 0 and negotiated not set should succeed, but not set the channel's id",
          "status": "fail"
        },
        {
          "name": "createDataChannel with id 1 and negotiated not set should succeed, but not set the channel's id",
          "status": "fail"
        },
        {
          "name": "createDataChannel with id 65534 and negotiated not set should succeed, but not set the channel's id",
          "status": "fail"
        },
        {
          "name": "createDataChannel with id 65535 and negotiated not set should succeed, but not set the channel's id",
          "status": "fail"
        },
        {
          "name": "createDataChannel with id 0 and negotiated true should succeed, and set the channel's id",
          "status": "fail"
        },
        {
          "name": "createDataChannel with id 1 and negotiated true should succeed, and set the channel's id",
          "status": "fail"
        },
        {
          "name": "createDataChannel with id 65534 and negotiated true should succeed, and set the channel's id",
          "status": "fail"
        },
        {
          "name": "createDataChannel with id -1 and negotiated not set should throw TypeError",
          "status": "fail"
        },
        {
          "name": "createDataChannel with id 65536 and negotiated not set should throw TypeError",
          "status": "fail"
        },
        {
          "name": "createDataChannel with id -1 should throw TypeError",
          "status": "fail"
        },
        {
          "name": "createDataChannel with id 65535 should throw TypeError",
          "status": "fail"
        },
        {
          "name": "createDataChannel with id 65536 should throw TypeError",
          "status": "fail"
        },
        {
          "name": "createDataChannel with too long label should throw TypeError",
          "status": "fail"
        },
        {
          "name": "createDataChannel with too long label (2 byte unicode) should throw TypeError",
          "status": "fail"
        },
        {
          "name": "createDataChannel with same label used twice should not throw",
          "status": "fail"
        },
        {
          "name": "createDataChannel with negotiated true and id should succeed",
          "status": "fail"
        },
        {
          "name": "createDataChannel with too long protocol should throw TypeError",
          "status": "fail"
        },
        {
          "name": "createDataChannel with too long protocol (2 byte unicode) should throw TypeError",
          "status": "fail"
        },
        {
          "name": "createDataChannel with maximum length label and protocol should succeed",
          "status": "fail"
        },
        {
          "name": "createDataChannel with negotiated false should succeed",
          "status": "fail"
        },
        {
          "name": "createDataChannel with negotiated false and id 42 should ignore the id",
          "status": "fail"
        },
        {
          "name": "createDataChannel with negotiated true and id not defined should throw TypeError",
          "status": "fail"
        },
        {
          "name": "Channels created (after SCTP connected) should have id assigned",
          "status": "fail"
        },
        {
          "name": "Reusing a data channel id that is in use should throw OperationError",
          "status": "fail"
        },
        {
          "name": "Reusing a data channel id that is in use (after setRemoteDescription) should throw OperationError",
          "status": "fail"
        },
        {
          "name": "Reusing a data channel id that is in use (after setRemoteDescription, negotiated via DCEP) should throw OperationError",
          "status": "fail"
        },
        {
          "name": "New datachannel should be in the connecting state after creation (after connection establishment)",
          "status": "fail"
        },
        {
          "name": "addTrack, then creating datachannel, should negotiate properly",
          "status": "fail"
        },
        {
          "name": "addTrack, then creating datachannel, should negotiate properly when max-bundle is used",
          "status": "fail"
        },
        {
          "name": "Renegotiation with audio/video and datachannel should work properly",
          "status": "fail"
        },
        {
          "name": "New negotiated datachannel should be in the connecting state after creation (after connection establishment)",
          "status": "fail"
        },
        {
          "name": "addTrack, then creating negotiated datachannel, should negotiate properly",
          "status": "fail"
        },
        {
          "name": "addTrack, then creating negotiated datachannel, should negotiate properly when max-bundle is used",
          "status": "fail"
        },
        {
          "name": "Renegotiation with audio/video and negotiated datachannel should work properly",
          "status": "fail"
        }
      ],
      "subtests_passed": 0,
      "subtests_total": 52,
      "engine": "boa",
      "test": "webrtc/RTCPeerConnection-createDataChannel.html",
      "exit_code": 0,
      "command": [
        "C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe",
        "testharness-one",
        "webrtc/RTCPeerConnection-createDataChannel.html",
        "--test-path",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCPeerConnection-createDataChannel.html",
        "--tests-root",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests",
        "--engine",
        "boa",
        "--drive-deadline",
        "5"
      ]
    },
    {
      "detail": null,
      "failures": [
        "        [1] RTCPeerConnection.length Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] new RTCPeerConnection() Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] new RTCPeerConnection(null) Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] new RTCPeerConnection(undefined) Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] new RTCPeerConnection({}) Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] new RTCPeerConnection({ certificates: null }) assert_throws_js: function \"function() {\n        eval(expr);\n      }\" threw object \"ReferenceError: Cannot access undeclared variable 'RTCPeerConnection'.\" (\"ReferenceError\") expected instance of function \"function TypeError() { [ native code ] }\" (\"TypeError\")",
        "        [1] new RTCPeerConnection({ certificates: undefined }) Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] new RTCPeerConnection({ certificates: [] }) Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] new RTCPeerConnection({ certificates: [null] }) assert_throws_js: function \"function() {\n        eval(expr);\n      }\" threw object \"ReferenceError: Cannot access undeclared variable 'RTCPeerConnection'.\" (\"ReferenceError\") expected instance of function \"function TypeError() { [ native code ] }\" (\"TypeError\")",
        "        [1] new RTCPeerConnection({ certificates: [undefined] }) assert_throws_js: function \"function() {\n        eval(expr);\n      }\" threw object \"ReferenceError: Cannot access undeclared variable 'RTCPeerConnection'.\" (\"ReferenceError\") expected instance of function \"function TypeError() { [ native code ] }\" (\"TypeError\")",
        "        [1] new RTCPeerConnection({ iceCandidatePoolSize: toNumberThrows }) assert_throws_js: function \"function() {\n        eval(expr);\n      }\" threw object \"ReferenceError: Cannot access undeclared variable 'RTCPeerConnection'.\" (\"ReferenceError\") expected instance of function \"function TypeError() { [ native code ] }\" (\"TypeError\")",
        "        [1] localDescription initial value Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] currentLocalDescription initial value Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] pendingLocalDescription initial value Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] remoteDescription initial value Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] currentRemoteDescription initial value Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] pendingRemoteDescription initial value Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] signalingState initial value Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] iceGatheringState initial value Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] iceConnectionState initial value Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] connectionState initial value Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] canTrickleIceCandidates initial value Cannot access undeclared variable 'RTCPeerConnection'."
      ],
      "reason": null,
      "status": "fail",
      "subtests": [
        {
          "name": "RTCPeerConnection.length",
          "status": "fail"
        },
        {
          "name": "new RTCPeerConnection()",
          "status": "fail"
        },
        {
          "name": "new RTCPeerConnection(null)",
          "status": "fail"
        },
        {
          "name": "new RTCPeerConnection(undefined)",
          "status": "fail"
        },
        {
          "name": "new RTCPeerConnection({})",
          "status": "fail"
        },
        {
          "name": "new RTCPeerConnection({ certificates: null })",
          "status": "fail"
        },
        {
          "name": "new RTCPeerConnection({ certificates: undefined })",
          "status": "fail"
        },
        {
          "name": "new RTCPeerConnection({ certificates: [] })",
          "status": "fail"
        },
        {
          "name": "new RTCPeerConnection({ certificates: [null] })",
          "status": "fail"
        },
        {
          "name": "new RTCPeerConnection({ certificates: [undefined] })",
          "status": "fail"
        },
        {
          "name": "new RTCPeerConnection({ iceCandidatePoolSize: toNumberThrows })",
          "status": "fail"
        },
        {
          "name": "localDescription initial value",
          "status": "fail"
        },
        {
          "name": "currentLocalDescription initial value",
          "status": "fail"
        },
        {
          "name": "pendingLocalDescription initial value",
          "status": "fail"
        },
        {
          "name": "remoteDescription initial value",
          "status": "fail"
        },
        {
          "name": "currentRemoteDescription initial value",
          "status": "fail"
        },
        {
          "name": "pendingRemoteDescription initial value",
          "status": "fail"
        },
        {
          "name": "signalingState initial value",
          "status": "fail"
        },
        {
          "name": "iceGatheringState initial value",
          "status": "fail"
        },
        {
          "name": "iceConnectionState initial value",
          "status": "fail"
        },
        {
          "name": "connectionState initial value",
          "status": "fail"
        },
        {
          "name": "canTrickleIceCandidates initial value",
          "status": "fail"
        }
      ],
      "subtests_passed": 0,
      "subtests_total": 22,
      "engine": "nova",
      "test": "webrtc/RTCPeerConnection-constructor.html",
      "exit_code": 0,
      "command": [
        "C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe",
        "testharness-one",
        "webrtc/RTCPeerConnection-constructor.html",
        "--test-path",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCPeerConnection-constructor.html",
        "--tests-root",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests",
        "--engine",
        "nova",
        "--drive-deadline",
        "5"
      ]
    },
    {
      "detail": null,
      "failures": [
        "        [1] new RTCIceCandidate() assert_throws_js: function \"() => new RTCIceCandidate()\" threw object \"ReferenceError: Cannot access undeclared variable 'RTCIceCandidate'.\" (\"ReferenceError\") expected instance of function \"function TypeError() { [ native code ] }\" (\"TypeError\")",
        "        [1] new RTCIceCandidate({}) assert_throws_js: function \"() => new RTCIceCandidate({})\" threw object \"ReferenceError: Cannot access undeclared variable 'RTCIceCandidate'.\" (\"ReferenceError\") expected instance of function \"function TypeError() { [ native code ] }\" (\"TypeError\")",
        "        [1] new RTCIceCandidate({ ... }) with manually filled default values assert_throws_js: function \"() => new RTCIceCandidate({\n        candidate: '',\n        sdpMid: null,\n        sdpMLineIndex: null,\n        usernameFragment: undefined\n      })\" threw object \"ReferenceError: Cannot access undeclared variable 'RTCIceCandidate'.\" (\"ReferenceError\") expected instance of function \"function TypeError() { [ native code ] }\" (\"TypeError\")",
        "        [1] new RTCIceCandidate({ sdpMid: null, sdpMLineIndex: null }) assert_throws_js: function \"() => new RTCIceCandidate({\n        sdpMid: null,\n        sdpMLineIndex: null\n      })\" threw object \"ReferenceError: Cannot access undeclared variable 'RTCIceCandidate'.\" (\"ReferenceError\") expected instance of function \"function TypeError() { [ native code ] }\" (\"TypeError\")",
        "        [1] new RTCIceCandidate({ candidate: '' }) assert_throws_js: function \"() => new RTCIceCandidate({\n        candidate: ''\n      })\" threw object \"ReferenceError: Cannot access undeclared variable 'RTCIceCandidate'.\" (\"ReferenceError\") expected instance of function \"function TypeError() { [ native code ] }\" (\"TypeError\")",
        "        [1] new RTCIceCandidate({ candidate: null }) assert_throws_js: function \"() => new RTCIceCandidate({\n        candidate: null\n      })\" threw object \"ReferenceError: Cannot access undeclared variable 'RTCIceCandidate'.\" (\"ReferenceError\") expected instance of function \"function TypeError() { [ native code ] }\" (\"TypeError\")",
        "        [1] new RTCIceCandidate({ ... }) with valid candidate string only assert_throws_js: function \"() => new RTCIceCandidate({\n        candidate: candidateString\n      })\" threw object \"ReferenceError: Cannot access undeclared variable 'RTCIceCandidate'.\" (\"ReferenceError\") expected instance of function \"function TypeError() { [ native code ] }\" (\"TypeError\")",
        "        [1] new RTCIceCandidate({ sdpMid: 'audio' }) Cannot access undeclared variable 'RTCIceCandidate'.",
        "        [1] new RTCIceCandidate({ sdpMLineIndex: 0 }) Cannot access undeclared variable 'RTCIceCandidate'.",
        "        [1] new RTCIceCandidate({ sdpMid: 'audio', sdpMLineIndex: 0 }) Cannot access undeclared variable 'RTCIceCandidate'.",
        "        [1] new RTCIceCandidate({ candidate: '', sdpMid: 'audio' } Cannot access undeclared variable 'RTCIceCandidate'.",
        "        [1] new RTCIceCandidate({ candidate: null, sdpMLineIndex: 0 } Cannot access undeclared variable 'RTCIceCandidate'.",
        "        [1] new RTCIceCandidate({ ... }) with valid candidate string and sdpMid Cannot access undeclared variable 'RTCIceCandidate'.",
        "        [1] new RTCIceCandidate({ ... }) with invalid candidate string and sdpMid Cannot access undeclared variable 'RTCIceCandidate'.",
        "        [1] new RTCIceCandidate({ ... }) with nondefault values for most fields Cannot access undeclared variable 'RTCIceCandidate'.",
        "        [1] new RTCIceCandidate({ ... }) with nondefault values for most fields, tcp candidate Cannot access undeclared variable 'RTCIceCandidate'.",
        "        [1] new RTCIceCandidate({ ... }) with nondefault values for all fields Cannot access undeclared variable 'RTCIceCandidate'.",
        "        [1] new RTCIceCandidate({ ... }) with nondefault values for all fields, tcp candidate Cannot access undeclared variable 'RTCIceCandidate'.",
        "        [1] new RTCIceCandidate({ relayProtocol, url }) cloned vs signaled Cannot access undeclared variable 'RTCIceCandidate'.",
        "        [1] new RTCIceCandidate({ ... }) with invalid sdpMid Cannot access undeclared variable 'RTCIceCandidate'.",
        "        [1] new RTCIceCandidate({ ... }) with invalid sdpMLineIndex Cannot access undeclared variable 'RTCIceCandidate'."
      ],
      "reason": null,
      "status": "fail",
      "subtests": [
        {
          "name": "new RTCIceCandidate()",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({})",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with manually filled default values",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ sdpMid: null, sdpMLineIndex: null })",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ candidate: '' })",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ candidate: null })",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with valid candidate string only",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ sdpMid: 'audio' })",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ sdpMLineIndex: 0 })",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ sdpMid: 'audio', sdpMLineIndex: 0 })",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ candidate: '', sdpMid: 'audio' }",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ candidate: null, sdpMLineIndex: 0 }",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with valid candidate string and sdpMid",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with invalid candidate string and sdpMid",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for most fields",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for most fields, tcp candidate",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for all fields",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for all fields, tcp candidate",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ relayProtocol, url }) cloned vs signaled",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with invalid sdpMid",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with invalid sdpMLineIndex",
          "status": "fail"
        }
      ],
      "subtests_passed": 0,
      "subtests_total": 21,
      "engine": "nova",
      "test": "webrtc/RTCIceCandidate-constructor.html",
      "exit_code": 0,
      "command": [
        "C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe",
        "testharness-one",
        "webrtc/RTCIceCandidate-constructor.html",
        "--test-path",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCIceCandidate-constructor.html",
        "--tests-root",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests",
        "--engine",
        "nova",
        "--drive-deadline",
        "5"
      ]
    },
    {
      "detail": null,
      "failures": [
        "        [1] createDataChannel with no argument should throw TypeError Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] createDataChannel with closed connection should throw InvalidStateError Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] createDataChannel attribute default values Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] createDataChannel with provided parameters should initialize attributes to provided values Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] createDataChannel with label \"foo\" should succeed Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] createDataChannel with label null should succeed Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] createDataChannel with label undefined should succeed Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] createDataChannel with label lone surrogate should succeed Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] createDataChannel with ordered false should succeed Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] createDataChannel with ordered null/undefined should succeed Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] createDataChannel with maxPacketLifeTime 0 should succeed Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] createDataChannel with maxRetransmits 0 should succeed Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] createDataChannel with both maxPacketLifeTime and maxRetransmits undefined should succeed Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] createDataChannel with both maxPacketLifeTime and maxRetransmits should throw TypeError Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] createDataChannel with protocol \"foo\" should succeed Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] createDataChannel with protocol null should succeed Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] createDataChannel with protocol undefined should succeed Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] createDataChannel with protocol lone surrogate should succeed Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] createDataChannel with id 0 and negotiated not set should succeed, but not set the channel's id Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] createDataChannel with id 1 and negotiated not set should succeed, but not set the channel's id Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] createDataChannel with id 65534 and negotiated not set should succeed, but not set the channel's id Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] createDataChannel with id 65535 and negotiated not set should succeed, but not set the channel's id Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] createDataChannel with id 0 and negotiated true should succeed, and set the channel's id Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] createDataChannel with id 1 and negotiated true should succeed, and set the channel's id Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] createDataChannel with id 65534 and negotiated true should succeed, and set the channel's id Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] createDataChannel with id -1 and negotiated not set should throw TypeError Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] createDataChannel with id 65536 and negotiated not set should throw TypeError Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] createDataChannel with id -1 should throw TypeError Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] createDataChannel with id 65535 should throw TypeError Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] createDataChannel with id 65536 should throw TypeError Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] createDataChannel with too long label should throw TypeError Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] createDataChannel with too long label (2 byte unicode) should throw TypeError Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] createDataChannel with same label used twice should not throw Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] createDataChannel with negotiated true and id should succeed Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] createDataChannel with too long protocol should throw TypeError Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] createDataChannel with too long protocol (2 byte unicode) should throw TypeError Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] createDataChannel with maximum length label and protocol should succeed Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] createDataChannel with negotiated false should succeed Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] createDataChannel with negotiated false and id 42 should ignore the id Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] createDataChannel with negotiated true and id not defined should throw TypeError Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] Channels created (after SCTP connected) should have id assigned promise_test: Unhandled rejection with value: object \"ReferenceError: Cannot access undeclared variable 'RTCPeerConnection'.\"",
        "        [1] Reusing a data channel id that is in use should throw OperationError Cannot access undeclared variable 'RTCPeerConnection'.",
        "        [1] Reusing a data channel id that is in use (after setRemoteDescription) should throw OperationError promise_test: Unhandled rejection with value: object \"ReferenceError: Cannot access undeclared variable 'RTCPeerConnection'.\"",
        "        [1] Reusing a data channel id that is in use (after setRemoteDescription, negotiated via DCEP) should throw OperationError promise_test: Unhandled rejection with value: object \"ReferenceError: Cannot access undeclared variable 'RTCPeerConnection'.\"",
        "        [1] New datachannel should be in the connecting state after creation (after connection establishment) promise_test: Unhandled rejection with value: object \"ReferenceError: Cannot access undeclared variable 'RTCPeerConnection'.\"",
        "        [1] addTrack, then creating datachannel, should negotiate properly promise_test: Unhandled rejection with value: object \"ReferenceError: Cannot access undeclared variable 'RTCPeerConnection'.\"",
        "        [1] addTrack, then creating datachannel, should negotiate properly when max-bundle is used promise_test: Unhandled rejection with value: object \"ReferenceError: Cannot access undeclared variable 'RTCPeerConnection'.\"",
        "        [1] Renegotiation with audio/video and datachannel should work properly promise_test: Unhandled rejection with value: object \"ReferenceError: Cannot access undeclared variable 'RTCPeerConnection'.\"",
        "        [1] New negotiated datachannel should be in the connecting state after creation (after connection establishment) promise_test: Unhandled rejection with value: object \"ReferenceError: Cannot access undeclared variable 'RTCPeerConnection'.\"",
        "        [1] addTrack, then creating negotiated datachannel, should negotiate properly promise_test: Unhandled rejection with value: object \"ReferenceError: Cannot access undeclared variable 'RTCPeerConnection'.\"",
        "        [1] addTrack, then creating negotiated datachannel, should negotiate properly when max-bundle is used promise_test: Unhandled rejection with value: object \"ReferenceError: Cannot access undeclared variable 'RTCPeerConnection'.\"",
        "        [1] Renegotiation with audio/video and negotiated datachannel should work properly promise_test: Unhandled rejection with value: object \"ReferenceError: Cannot access undeclared variable 'RTCPeerConnection'.\""
      ],
      "reason": null,
      "status": "fail",
      "subtests": [
        {
          "name": "createDataChannel with no argument should throw TypeError",
          "status": "fail"
        },
        {
          "name": "createDataChannel with closed connection should throw InvalidStateError",
          "status": "fail"
        },
        {
          "name": "createDataChannel attribute default values",
          "status": "fail"
        },
        {
          "name": "createDataChannel with provided parameters should initialize attributes to provided values",
          "status": "fail"
        },
        {
          "name": "createDataChannel with label \"foo\" should succeed",
          "status": "fail"
        },
        {
          "name": "createDataChannel with label null should succeed",
          "status": "fail"
        },
        {
          "name": "createDataChannel with label undefined should succeed",
          "status": "fail"
        },
        {
          "name": "createDataChannel with label lone surrogate should succeed",
          "status": "fail"
        },
        {
          "name": "createDataChannel with ordered false should succeed",
          "status": "fail"
        },
        {
          "name": "createDataChannel with ordered null/undefined should succeed",
          "status": "fail"
        },
        {
          "name": "createDataChannel with maxPacketLifeTime 0 should succeed",
          "status": "fail"
        },
        {
          "name": "createDataChannel with maxRetransmits 0 should succeed",
          "status": "fail"
        },
        {
          "name": "createDataChannel with both maxPacketLifeTime and maxRetransmits undefined should succeed",
          "status": "fail"
        },
        {
          "name": "createDataChannel with both maxPacketLifeTime and maxRetransmits should throw TypeError",
          "status": "fail"
        },
        {
          "name": "createDataChannel with protocol \"foo\" should succeed",
          "status": "fail"
        },
        {
          "name": "createDataChannel with protocol null should succeed",
          "status": "fail"
        },
        {
          "name": "createDataChannel with protocol undefined should succeed",
          "status": "fail"
        },
        {
          "name": "createDataChannel with protocol lone surrogate should succeed",
          "status": "fail"
        },
        {
          "name": "createDataChannel with id 0 and negotiated not set should succeed, but not set the channel's id",
          "status": "fail"
        },
        {
          "name": "createDataChannel with id 1 and negotiated not set should succeed, but not set the channel's id",
          "status": "fail"
        },
        {
          "name": "createDataChannel with id 65534 and negotiated not set should succeed, but not set the channel's id",
          "status": "fail"
        },
        {
          "name": "createDataChannel with id 65535 and negotiated not set should succeed, but not set the channel's id",
          "status": "fail"
        },
        {
          "name": "createDataChannel with id 0 and negotiated true should succeed, and set the channel's id",
          "status": "fail"
        },
        {
          "name": "createDataChannel with id 1 and negotiated true should succeed, and set the channel's id",
          "status": "fail"
        },
        {
          "name": "createDataChannel with id 65534 and negotiated true should succeed, and set the channel's id",
          "status": "fail"
        },
        {
          "name": "createDataChannel with id -1 and negotiated not set should throw TypeError",
          "status": "fail"
        },
        {
          "name": "createDataChannel with id 65536 and negotiated not set should throw TypeError",
          "status": "fail"
        },
        {
          "name": "createDataChannel with id -1 should throw TypeError",
          "status": "fail"
        },
        {
          "name": "createDataChannel with id 65535 should throw TypeError",
          "status": "fail"
        },
        {
          "name": "createDataChannel with id 65536 should throw TypeError",
          "status": "fail"
        },
        {
          "name": "createDataChannel with too long label should throw TypeError",
          "status": "fail"
        },
        {
          "name": "createDataChannel with too long label (2 byte unicode) should throw TypeError",
          "status": "fail"
        },
        {
          "name": "createDataChannel with same label used twice should not throw",
          "status": "fail"
        },
        {
          "name": "createDataChannel with negotiated true and id should succeed",
          "status": "fail"
        },
        {
          "name": "createDataChannel with too long protocol should throw TypeError",
          "status": "fail"
        },
        {
          "name": "createDataChannel with too long protocol (2 byte unicode) should throw TypeError",
          "status": "fail"
        },
        {
          "name": "createDataChannel with maximum length label and protocol should succeed",
          "status": "fail"
        },
        {
          "name": "createDataChannel with negotiated false should succeed",
          "status": "fail"
        },
        {
          "name": "createDataChannel with negotiated false and id 42 should ignore the id",
          "status": "fail"
        },
        {
          "name": "createDataChannel with negotiated true and id not defined should throw TypeError",
          "status": "fail"
        },
        {
          "name": "Channels created (after SCTP connected) should have id assigned",
          "status": "fail"
        },
        {
          "name": "Reusing a data channel id that is in use should throw OperationError",
          "status": "fail"
        },
        {
          "name": "Reusing a data channel id that is in use (after setRemoteDescription) should throw OperationError",
          "status": "fail"
        },
        {
          "name": "Reusing a data channel id that is in use (after setRemoteDescription, negotiated via DCEP) should throw OperationError",
          "status": "fail"
        },
        {
          "name": "New datachannel should be in the connecting state after creation (after connection establishment)",
          "status": "fail"
        },
        {
          "name": "addTrack, then creating datachannel, should negotiate properly",
          "status": "fail"
        },
        {
          "name": "addTrack, then creating datachannel, should negotiate properly when max-bundle is used",
          "status": "fail"
        },
        {
          "name": "Renegotiation with audio/video and datachannel should work properly",
          "status": "fail"
        },
        {
          "name": "New negotiated datachannel should be in the connecting state after creation (after connection establishment)",
          "status": "fail"
        },
        {
          "name": "addTrack, then creating negotiated datachannel, should negotiate properly",
          "status": "fail"
        },
        {
          "name": "addTrack, then creating negotiated datachannel, should negotiate properly when max-bundle is used",
          "status": "fail"
        },
        {
          "name": "Renegotiation with audio/video and negotiated datachannel should work properly",
          "status": "fail"
        }
      ],
      "subtests_passed": 0,
      "subtests_total": 52,
      "engine": "nova",
      "test": "webrtc/RTCPeerConnection-createDataChannel.html",
      "exit_code": 0,
      "command": [
        "C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe",
        "testharness-one",
        "webrtc/RTCPeerConnection-createDataChannel.html",
        "--test-path",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCPeerConnection-createDataChannel.html",
        "--tests-root",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests",
        "--engine",
        "nova",
        "--drive-deadline",
        "5"
      ]
    }
  ]
}
```

## 2026-11-01: initial native integration diagnostic

```json
{
  "stage": "first-native-js-run",
  "command": "cargo test -p genet-webrtc --test browser --offline -j 1",
  "exit_code": 1,
  "passed": [
    "boa::constructors",
    "boa::rollback",
    "boa::transfer",
    "boa::policy_and_budgets"
  ],
  "failed": {
    "vano::*": "runtime init: TypeError: Cannot delete property",
    "boa::document_teardown": "fixture used document.body before parsing a document"
  },
  "source_sha256": {
    "components\\script-runtime-api\\webrtc\\bootstrap.js": "234e722f7bf20cdb9dd62e9616caa40932e9e9a1806807504da08447f3a8edf2",
    "components\\genet-webrtc\\tests\\browser.rs": "46c5770cbae52f84b72078b6464375a34e1adadcf03de9aa3f888e1b2d010a94"
  }
}
```

## 2026-11-01: earlier evaluation and provider run notes

Moved from the plan under the merged documentation policy; original supplier
receipts stay in their cited locations.


- 2026-11-01: Mark accepted continuing with RTC after E0. E1a implements the
  independent provider and an explicitly driven native host route; candidate
  discovery/relay expansion and the receiving consumer migrations remain E1b.

- 2026-11-01: E0 complete. Working Rust 1.99.1 on Windows passes all six
  independent native probes: current/pending SDP, local rollback and a fresh
  offer, invalid options and closed-peer refusal, str0m's own negotiation,
  and cross-engine data-channel SDP offer/answer in each direction.
  `murm/testing/webrtc-evaluation/receipts/native-tests.json` records exact
  source/lock/binary hashes and the passing cases. The dependency report freezes
  independent graphs; `existing-carrier-crypto.txt` verifies the current native
  carrier's transitive AWS-LC edge. Native tests used both cores in one process
  and exchanged no network packets. No MSRV execution, WPT pass count, headed
  interop, TURN, media, provider migration or runtime dependency change is
  claimed. E1-E3 retain the implementation and acceptance gates above.

- 2026-11-01: E1a real packet tests found a release defect that the E0 SDP
  probes could not reveal: rtc 0.21.1 discards accepted DCEP reliability
  settings and leaves the accepting SCTP stream ordered/reliable. The exact
  upstream fix is `70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38` (#280), now
  pinned by `murm-rtc`. The published-release failed run is retained. The UDP
  adapter observes remote DTLS closure separately from a local peer-connection
  `Closed` state, so it does not fabricate a web-visible connection-state
  transition. All four native tests pass on Rust 1.99.1 and MSRV 1.97.1;
  core-only compilation, strict Clippy and formatting pass. Native coverage
  includes text, fragmented binary, ordered and unordered/zero-retransmit
  channels, reverse traffic during a paused data read, retained message
  delivery, certificate fingerprints, close, cancellation/socket release and
  route/datagram-budget refusal. `murm/testing/webrtc-evaluation/receipts/provider-qualification.json`
  and `provider-boundaries.json` freeze the source/binary/compiler and core
  dependency evidence. The final workspace default-selection audit preserves
  ordinary HTTP/WebSocket builds. E1b/E2 retain actual Genet, application,
  discovery/relay, browser and WPT qualification. No saturation, packet-loss
  policy or media claim follows from the native loopback test.

- 2026-11-01: E1b/E2 receiving work uses a per-realm `WebRtcFactory` contract in
  `genet-runtime` and a separate native `genet-webrtc` component. Both std socket
  polling and Tokio waits now use Murm's same packet/timer driver. The immutable
  supplier is `murm` commit `c58b70421d31ed9f2288dc44152dcb6d719552a9`, published
  through main after preserving and merging concurrent remote work. Protocol
  source remains the exact upstream DCEP fix. Native provider qualification is
  appended to Murm's family-move receipt file; earlier cited evidence remains
  in its original location.

- Receiving boundaries under qualification: Window-only objects; private
  retained event hooks; caller-owned peer resources; child factory inheritance;
  native resource revocation on discarded realms; active-peer snapshot refusal;
  and document/WPT scheduling that waits for actual network progress. Ortet's
  scripted route installs the real host, with configurable `--rtc-bind` and
  loopback as its initial concrete address. STUN/TURN discovery, route-policy
  changes after construction, custom certificates, full media, headed interop
  and broad WPT conformance remain open. Relay-only construction registers no
  socket or direct candidate in this host-only adapter.


## 2026-11-01: merged-source native integration

```json
{
  "recorded_at_utc": "2026-11-01T22:14:25.051435+00:00",
  "source_revision": "12c3ac8e2f4a45014cd3a81404f3bc0ff05fe480",
  "command": [
    "cargo",
    "test",
    "-p",
    "genet-webrtc",
    "--test",
    "browser",
    "--offline",
    "-j",
    "1"
  ],
  "exit_code": 101,
  "summary": [
    "runtime: \"evaluation threw: TypeError: Could not set property '__rtc_new' of [object Object].\"",
    "",
    "---- boa::navigation_revokes_old_handles stdout ----",
    "",
    "thread 'boa::navigation_revokes_old_handles' (42592) panicked at components\\genet-webrtc\\tests\\browser.rs:229:5:",
    "assertion `left == right` failed",
    "  left: \"InvalidStateError,TypeError\"",
    " right: \"InvalidStateError,InvalidStateError\"",
    "",
    "",
    "failures:",
    "    boa::navigation_revokes_old_handles",
    "    vano::constructors",
    "    vano::document_teardown",
    "    vano::navigation_revokes_old_handles",
    "    vano::policy_and_budgets",
    "    vano::rollback",
    "    vano::transfer",
    "",
    "test result: FAILED. 5 passed; 7 failed; 0 ignored; 0 measured; 0 filtered out; finished in 13.95s",
    "",
    "",
    "    Blocking waiting for file lock on package cache",
    "    Blocking waiting for file lock on package cache",
    "     Locking 4 packages to highest compatible versions",
    "      Adding arboard v3.7.0",
    "    Updating genet-taffy v0.14.0 (C:\\Users\\mark_\\Code\\repos\\genet\\support\\patches\\taffy) -> v0.16.0",
    "      Adding ipc-channel v0.23.1",
    " Downgrading uuid v1.27.0 -> v1.26.1 (available: v1.29.0)",
    "    Blocking waiting for file lock on package cache",
    "   Compiling livery v0.0.5 (C:\\Users\\mark_\\Code\\repos\\genet\\components\\livery)",
    "   Compiling uuid v1.26.1",
    "   Compiling rtc-ice v0.21.1 (https://github.com/webrtc-rs/rtc?rev=70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38#70901c3d)",
    "warning: use of deprecated method `std::sync::atomic::Atomic::<u64>::fetch_update`: renamed to `try_update` for consistency",
    "  --> components\\genet-scripted-dom\\forms_submission.rs:98:14",
    "   |",
    "98 |             .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |next| {",
    "   |              ^^^^^^^^^^^^",
    "   |",
    "   = note: `#[warn(deprecated)]` on by default",
    "help: replace the use of the deprecated method",
    "   |",
    "98 -             .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |next| {",
    "98 +             .try_update(Ordering::Relaxed, Ordering::Relaxed, |next| {",
    "   |",
    "",
    "warning: use of deprecated method `std::sync::atomic::Atomic::<u32>::fetch_update`: renamed to `try_update` for consistency",
    "   --> components\\genet-scripted-dom\\lib.rs:166:10",
    "    |",
    "166 |         .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |next| {",
    "    |          ^^^^^^^^^^^^",
    "    |",
    "help: replace the use of the deprecated method",
    "    |",
    "166 -         .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |next| {",
    "166 +         .try_update(Ordering::Relaxed, Ordering::Relaxed, |next| {",
    "    |",
    "",
    "warning: `genet-scripted-dom` (lib) generated 2 warnings (run `cargo fix --lib -p genet-scripted-dom` to apply 2 suggestions)",
    "   Compiling rtc v0.21.1 (https://github.com/webrtc-rs/rtc?rev=70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38#70901c3d)",
    "   Compiling genet-runtime v0.0.0 (C:\\Users\\mark_\\Code\\repos\\genet\\components\\script-runtime-api)",
    "warning: use of deprecated method `std::sync::atomic::Atomic::<u64>::fetch_update`: renamed to `try_update` for consistency",
    "  --> components\\script-runtime-api\\text_encoding.rs:22:10",
    "   |",
    "22 |         .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))",
    "   |          ^^^^^^^^^^^^",
    "   |",
    "   = note: `#[warn(deprecated)]` on by default",
    "help: replace the use of the deprecated method",
    "   |",
    "22 -         .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))",
    "22 +         .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))",
    "   |",
    "",
    "warning: unused import: `crate::OwnerResolvedCx`",
    " --> components\\script-runtime-api\\dom\\xpath_eval.rs:9:5",
    "  |",
    "9 | use crate::OwnerResolvedCx as _;",
    "  |     ^^^^^^^^^^^^^^^^^^^^^^",
    "  |",
    "  = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default",
    "",
    "warning: unused import: `crate::OwnerResolvedCx`",
    "  --> components\\script-runtime-api\\dom\\shadow.rs:14:5",
    "   |",
    "14 | use crate::OwnerResolvedCx as _;",
    "   |     ^^^^^^^^^^^^^^^^^^^^^^",
    "",
    "warning: unused import: `crate::OwnerResolvedCx`",
    "  --> components\\script-runtime-api\\dom\\markup_insertion.rs:37:5",
    "   |",
    "37 | use crate::OwnerResolvedCx as _;",
    "   |     ^^^^^^^^^^^^^^^^^^^^^^",
    "",
    "warning: `genet-runtime` (lib) generated 4 warnings (run `cargo fix --lib -p genet-runtime` to apply 1 suggestion)",
    "   Compiling murm-rtc v0.1.0 (https://github.com/merely-made/murm.git?rev=c58b70421d31ed9f2288dc44152dcb6d719552a9#c58b7042)",
    "   Compiling genet-webrtc v0.2.0 (C:\\Users\\mark_\\Code\\repos\\genet\\components\\genet-webrtc)",
    "    Finished `test` profile [unoptimized] target(s) in 3m 57s",
    "     Running tests\\browser.rs (C:\\t\\cargo-targets\\genet\\build\\debug\\deps\\browser-9acfd357843eb5f3.exe)",
    "error: test failed, to rerun pass `-p genet-webrtc --test browser`"
  ]
}
```

## 2026-11-01: receiving qualification - test

```json
{
  "recorded_at_utc": "2026-11-01T22:21:45.552017+00:00",
  "source_revision": "4ba9ff5c640fc51dd39c0c737649fe5cfbbe79ea",
  "command": [
    "cargo",
    "test",
    "-p",
    "genet-webrtc",
    "--test",
    "browser",
    "--locked",
    "--offline",
    "-j",
    "1"
  ],
  "exit_code": 101,
  "stdout": [
    "",
    "running 12 tests",
    "test vano::rollback ... ok",
    "test vano::policy_and_budgets ... ok",
    "test vano::navigation_revokes_old_handles ... FAILED",
    "test vano::document_teardown ... ok",
    "test vano::constructors ... ok",
    "test boa::transfer ... ok",
    "test vano::transfer ... ok",
    "test boa::document_teardown ... ok",
    "test boa::navigation_revokes_old_handles ... FAILED",
    "test boa::rollback ... ok",
    "test boa::policy_and_budgets ... ok",
    "test boa::constructors ... ok",
    "",
    "failures:",
    "",
    "---- vano::navigation_revokes_old_handles stdout ----",
    "",
    "thread 'vano::navigation_revokes_old_handles' (30744) panicked at components\\genet-webrtc\\tests\\browser.rs:229:5:",
    "assertion `left == right` failed",
    "  left: \"InvalidStateError,TypeError\"",
    " right: \"InvalidStateError,InvalidStateError\"",
    "note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace",
    "",
    "---- boa::navigation_revokes_old_handles stdout ----",
    "",
    "thread 'boa::navigation_revokes_old_handles' (7536) panicked at components\\genet-webrtc\\tests\\browser.rs:229:5:",
    "assertion `left == right` failed",
    "  left: \"InvalidStateError,TypeError\"",
    " right: \"InvalidStateError,InvalidStateError\"",
    "",
    "",
    "failures:",
    "    boa::navigation_revokes_old_handles",
    "    vano::navigation_revokes_old_handles",
    "",
    "test result: FAILED. 10 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 18.52s",
    ""
  ],
  "stderr": [
    "22 |         .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))",
    "   |          ^^^^^^^^^^^^",
    "   |",
    "   = note: `#[warn(deprecated)]` on by default",
    "help: replace the use of the deprecated method",
    "   |",
    "22 -         .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))",
    "22 +         .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))",
    "   |",
    "",
    "warning: unused import: `crate::OwnerResolvedCx`",
    " --> components\\script-runtime-api\\dom\\xpath_eval.rs:9:5",
    "  |",
    "9 | use crate::OwnerResolvedCx as _;",
    "  |     ^^^^^^^^^^^^^^^^^^^^^^",
    "  |",
    "  = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default",
    "",
    "warning: unused import: `crate::OwnerResolvedCx`",
    "  --> components\\script-runtime-api\\dom\\shadow.rs:14:5",
    "   |",
    "14 | use crate::OwnerResolvedCx as _;",
    "   |     ^^^^^^^^^^^^^^^^^^^^^^",
    "",
    "warning: unused import: `crate::OwnerResolvedCx`",
    "  --> components\\script-runtime-api\\dom\\markup_insertion.rs:37:5",
    "   |",
    "37 | use crate::OwnerResolvedCx as _;",
    "   |     ^^^^^^^^^^^^^^^^^^^^^^",
    "",
    "warning: `genet-runtime` (lib) generated 4 warnings (run `cargo fix --lib -p genet-runtime` to apply 1 suggestion)",
    "   Compiling genet-webrtc v0.2.0 (C:\\Users\\mark_\\Code\\repos\\genet\\components\\genet-webrtc)",
    "    Finished `test` profile [unoptimized] target(s) in 4m 56s",
    "     Running tests\\browser.rs (C:\\t\\cargo-targets\\genet\\build\\debug\\deps\\browser-9acfd357843eb5f3.exe)",
    "error: test failed, to rerun pass `-p genet-webrtc --test browser`"
  ]
}
```

## 2026-11-01: receiving qualification - test

```json
{
  "recorded_at_utc": "2026-11-01T22:28:38.125388+00:00",
  "source_revision": "f48439d1e7d6f81a9c3eb1c87e0562594007dee6",
  "command": [
    "cargo",
    "test",
    "-p",
    "genet-webrtc",
    "--test",
    "browser",
    "--locked",
    "--offline",
    "-j",
    "1"
  ],
  "exit_code": 0,
  "stdout": [
    "",
    "running 12 tests",
    "test vano::policy_and_budgets ... ok",
    "test vano::rollback ... ok",
    "test vano::transfer ... ok",
    "test vano::document_teardown ... ok",
    "test vano::navigation_revokes_old_handles ... ok",
    "test vano::constructors ... ok",
    "test boa::rollback ... ok",
    "test boa::policy_and_budgets ... ok",
    "test boa::transfer ... ok",
    "test boa::document_teardown ... ok",
    "test boa::navigation_revokes_old_handles ... ok",
    "test boa::constructors ... ok",
    "",
    "test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 13.28s",
    ""
  ],
  "stderr": [
    "22 -         .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))",
    "22 +         .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))",
    "   |",
    "",
    "warning: unused import: `crate::OwnerResolvedCx`",
    " --> components\\script-runtime-api\\dom\\xpath_eval.rs:9:5",
    "  |",
    "9 | use crate::OwnerResolvedCx as _;",
    "  |     ^^^^^^^^^^^^^^^^^^^^^^",
    "  |",
    "  = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default",
    "",
    "warning: unused import: `crate::OwnerResolvedCx`",
    "  --> components\\script-runtime-api\\dom\\shadow.rs:14:5",
    "   |",
    "14 | use crate::OwnerResolvedCx as _;",
    "   |     ^^^^^^^^^^^^^^^^^^^^^^",
    "",
    "warning: unused import: `crate::OwnerResolvedCx`",
    "  --> components\\script-runtime-api\\dom\\markup_insertion.rs:37:5",
    "   |",
    "37 | use crate::OwnerResolvedCx as _;",
    "   |     ^^^^^^^^^^^^^^^^^^^^^^",
    "",
    "warning: `genet-runtime` (lib) generated 4 warnings (run `cargo fix --lib -p genet-runtime` to apply 1 suggestion)",
    "   Compiling genet-webrtc v0.2.0 (C:\\Users\\mark_\\Code\\repos\\genet\\components\\genet-webrtc)",
    "    Finished `test` profile [unoptimized] target(s) in 50.65s",
    "     Running tests\\browser.rs (C:\\t\\cargo-targets\\genet\\build\\debug\\deps\\browser-9acfd357843eb5f3.exe)"
  ]
}
```

## 2026-11-01: receiving qualification - build

```json
{
  "recorded_at_utc": "2026-11-01T22:35:56.996778+00:00",
  "source_revision": "f48439d1e7d6f81a9c3eb1c87e0562594007dee6",
  "command": [
    "cargo",
    "build",
    "-p",
    "genet-wpt",
    "--no-default-features",
    "--features",
    "webrtc",
    "--locked",
    "--offline",
    "-j",
    "1"
  ],
  "exit_code": 0,
  "stdout": [],
  "stderr": [
    "    |",
    "455 | pub fn run_test_with_style_and_ws(",
    "    |        ^^^^^^^^^^^^^^^^^^^^^^^^^^",
    "    |",
    "    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default",
    "",
    "warning: method `assemble` is never used",
    "   --> ports\\genet-wpt\\src\\test262.rs:178:12",
    "    |",
    "162 | impl Harness {",
    "    | ------------ method in this implementation",
    "...",
    "178 |     pub fn assemble(&self, test_src: &str, meta: &Test262Meta, strict: bool) -> io::Result<String> {",
    "    |            ^^^^^^^^",
    "",
    "warning: variant `Unsupported` is never constructed",
    "   --> ports\\genet-wpt\\src\\test262.rs:270:5",
    "    |",
    "263 | pub enum Observation {",
    "    |          ----------- variant in this enum",
    "...",
    "270 |     Unsupported,",
    "    |     ^^^^^^^^^^^",
    "    |",
    "    = note: `Observation` has derived impls for the traits `Debug` and `Clone`, but these are intentionally ignored during dead code analysis",
    "",
    "warning: `genet-wpt` (bin \"genet-wpt\") generated 3 warnings",
    "    Finished `dev` profile [unoptimized] target(s) in 7m 18s"
  ]
}
```

## 2026-11-01: initial selected WPT diagnostic

The data-channel files exceeded the subprocess limit. Missing assertion identities
are recorded explicitly; this run is not an accepted matched map.

```json
{
  "schema": 1,
  "phase": "after-rtc",
  "source_revision": "f48439d1e7d6f81a9c3eb1c87e0562594007dee6",
  "recorded_at_utc": "2026-11-01T22:38:26.867084+00:00",
  "binary_sha256": "1a8466e4e05654b17d6b426bf1a9d2090c3d4db74df0768a7e2de388b4f805df",
  "source_sha256": {
    "tests/wpt/tests/resources/testharness.js": "d2399236c2a09c429804ff2299ad6629e17e2b53f17a74dd341e936adb11ae3e",
    "tests/wpt/tests/webrtc/RTCPeerConnection-helper.js": "9709abd9c5a024c8bdbedd1cfab8a8f2c47b9358a987905cedc2a770ded69d2d",
    "tests/wpt/tests/webrtc/RTCPeerConnection-constructor.html": "31cf6a8f9f9d9c36d4a00a6ca1c224b493b6ee39b848dc4e380ce5b96988d074",
    "tests/wpt/tests/webrtc/RTCIceCandidate-constructor.html": "9bab537b6f3618bda3410d37bba9d63bbdec6116ebe9958759e67fb9fe7d5449",
    "tests/wpt/tests/webrtc/RTCPeerConnection-createDataChannel.html": "0356665f81d466804a4fd06d73075cc03767a5b4944e3f7d62e1da002c49c71f"
  },
  "outcomes": [
    {
      "detail": null,
      "failures": [],
      "reason": null,
      "status": "pass",
      "subtests": [
        {
          "name": "RTCPeerConnection.length",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection()",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection(null)",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection(undefined)",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({})",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: null })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: undefined })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: [] })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: [null] })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: [undefined] })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ iceCandidatePoolSize: toNumberThrows })",
          "status": "pass"
        },
        {
          "name": "localDescription initial value",
          "status": "pass"
        },
        {
          "name": "currentLocalDescription initial value",
          "status": "pass"
        },
        {
          "name": "pendingLocalDescription initial value",
          "status": "pass"
        },
        {
          "name": "remoteDescription initial value",
          "status": "pass"
        },
        {
          "name": "currentRemoteDescription initial value",
          "status": "pass"
        },
        {
          "name": "pendingRemoteDescription initial value",
          "status": "pass"
        },
        {
          "name": "signalingState initial value",
          "status": "pass"
        },
        {
          "name": "iceGatheringState initial value",
          "status": "pass"
        },
        {
          "name": "iceConnectionState initial value",
          "status": "pass"
        },
        {
          "name": "connectionState initial value",
          "status": "pass"
        },
        {
          "name": "canTrickleIceCandidates initial value",
          "status": "pass"
        }
      ],
      "subtests_passed": 22,
      "subtests_total": 22,
      "engine": "boa",
      "test": "webrtc/RTCPeerConnection-constructor.html",
      "exit_code": 0,
      "command": [
        "C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe",
        "testharness-one",
        "webrtc/RTCPeerConnection-constructor.html",
        "--test-path",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCPeerConnection-constructor.html",
        "--tests-root",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests",
        "--engine",
        "boa",
        "--drive-deadline",
        "5"
      ]
    },
    {
      "detail": null,
      "failures": [
        "        [1] new RTCIceCandidate({ ... }) with nondefault values for all fields assert_equals: relayProtocol expected (string) \"udp\" but got (object) null",
        "        [1] new RTCIceCandidate({ ... }) with nondefault values for all fields, tcp candidate assert_equals: url expected (string) \"turn:turn.example.net\" but got (object) null",
        "        [1] new RTCIceCandidate({ relayProtocol, url }) cloned vs signaled assert_equals: relayProtocol cloned expected (string) \"tls\" but got (object) null"
      ],
      "reason": null,
      "status": "fail",
      "subtests": [
        {
          "name": "new RTCIceCandidate()",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({})",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with manually filled default values",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ sdpMid: null, sdpMLineIndex: null })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ candidate: '' })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ candidate: null })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with valid candidate string only",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ sdpMid: 'audio' })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ sdpMLineIndex: 0 })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ sdpMid: 'audio', sdpMLineIndex: 0 })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ candidate: '', sdpMid: 'audio' }",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ candidate: null, sdpMLineIndex: 0 }",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with valid candidate string and sdpMid",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with invalid candidate string and sdpMid",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for most fields",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for most fields, tcp candidate",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for all fields",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for all fields, tcp candidate",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ relayProtocol, url }) cloned vs signaled",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with invalid sdpMid",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with invalid sdpMLineIndex",
          "status": "pass"
        }
      ],
      "subtests_passed": 18,
      "subtests_total": 21,
      "engine": "boa",
      "test": "webrtc/RTCIceCandidate-constructor.html",
      "exit_code": 0,
      "command": [
        "C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe",
        "testharness-one",
        "webrtc/RTCIceCandidate-constructor.html",
        "--test-path",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCIceCandidate-constructor.html",
        "--tests-root",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests",
        "--engine",
        "boa",
        "--drive-deadline",
        "5"
      ]
    },
    {
      "engine": "boa",
      "test": "webrtc/RTCPeerConnection-createDataChannel.html",
      "status": "hang-killed",
      "command": [
        "C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe",
        "testharness-one",
        "webrtc/RTCPeerConnection-createDataChannel.html",
        "--test-path",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCPeerConnection-createDataChannel.html",
        "--tests-root",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests",
        "--engine",
        "boa",
        "--drive-deadline",
        "5"
      ]
    },
    {
      "detail": null,
      "failures": [],
      "reason": null,
      "status": "pass",
      "subtests": [
        {
          "name": "RTCPeerConnection.length",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection()",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection(null)",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection(undefined)",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({})",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: null })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: undefined })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: [] })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: [null] })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: [undefined] })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ iceCandidatePoolSize: toNumberThrows })",
          "status": "pass"
        },
        {
          "name": "localDescription initial value",
          "status": "pass"
        },
        {
          "name": "currentLocalDescription initial value",
          "status": "pass"
        },
        {
          "name": "pendingLocalDescription initial value",
          "status": "pass"
        },
        {
          "name": "remoteDescription initial value",
          "status": "pass"
        },
        {
          "name": "currentRemoteDescription initial value",
          "status": "pass"
        },
        {
          "name": "pendingRemoteDescription initial value",
          "status": "pass"
        },
        {
          "name": "signalingState initial value",
          "status": "pass"
        },
        {
          "name": "iceGatheringState initial value",
          "status": "pass"
        },
        {
          "name": "iceConnectionState initial value",
          "status": "pass"
        },
        {
          "name": "connectionState initial value",
          "status": "pass"
        },
        {
          "name": "canTrickleIceCandidates initial value",
          "status": "pass"
        }
      ],
      "subtests_passed": 22,
      "subtests_total": 22,
      "engine": "nova",
      "test": "webrtc/RTCPeerConnection-constructor.html",
      "exit_code": 0,
      "command": [
        "C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe",
        "testharness-one",
        "webrtc/RTCPeerConnection-constructor.html",
        "--test-path",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCPeerConnection-constructor.html",
        "--tests-root",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests",
        "--engine",
        "nova",
        "--drive-deadline",
        "5"
      ]
    },
    {
      "detail": null,
      "failures": [
        "        [1] new RTCIceCandidate({ ... }) with nondefault values for all fields assert_equals: relayProtocol expected (string) \"udp\" but got (object) null",
        "        [1] new RTCIceCandidate({ ... }) with nondefault values for all fields, tcp candidate assert_equals: url expected (string) \"turn:turn.example.net\" but got (object) null",
        "        [1] new RTCIceCandidate({ relayProtocol, url }) cloned vs signaled assert_equals: relayProtocol cloned expected (string) \"tls\" but got (object) null"
      ],
      "reason": null,
      "status": "fail",
      "subtests": [
        {
          "name": "new RTCIceCandidate()",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({})",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with manually filled default values",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ sdpMid: null, sdpMLineIndex: null })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ candidate: '' })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ candidate: null })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with valid candidate string only",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ sdpMid: 'audio' })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ sdpMLineIndex: 0 })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ sdpMid: 'audio', sdpMLineIndex: 0 })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ candidate: '', sdpMid: 'audio' }",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ candidate: null, sdpMLineIndex: 0 }",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with valid candidate string and sdpMid",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with invalid candidate string and sdpMid",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for most fields",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for most fields, tcp candidate",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for all fields",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for all fields, tcp candidate",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ relayProtocol, url }) cloned vs signaled",
          "status": "fail"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with invalid sdpMid",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with invalid sdpMLineIndex",
          "status": "pass"
        }
      ],
      "subtests_passed": 18,
      "subtests_total": 21,
      "engine": "nova",
      "test": "webrtc/RTCIceCandidate-constructor.html",
      "exit_code": 0,
      "command": [
        "C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe",
        "testharness-one",
        "webrtc/RTCIceCandidate-constructor.html",
        "--test-path",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCIceCandidate-constructor.html",
        "--tests-root",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests",
        "--engine",
        "nova",
        "--drive-deadline",
        "5"
      ]
    },
    {
      "engine": "nova",
      "test": "webrtc/RTCPeerConnection-createDataChannel.html",
      "status": "hang-killed",
      "command": [
        "C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe",
        "testharness-one",
        "webrtc/RTCPeerConnection-createDataChannel.html",
        "--test-path",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCPeerConnection-createDataChannel.html",
        "--tests-root",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests",
        "--engine",
        "nova",
        "--drive-deadline",
        "5"
      ]
    }
  ],
  "matched_cases": [
    {
      "engine": "boa",
      "test": "webrtc/RTCPeerConnection-constructor.html",
      "before_cases": 22,
      "after_cases": 22,
      "missing": [],
      "added": [],
      "improved": 22,
      "regressed": 0
    },
    {
      "engine": "boa",
      "test": "webrtc/RTCIceCandidate-constructor.html",
      "before_cases": 21,
      "after_cases": 21,
      "missing": [],
      "added": [],
      "improved": 18,
      "regressed": 0
    },
    {
      "engine": "boa",
      "test": "webrtc/RTCPeerConnection-createDataChannel.html",
      "before_cases": 52,
      "after_cases": 0,
      "missing": [
        [
          "Channels created (after SCTP connected) should have id assigned",
          1
        ],
        [
          "New datachannel should be in the connecting state after creation (after connection establishment)",
          1
        ],
        [
          "New negotiated datachannel should be in the connecting state after creation (after connection establishment)",
          1
        ],
        [
          "Renegotiation with audio/video and datachannel should work properly",
          1
        ],
        [
          "Renegotiation with audio/video and negotiated datachannel should work properly",
          1
        ],
        [
          "Reusing a data channel id that is in use (after setRemoteDescription) should throw OperationError",
          1
        ],
        [
          "Reusing a data channel id that is in use (after setRemoteDescription, negotiated via DCEP) should throw OperationError",
          1
        ],
        [
          "Reusing a data channel id that is in use should throw OperationError",
          1
        ],
        [
          "addTrack, then creating datachannel, should negotiate properly",
          1
        ],
        [
          "addTrack, then creating datachannel, should negotiate properly when max-bundle is used",
          1
        ],
        [
          "addTrack, then creating negotiated datachannel, should negotiate properly",
          1
        ],
        [
          "addTrack, then creating negotiated datachannel, should negotiate properly when max-bundle is used",
          1
        ],
        [
          "createDataChannel attribute default values",
          1
        ],
        [
          "createDataChannel with both maxPacketLifeTime and maxRetransmits should throw TypeError",
          1
        ],
        [
          "createDataChannel with both maxPacketLifeTime and maxRetransmits undefined should succeed",
          1
        ],
        [
          "createDataChannel with closed connection should throw InvalidStateError",
          1
        ],
        [
          "createDataChannel with id -1 and negotiated not set should throw TypeError",
          1
        ],
        [
          "createDataChannel with id -1 should throw TypeError",
          1
        ],
        [
          "createDataChannel with id 0 and negotiated not set should succeed, but not set the channel's id",
          1
        ],
        [
          "createDataChannel with id 0 and negotiated true should succeed, and set the channel's id",
          1
        ],
        [
          "createDataChannel with id 1 and negotiated not set should succeed, but not set the channel's id",
          1
        ],
        [
          "createDataChannel with id 1 and negotiated true should succeed, and set the channel's id",
          1
        ],
        [
          "createDataChannel with id 65534 and negotiated not set should succeed, but not set the channel's id",
          1
        ],
        [
          "createDataChannel with id 65534 and negotiated true should succeed, and set the channel's id",
          1
        ],
        [
          "createDataChannel with id 65535 and negotiated not set should succeed, but not set the channel's id",
          1
        ],
        [
          "createDataChannel with id 65535 should throw TypeError",
          1
        ],
        [
          "createDataChannel with id 65536 and negotiated not set should throw TypeError",
          1
        ],
        [
          "createDataChannel with id 65536 should throw TypeError",
          1
        ],
        [
          "createDataChannel with label \"foo\" should succeed",
          1
        ],
        [
          "createDataChannel with label lone surrogate should succeed",
          1
        ],
        [
          "createDataChannel with label null should succeed",
          1
        ],
        [
          "createDataChannel with label undefined should succeed",
          1
        ],
        [
          "createDataChannel with maxPacketLifeTime 0 should succeed",
          1
        ],
        [
          "createDataChannel with maxRetransmits 0 should succeed",
          1
        ],
        [
          "createDataChannel with maximum length label and protocol should succeed",
          1
        ],
        [
          "createDataChannel with negotiated false and id 42 should ignore the id",
          1
        ],
        [
          "createDataChannel with negotiated false should succeed",
          1
        ],
        [
          "createDataChannel with negotiated true and id not defined should throw TypeError",
          1
        ],
        [
          "createDataChannel with negotiated true and id should succeed",
          1
        ],
        [
          "createDataChannel with no argument should throw TypeError",
          1
        ],
        [
          "createDataChannel with ordered false should succeed",
          1
        ],
        [
          "createDataChannel with ordered null/undefined should succeed",
          1
        ],
        [
          "createDataChannel with protocol \"foo\" should succeed",
          1
        ],
        [
          "createDataChannel with protocol lone surrogate should succeed",
          1
        ],
        [
          "createDataChannel with protocol null should succeed",
          1
        ],
        [
          "createDataChannel with protocol undefined should succeed",
          1
        ],
        [
          "createDataChannel with provided parameters should initialize attributes to provided values",
          1
        ],
        [
          "createDataChannel with same label used twice should not throw",
          1
        ],
        [
          "createDataChannel with too long label (2 byte unicode) should throw TypeError",
          1
        ],
        [
          "createDataChannel with too long label should throw TypeError",
          1
        ],
        [
          "createDataChannel with too long protocol (2 byte unicode) should throw TypeError",
          1
        ],
        [
          "createDataChannel with too long protocol should throw TypeError",
          1
        ]
      ],
      "added": [],
      "improved": 0,
      "regressed": 0
    },
    {
      "engine": "nova",
      "test": "webrtc/RTCPeerConnection-constructor.html",
      "before_cases": 22,
      "after_cases": 22,
      "missing": [],
      "added": [],
      "improved": 22,
      "regressed": 0
    },
    {
      "engine": "nova",
      "test": "webrtc/RTCIceCandidate-constructor.html",
      "before_cases": 21,
      "after_cases": 21,
      "missing": [],
      "added": [],
      "improved": 18,
      "regressed": 0
    },
    {
      "engine": "nova",
      "test": "webrtc/RTCPeerConnection-createDataChannel.html",
      "before_cases": 52,
      "after_cases": 0,
      "missing": [
        [
          "Channels created (after SCTP connected) should have id assigned",
          1
        ],
        [
          "New datachannel should be in the connecting state after creation (after connection establishment)",
          1
        ],
        [
          "New negotiated datachannel should be in the connecting state after creation (after connection establishment)",
          1
        ],
        [
          "Renegotiation with audio/video and datachannel should work properly",
          1
        ],
        [
          "Renegotiation with audio/video and negotiated datachannel should work properly",
          1
        ],
        [
          "Reusing a data channel id that is in use (after setRemoteDescription) should throw OperationError",
          1
        ],
        [
          "Reusing a data channel id that is in use (after setRemoteDescription, negotiated via DCEP) should throw OperationError",
          1
        ],
        [
          "Reusing a data channel id that is in use should throw OperationError",
          1
        ],
        [
          "addTrack, then creating datachannel, should negotiate properly",
          1
        ],
        [
          "addTrack, then creating datachannel, should negotiate properly when max-bundle is used",
          1
        ],
        [
          "addTrack, then creating negotiated datachannel, should negotiate properly",
          1
        ],
        [
          "addTrack, then creating negotiated datachannel, should negotiate properly when max-bundle is used",
          1
        ],
        [
          "createDataChannel attribute default values",
          1
        ],
        [
          "createDataChannel with both maxPacketLifeTime and maxRetransmits should throw TypeError",
          1
        ],
        [
          "createDataChannel with both maxPacketLifeTime and maxRetransmits undefined should succeed",
          1
        ],
        [
          "createDataChannel with closed connection should throw InvalidStateError",
          1
        ],
        [
          "createDataChannel with id -1 and negotiated not set should throw TypeError",
          1
        ],
        [
          "createDataChannel with id -1 should throw TypeError",
          1
        ],
        [
          "createDataChannel with id 0 and negotiated not set should succeed, but not set the channel's id",
          1
        ],
        [
          "createDataChannel with id 0 and negotiated true should succeed, and set the channel's id",
          1
        ],
        [
          "createDataChannel with id 1 and negotiated not set should succeed, but not set the channel's id",
          1
        ],
        [
          "createDataChannel with id 1 and negotiated true should succeed, and set the channel's id",
          1
        ],
        [
          "createDataChannel with id 65534 and negotiated not set should succeed, but not set the channel's id",
          1
        ],
        [
          "createDataChannel with id 65534 and negotiated true should succeed, and set the channel's id",
          1
        ],
        [
          "createDataChannel with id 65535 and negotiated not set should succeed, but not set the channel's id",
          1
        ],
        [
          "createDataChannel with id 65535 should throw TypeError",
          1
        ],
        [
          "createDataChannel with id 65536 and negotiated not set should throw TypeError",
          1
        ],
        [
          "createDataChannel with id 65536 should throw TypeError",
          1
        ],
        [
          "createDataChannel with label \"foo\" should succeed",
          1
        ],
        [
          "createDataChannel with label lone surrogate should succeed",
          1
        ],
        [
          "createDataChannel with label null should succeed",
          1
        ],
        [
          "createDataChannel with label undefined should succeed",
          1
        ],
        [
          "createDataChannel with maxPacketLifeTime 0 should succeed",
          1
        ],
        [
          "createDataChannel with maxRetransmits 0 should succeed",
          1
        ],
        [
          "createDataChannel with maximum length label and protocol should succeed",
          1
        ],
        [
          "createDataChannel with negotiated false and id 42 should ignore the id",
          1
        ],
        [
          "createDataChannel with negotiated false should succeed",
          1
        ],
        [
          "createDataChannel with negotiated true and id not defined should throw TypeError",
          1
        ],
        [
          "createDataChannel with negotiated true and id should succeed",
          1
        ],
        [
          "createDataChannel with no argument should throw TypeError",
          1
        ],
        [
          "createDataChannel with ordered false should succeed",
          1
        ],
        [
          "createDataChannel with ordered null/undefined should succeed",
          1
        ],
        [
          "createDataChannel with protocol \"foo\" should succeed",
          1
        ],
        [
          "createDataChannel with protocol lone surrogate should succeed",
          1
        ],
        [
          "createDataChannel with protocol null should succeed",
          1
        ],
        [
          "createDataChannel with protocol undefined should succeed",
          1
        ],
        [
          "createDataChannel with provided parameters should initialize attributes to provided values",
          1
        ],
        [
          "createDataChannel with same label used twice should not throw",
          1
        ],
        [
          "createDataChannel with too long label (2 byte unicode) should throw TypeError",
          1
        ],
        [
          "createDataChannel with too long label should throw TypeError",
          1
        ],
        [
          "createDataChannel with too long protocol (2 byte unicode) should throw TypeError",
          1
        ],
        [
          "createDataChannel with too long protocol should throw TypeError",
          1
        ]
      ],
      "added": [],
      "improved": 0,
      "regressed": 0
    }
  ],
  "scope": "Three unmodified source files at unvaried URLs; native host-only loopback. Not a manifest-wide RTC coverage result or headed interoperability."
}
```

## 2026-11-01: receiving qualification - test

```json
{
  "source_revision": "735c0b660094b480243ce8986c8ac5ad9e655c5a",
  "recorded_at_utc": "2026-11-01T22:47:28.426681+00:00",
  "command": [
    "cargo",
    "test",
    "-p",
    "genet-webrtc",
    "--test",
    "browser",
    "--locked",
    "--offline",
    "-j",
    "1"
  ],
  "exit_code": 0,
  "stdout": [
    "",
    "running 14 tests",
    "test vano::transfer ... ok",
    "test vano::policy_and_budgets ... ok",
    "test vano::rollback ... ok",
    "test vano::strings_and_utf8_limits ... ok",
    "test vano::document_teardown ... ok",
    "test vano::navigation_revokes_old_handles ... ok",
    "test vano::constructors ... ok",
    "test boa::strings_and_utf8_limits ... ok",
    "test boa::transfer ... ok",
    "test boa::rollback ... ok",
    "test boa::policy_and_budgets ... ok",
    "test boa::navigation_revokes_old_handles ... ok",
    "test boa::document_teardown ... ok",
    "test boa::constructors ... ok",
    "",
    "test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 16.50s",
    ""
  ],
  "stderr": [
    "warning: unused import: `crate::OwnerResolvedCx`",
    " --> components\\script-runtime-api\\dom\\xpath_eval.rs:9:5",
    "  |",
    "9 | use crate::OwnerResolvedCx as _;",
    "  |     ^^^^^^^^^^^^^^^^^^^^^^",
    "  |",
    "  = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default",
    "",
    "warning: unused import: `crate::OwnerResolvedCx`",
    "  --> components\\script-runtime-api\\dom\\shadow.rs:14:5",
    "   |",
    "14 | use crate::OwnerResolvedCx as _;",
    "   |     ^^^^^^^^^^^^^^^^^^^^^^",
    "",
    "warning: unused import: `crate::OwnerResolvedCx`",
    "  --> components\\script-runtime-api\\dom\\markup_insertion.rs:37:5",
    "   |",
    "37 | use crate::OwnerResolvedCx as _;",
    "   |     ^^^^^^^^^^^^^^^^^^^^^^",
    "",
    "warning: `genet-runtime` (lib) generated 4 warnings (run `cargo fix --lib -p genet-runtime` to apply 1 suggestion)",
    "   Compiling genet-webrtc v0.2.0 (C:\\Users\\mark_\\Code\\repos\\genet\\components\\genet-webrtc)",
    "    Finished `test` profile [unoptimized] target(s) in 52.63s",
    "     Running tests\\browser.rs (C:\\t\\cargo-targets\\genet\\build\\debug\\deps\\browser-9acfd357843eb5f3.exe)"
  ]
}
```

## 2026-11-01: receiving qualification - build

```json
{
  "source_revision": "735c0b660094b480243ce8986c8ac5ad9e655c5a",
  "recorded_at_utc": "2026-11-01T22:48:37.897001+00:00",
  "command": [
    "cargo",
    "build",
    "-p",
    "genet-wpt",
    "--no-default-features",
    "--features",
    "webrtc",
    "--locked",
    "--offline",
    "-j",
    "1"
  ],
  "exit_code": 0,
  "stdout": [],
  "stderr": [
    "    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default",
    "",
    "warning: method `assemble` is never used",
    "   --> ports\\genet-wpt\\src\\test262.rs:178:12",
    "    |",
    "162 | impl Harness {",
    "    | ------------ method in this implementation",
    "...",
    "178 |     pub fn assemble(&self, test_src: &str, meta: &Test262Meta, strict: bool) -> io::Result<String> {",
    "    |            ^^^^^^^^",
    "",
    "warning: variant `Unsupported` is never constructed",
    "   --> ports\\genet-wpt\\src\\test262.rs:270:5",
    "    |",
    "263 | pub enum Observation {",
    "    |          ----------- variant in this enum",
    "...",
    "270 |     Unsupported,",
    "    |     ^^^^^^^^^^^",
    "    |",
    "    = note: `Observation` has derived impls for the traits `Debug` and `Clone`, but these are intentionally ignored during dead code analysis",
    "",
    "warning: `genet-wpt` (bin \"genet-wpt\") generated 3 warnings",
    "    Finished `dev` profile [unoptimized] target(s) in 1m 08s"
  ]
}
```

## 2026-11-01: matched selected WPT - after RTC

```json
{
  "schema": 1,
  "phase": "after-rtc",
  "source_revision": "735c0b660094b480243ce8986c8ac5ad9e655c5a",
  "binary_sha256": "b46ebdd59dcdd5924cb4a136c365f31a5781a67454e1135064df42c05310961a",
  "outcomes": [
    {
      "detail": null,
      "failures": [],
      "reason": null,
      "status": "pass",
      "subtests": [
        {
          "name": "RTCPeerConnection.length",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection()",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection(null)",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection(undefined)",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({})",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: null })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: undefined })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: [] })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: [null] })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: [undefined] })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ iceCandidatePoolSize: toNumberThrows })",
          "status": "pass"
        },
        {
          "name": "localDescription initial value",
          "status": "pass"
        },
        {
          "name": "currentLocalDescription initial value",
          "status": "pass"
        },
        {
          "name": "pendingLocalDescription initial value",
          "status": "pass"
        },
        {
          "name": "remoteDescription initial value",
          "status": "pass"
        },
        {
          "name": "currentRemoteDescription initial value",
          "status": "pass"
        },
        {
          "name": "pendingRemoteDescription initial value",
          "status": "pass"
        },
        {
          "name": "signalingState initial value",
          "status": "pass"
        },
        {
          "name": "iceGatheringState initial value",
          "status": "pass"
        },
        {
          "name": "iceConnectionState initial value",
          "status": "pass"
        },
        {
          "name": "connectionState initial value",
          "status": "pass"
        },
        {
          "name": "canTrickleIceCandidates initial value",
          "status": "pass"
        }
      ],
      "subtests_passed": 22,
      "subtests_total": 22,
      "engine": "boa",
      "test": "webrtc/RTCPeerConnection-constructor.html",
      "exit_code": 0,
      "command": [
        "C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe",
        "testharness-one",
        "webrtc/RTCPeerConnection-constructor.html",
        "--test-path",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCPeerConnection-constructor.html",
        "--tests-root",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests",
        "--engine",
        "boa",
        "--drive-deadline",
        "5"
      ]
    },
    {
      "detail": null,
      "failures": [],
      "reason": null,
      "status": "pass",
      "subtests": [
        {
          "name": "new RTCIceCandidate()",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({})",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with manually filled default values",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ sdpMid: null, sdpMLineIndex: null })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ candidate: '' })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ candidate: null })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with valid candidate string only",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ sdpMid: 'audio' })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ sdpMLineIndex: 0 })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ sdpMid: 'audio', sdpMLineIndex: 0 })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ candidate: '', sdpMid: 'audio' }",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ candidate: null, sdpMLineIndex: 0 }",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with valid candidate string and sdpMid",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with invalid candidate string and sdpMid",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for most fields",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for most fields, tcp candidate",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for all fields",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for all fields, tcp candidate",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ relayProtocol, url }) cloned vs signaled",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with invalid sdpMid",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with invalid sdpMLineIndex",
          "status": "pass"
        }
      ],
      "subtests_passed": 21,
      "subtests_total": 21,
      "engine": "boa",
      "test": "webrtc/RTCIceCandidate-constructor.html",
      "exit_code": 0,
      "command": [
        "C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe",
        "testharness-one",
        "webrtc/RTCIceCandidate-constructor.html",
        "--test-path",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCIceCandidate-constructor.html",
        "--tests-root",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests",
        "--engine",
        "boa",
        "--drive-deadline",
        "5"
      ]
    },
    {
      "detail": null,
      "failures": [
        "        [1] createDataChannel with id 65535 and negotiated not set should succeed, but not set the channel's id Reserved channel ID",
        "        [1] Reusing a data channel id that is in use should throw OperationError assert_throws_dom: function \"() =>\n    pc.createDataChannel('channel-3', {\n      negotiated: true,\n      id: 42,\n    })\" did not throw",
        "        [1] Reusing a data channel id that is in use (after setRemoteDescription) should throw OperationError assert_throws_dom: function \"() =>\n    pc1.createDataChannel('channel-3', {\n      negotiated: true,\n      id: 42,\n    })\" did not throw",
        "        [1] Reusing a data channel id that is in use (after setRemoteDescription, negotiated via DCEP) should throw OperationError assert_not_equals: Expect dc1.id to be assigned after SCTP has connected got disallowed value null",
        "        [1] addTrack, then creating datachannel, should negotiate properly promise_test: Unhandled rejection with value: object \"TypeError: cannot convert 'null' or 'undefined' to object\"",
        "        [1] addTrack, then creating datachannel, should negotiate properly when max-bundle is used promise_test: Unhandled rejection with value: object \"TypeError: cannot convert 'null' or 'undefined' to object\"",
        "        [1] Renegotiation with audio/video and datachannel should work properly promise_test: Unhandled rejection with value: object \"TypeError: cannot convert 'null' or 'undefined' to object\"",
        "        [1] addTrack, then creating negotiated datachannel, should negotiate properly promise_test: Unhandled rejection with value: object \"TypeError: cannot convert 'null' or 'undefined' to object\"",
        "        [1] addTrack, then creating negotiated datachannel, should negotiate properly when max-bundle is used promise_test: Unhandled rejection with value: object \"TypeError: cannot convert 'null' or 'undefined' to object\"",
        "        [1] Renegotiation with audio/video and negotiated datachannel should work properly promise_test: Unhandled rejection with value: object \"TypeError: cannot convert 'null' or 'undefined' to object\""
      ],
      "reason": null,
      "status": "fail",
      "subtests": [
        {
          "name": "createDataChannel with no argument should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with closed connection should throw InvalidStateError",
          "status": "pass"
        },
        {
          "name": "createDataChannel attribute default values",
          "status": "pass"
        },
        {
          "name": "createDataChannel with provided parameters should initialize attributes to provided values",
          "status": "pass"
        },
        {
          "name": "createDataChannel with label \"foo\" should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with label null should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with label undefined should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with label lone surrogate should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with ordered false should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with ordered null/undefined should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with maxPacketLifeTime 0 should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with maxRetransmits 0 should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with both maxPacketLifeTime and maxRetransmits undefined should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with both maxPacketLifeTime and maxRetransmits should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with protocol \"foo\" should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with protocol null should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with protocol undefined should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with protocol lone surrogate should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 0 and negotiated not set should succeed, but not set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 1 and negotiated not set should succeed, but not set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65534 and negotiated not set should succeed, but not set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65535 and negotiated not set should succeed, but not set the channel's id",
          "status": "fail"
        },
        {
          "name": "createDataChannel with id 0 and negotiated true should succeed, and set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 1 and negotiated true should succeed, and set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65534 and negotiated true should succeed, and set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id -1 and negotiated not set should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65536 and negotiated not set should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id -1 should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65535 should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65536 should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with too long label should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with too long label (2 byte unicode) should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with same label used twice should not throw",
          "status": "pass"
        },
        {
          "name": "createDataChannel with negotiated true and id should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with too long protocol should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with too long protocol (2 byte unicode) should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with maximum length label and protocol should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with negotiated false should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with negotiated false and id 42 should ignore the id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with negotiated true and id not defined should throw TypeError",
          "status": "pass"
        },
        {
          "name": "Channels created (after SCTP connected) should have id assigned",
          "status": "pass"
        },
        {
          "name": "Reusing a data channel id that is in use should throw OperationError",
          "status": "fail"
        },
        {
          "name": "Reusing a data channel id that is in use (after setRemoteDescription) should throw OperationError",
          "status": "fail"
        },
        {
          "name": "Reusing a data channel id that is in use (after setRemoteDescription, negotiated via DCEP) should throw OperationError",
          "status": "fail"
        },
        {
          "name": "New datachannel should be in the connecting state after creation (after connection establishment)",
          "status": "pass"
        },
        {
          "name": "addTrack, then creating datachannel, should negotiate properly",
          "status": "fail"
        },
        {
          "name": "addTrack, then creating datachannel, should negotiate properly when max-bundle is used",
          "status": "fail"
        },
        {
          "name": "Renegotiation with audio/video and datachannel should work properly",
          "status": "fail"
        },
        {
          "name": "New negotiated datachannel should be in the connecting state after creation (after connection establishment)",
          "status": "pass"
        },
        {
          "name": "addTrack, then creating negotiated datachannel, should negotiate properly",
          "status": "fail"
        },
        {
          "name": "addTrack, then creating negotiated datachannel, should negotiate properly when max-bundle is used",
          "status": "fail"
        },
        {
          "name": "Renegotiation with audio/video and negotiated datachannel should work properly",
          "status": "fail"
        }
      ],
      "subtests_passed": 42,
      "subtests_total": 52,
      "engine": "boa",
      "test": "webrtc/RTCPeerConnection-createDataChannel.html",
      "exit_code": 0,
      "command": [
        "C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe",
        "testharness-one",
        "webrtc/RTCPeerConnection-createDataChannel.html",
        "--test-path",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCPeerConnection-createDataChannel.html",
        "--tests-root",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests",
        "--engine",
        "boa",
        "--drive-deadline",
        "5"
      ]
    },
    {
      "detail": null,
      "failures": [],
      "reason": null,
      "status": "pass",
      "subtests": [
        {
          "name": "RTCPeerConnection.length",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection()",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection(null)",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection(undefined)",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({})",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: null })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: undefined })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: [] })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: [null] })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: [undefined] })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ iceCandidatePoolSize: toNumberThrows })",
          "status": "pass"
        },
        {
          "name": "localDescription initial value",
          "status": "pass"
        },
        {
          "name": "currentLocalDescription initial value",
          "status": "pass"
        },
        {
          "name": "pendingLocalDescription initial value",
          "status": "pass"
        },
        {
          "name": "remoteDescription initial value",
          "status": "pass"
        },
        {
          "name": "currentRemoteDescription initial value",
          "status": "pass"
        },
        {
          "name": "pendingRemoteDescription initial value",
          "status": "pass"
        },
        {
          "name": "signalingState initial value",
          "status": "pass"
        },
        {
          "name": "iceGatheringState initial value",
          "status": "pass"
        },
        {
          "name": "iceConnectionState initial value",
          "status": "pass"
        },
        {
          "name": "connectionState initial value",
          "status": "pass"
        },
        {
          "name": "canTrickleIceCandidates initial value",
          "status": "pass"
        }
      ],
      "subtests_passed": 22,
      "subtests_total": 22,
      "engine": "nova",
      "test": "webrtc/RTCPeerConnection-constructor.html",
      "exit_code": 0,
      "command": [
        "C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe",
        "testharness-one",
        "webrtc/RTCPeerConnection-constructor.html",
        "--test-path",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCPeerConnection-constructor.html",
        "--tests-root",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests",
        "--engine",
        "nova",
        "--drive-deadline",
        "5"
      ]
    },
    {
      "detail": null,
      "failures": [],
      "reason": null,
      "status": "pass",
      "subtests": [
        {
          "name": "new RTCIceCandidate()",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({})",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with manually filled default values",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ sdpMid: null, sdpMLineIndex: null })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ candidate: '' })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ candidate: null })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with valid candidate string only",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ sdpMid: 'audio' })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ sdpMLineIndex: 0 })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ sdpMid: 'audio', sdpMLineIndex: 0 })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ candidate: '', sdpMid: 'audio' }",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ candidate: null, sdpMLineIndex: 0 }",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with valid candidate string and sdpMid",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with invalid candidate string and sdpMid",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for most fields",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for most fields, tcp candidate",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for all fields",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for all fields, tcp candidate",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ relayProtocol, url }) cloned vs signaled",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with invalid sdpMid",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with invalid sdpMLineIndex",
          "status": "pass"
        }
      ],
      "subtests_passed": 21,
      "subtests_total": 21,
      "engine": "nova",
      "test": "webrtc/RTCIceCandidate-constructor.html",
      "exit_code": 0,
      "command": [
        "C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe",
        "testharness-one",
        "webrtc/RTCIceCandidate-constructor.html",
        "--test-path",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCIceCandidate-constructor.html",
        "--tests-root",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests",
        "--engine",
        "nova",
        "--drive-deadline",
        "5"
      ]
    },
    {
      "detail": null,
      "failures": [
        "        [1] createDataChannel with id 65535 and negotiated not set should succeed, but not set the channel's id Reserved channel ID",
        "        [1] Reusing a data channel id that is in use should throw OperationError assert_throws_dom: function \"() =>\n    pc.createDataChannel('channel-3', {\n      negotiated: true,\n      id: 42,\n    })\" did not throw",
        "        [1] Reusing a data channel id that is in use (after setRemoteDescription) should throw OperationError assert_throws_dom: function \"() =>\n    pc1.createDataChannel('channel-3', {\n      negotiated: true,\n      id: 42,\n    })\" did not throw",
        "        [1] Reusing a data channel id that is in use (after setRemoteDescription, negotiated via DCEP) should throw OperationError assert_not_equals: Expect dc1.id to be assigned after SCTP has connected got disallowed value null",
        "        [1] addTrack, then creating datachannel, should negotiate properly promise_test: Unhandled rejection with value: object \"TypeError: Cannot read property 'getUserMedia' of undefined.\"",
        "        [1] addTrack, then creating datachannel, should negotiate properly when max-bundle is used promise_test: Unhandled rejection with value: object \"TypeError: Cannot read property 'getUserMedia' of undefined.\"",
        "        [1] Renegotiation with audio/video and datachannel should work properly promise_test: Unhandled rejection with value: object \"TypeError: Cannot read property 'getUserMedia' of undefined.\"",
        "        [1] addTrack, then creating negotiated datachannel, should negotiate properly promise_test: Unhandled rejection with value: object \"TypeError: Cannot read property 'getUserMedia' of undefined.\"",
        "        [1] addTrack, then creating negotiated datachannel, should negotiate properly when max-bundle is used promise_test: Unhandled rejection with value: object \"TypeError: Cannot read property 'getUserMedia' of undefined.\"",
        "        [1] Renegotiation with audio/video and negotiated datachannel should work properly promise_test: Unhandled rejection with value: object \"TypeError: Cannot read property 'getUserMedia' of undefined.\""
      ],
      "reason": null,
      "status": "fail",
      "subtests": [
        {
          "name": "createDataChannel with no argument should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with closed connection should throw InvalidStateError",
          "status": "pass"
        },
        {
          "name": "createDataChannel attribute default values",
          "status": "pass"
        },
        {
          "name": "createDataChannel with provided parameters should initialize attributes to provided values",
          "status": "pass"
        },
        {
          "name": "createDataChannel with label \"foo\" should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with label null should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with label undefined should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with label lone surrogate should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with ordered false should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with ordered null/undefined should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with maxPacketLifeTime 0 should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with maxRetransmits 0 should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with both maxPacketLifeTime and maxRetransmits undefined should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with both maxPacketLifeTime and maxRetransmits should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with protocol \"foo\" should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with protocol null should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with protocol undefined should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with protocol lone surrogate should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 0 and negotiated not set should succeed, but not set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 1 and negotiated not set should succeed, but not set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65534 and negotiated not set should succeed, but not set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65535 and negotiated not set should succeed, but not set the channel's id",
          "status": "fail"
        },
        {
          "name": "createDataChannel with id 0 and negotiated true should succeed, and set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 1 and negotiated true should succeed, and set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65534 and negotiated true should succeed, and set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id -1 and negotiated not set should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65536 and negotiated not set should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id -1 should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65535 should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65536 should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with too long label should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with too long label (2 byte unicode) should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with same label used twice should not throw",
          "status": "pass"
        },
        {
          "name": "createDataChannel with negotiated true and id should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with too long protocol should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with too long protocol (2 byte unicode) should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with maximum length label and protocol should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with negotiated false should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with negotiated false and id 42 should ignore the id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with negotiated true and id not defined should throw TypeError",
          "status": "pass"
        },
        {
          "name": "Channels created (after SCTP connected) should have id assigned",
          "status": "pass"
        },
        {
          "name": "Reusing a data channel id that is in use should throw OperationError",
          "status": "fail"
        },
        {
          "name": "Reusing a data channel id that is in use (after setRemoteDescription) should throw OperationError",
          "status": "fail"
        },
        {
          "name": "Reusing a data channel id that is in use (after setRemoteDescription, negotiated via DCEP) should throw OperationError",
          "status": "fail"
        },
        {
          "name": "New datachannel should be in the connecting state after creation (after connection establishment)",
          "status": "pass"
        },
        {
          "name": "addTrack, then creating datachannel, should negotiate properly",
          "status": "fail"
        },
        {
          "name": "addTrack, then creating datachannel, should negotiate properly when max-bundle is used",
          "status": "fail"
        },
        {
          "name": "Renegotiation with audio/video and datachannel should work properly",
          "status": "fail"
        },
        {
          "name": "New negotiated datachannel should be in the connecting state after creation (after connection establishment)",
          "status": "pass"
        },
        {
          "name": "addTrack, then creating negotiated datachannel, should negotiate properly",
          "status": "fail"
        },
        {
          "name": "addTrack, then creating negotiated datachannel, should negotiate properly when max-bundle is used",
          "status": "fail"
        },
        {
          "name": "Renegotiation with audio/video and negotiated datachannel should work properly",
          "status": "fail"
        }
      ],
      "subtests_passed": 42,
      "subtests_total": 52,
      "engine": "nova",
      "test": "webrtc/RTCPeerConnection-createDataChannel.html",
      "exit_code": 0,
      "command": [
        "C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe",
        "testharness-one",
        "webrtc/RTCPeerConnection-createDataChannel.html",
        "--test-path",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCPeerConnection-createDataChannel.html",
        "--tests-root",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests",
        "--engine",
        "nova",
        "--drive-deadline",
        "5"
      ]
    }
  ],
  "matched_cases": [
    {
      "engine": "boa",
      "test": "webrtc/RTCPeerConnection-constructor.html",
      "before_cases": 22,
      "after_cases": 22,
      "missing": [],
      "added": [],
      "improved": 22,
      "regressed": 0
    },
    {
      "engine": "boa",
      "test": "webrtc/RTCIceCandidate-constructor.html",
      "before_cases": 21,
      "after_cases": 21,
      "missing": [],
      "added": [],
      "improved": 21,
      "regressed": 0
    },
    {
      "engine": "boa",
      "test": "webrtc/RTCPeerConnection-createDataChannel.html",
      "before_cases": 52,
      "after_cases": 52,
      "missing": [],
      "added": [],
      "improved": 42,
      "regressed": 0
    },
    {
      "engine": "nova",
      "test": "webrtc/RTCPeerConnection-constructor.html",
      "before_cases": 22,
      "after_cases": 22,
      "missing": [],
      "added": [],
      "improved": 22,
      "regressed": 0
    },
    {
      "engine": "nova",
      "test": "webrtc/RTCIceCandidate-constructor.html",
      "before_cases": 21,
      "after_cases": 21,
      "missing": [],
      "added": [],
      "improved": 21,
      "regressed": 0
    },
    {
      "engine": "nova",
      "test": "webrtc/RTCPeerConnection-createDataChannel.html",
      "before_cases": 52,
      "after_cases": 52,
      "missing": [],
      "added": [],
      "improved": 42,
      "regressed": 0
    }
  ],
  "source_sha256": {
    "tests/wpt/tests/resources/testharness.js": "d2399236c2a09c429804ff2299ad6629e17e2b53f17a74dd341e936adb11ae3e",
    "tests/wpt/tests/webrtc/RTCPeerConnection-helper.js": "9709abd9c5a024c8bdbedd1cfab8a8f2c47b9358a987905cedc2a770ded69d2d",
    "tests/wpt/tests/webrtc/RTCPeerConnection-constructor.html": "31cf6a8f9f9d9c36d4a00a6ca1c224b493b6ee39b848dc4e380ce5b96988d074",
    "tests/wpt/tests/webrtc/RTCIceCandidate-constructor.html": "9bab537b6f3618bda3410d37bba9d63bbdec6116ebe9958759e67fb9fe7d5449",
    "tests/wpt/tests/webrtc/RTCPeerConnection-createDataChannel.html": "0356665f81d466804a4fd06d73075cc03767a5b4944e3f7d62e1da002c49c71f"
  },
  "scope": "Same three unmodified source files at unvaried URLs, both engines, native host route only. No full RTC/manifest or headed browser qualification."
}
```

## 2026-11-01: receiving qualification - check

```json
{
  "source_revision": "735c0b660094b480243ce8986c8ac5ad9e655c5a",
  "recorded_at_utc": "2026-11-01T22:54:11.873138+00:00",
  "command": [
    "cargo",
    "check",
    "-p",
    "ortet",
    "--features",
    "scripted-nova",
    "--all-targets",
    "--locked",
    "--offline",
    "-j",
    "1"
  ],
  "exit_code": 0,
  "stdout": [],
  "stderr": [
    "",
    "warning: `genet-documents` (lib) generated 3 warnings (run `cargo fix --lib -p genet-documents` to apply 2 suggestions)",
    "warning: unused import: `SessionClick`",
    "  --> ports\\ortet\\src\\shell.rs:19:42",
    "   |",
    "19 |     DocumentSession, SessionButtonState, SessionClick, SessionCursor, SessionEffect, SessionEngine,",
    "   |                                          ^^^^^^^^^^^^",
    "   |",
    "   = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default",
    "",
    "warning: method `navigate` is never used",
    "   --> ports\\ortet\\src\\shell.rs:754:8",
    "    |",
    "462 | impl Ortet {",
    "    | ---------- method in this implementation",
    "...",
    "754 |     fn navigate(&mut self, target: &str) {",
    "    |        ^^^^^^^^",
    "    |",
    "    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default",
    "",
    "warning: `ortet` (lib) generated 2 warnings (run `cargo fix --lib -p ortet` to apply 1 suggestion)",
    "warning: `ortet` (lib test) generated 1 warning (1 duplicate)",
    "    Finished `dev` profile [unoptimized] target(s) in 4m 57s"
  ]
}
```

## 2026-11-01: receiving qualification - test

```json
{
  "source_revision": "74ddda90ccc0af8f2a9760dedb91102e60c2c1d0",
  "recorded_at_utc": "2026-11-01T23:00:59.207113+00:00",
  "command": [
    "cargo",
    "test",
    "-p",
    "genet-webrtc",
    "--test",
    "browser",
    "--locked",
    "--offline",
    "-j",
    "1"
  ],
  "exit_code": 0,
  "stdout": [
    "",
    "running 14 tests",
    "test vano::policy_and_budgets ... ok",
    "test vano::rollback ... ok",
    "test vano::strings_and_utf8_limits ... ok",
    "test vano::document_teardown ... ok",
    "test vano::transfer ... ok",
    "test vano::navigation_revokes_old_handles ... ok",
    "test vano::constructors ... ok",
    "test boa::policy_and_budgets ... ok",
    "test boa::rollback ... ok",
    "test boa::strings_and_utf8_limits ... ok",
    "test boa::transfer ... ok",
    "test boa::document_teardown ... ok",
    "test boa::navigation_revokes_old_handles ... ok",
    "test boa::constructors ... ok",
    "",
    "test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 14.96s",
    ""
  ],
  "stderr": [
    "warning: unused import: `crate::OwnerResolvedCx`",
    " --> components\\script-runtime-api\\dom\\xpath_eval.rs:9:5",
    "  |",
    "9 | use crate::OwnerResolvedCx as _;",
    "  |     ^^^^^^^^^^^^^^^^^^^^^^",
    "  |",
    "  = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default",
    "",
    "warning: unused import: `crate::OwnerResolvedCx`",
    "  --> components\\script-runtime-api\\dom\\shadow.rs:14:5",
    "   |",
    "14 | use crate::OwnerResolvedCx as _;",
    "   |     ^^^^^^^^^^^^^^^^^^^^^^",
    "",
    "warning: unused import: `crate::OwnerResolvedCx`",
    "  --> components\\script-runtime-api\\dom\\markup_insertion.rs:37:5",
    "   |",
    "37 | use crate::OwnerResolvedCx as _;",
    "   |     ^^^^^^^^^^^^^^^^^^^^^^",
    "",
    "warning: `genet-runtime` (lib) generated 4 warnings (run `cargo fix --lib -p genet-runtime` to apply 1 suggestion)",
    "   Compiling genet-webrtc v0.2.0 (C:\\Users\\mark_\\Code\\repos\\genet\\components\\genet-webrtc)",
    "    Finished `test` profile [unoptimized] target(s) in 1m 22s",
    "     Running tests\\browser.rs (C:\\t\\cargo-targets\\genet\\build\\debug\\deps\\browser-9acfd357843eb5f3.exe)"
  ]
}
```

## 2026-11-01: receiving qualification - build

```json
{
  "source_revision": "74ddda90ccc0af8f2a9760dedb91102e60c2c1d0",
  "recorded_at_utc": "2026-11-01T23:03:00.468493+00:00",
  "command": [
    "cargo",
    "build",
    "-p",
    "genet-wpt",
    "--no-default-features",
    "--features",
    "webrtc",
    "--locked",
    "--offline",
    "-j",
    "1"
  ],
  "exit_code": 0,
  "stdout": [],
  "stderr": [
    "    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default",
    "",
    "warning: method `assemble` is never used",
    "   --> ports\\genet-wpt\\src\\test262.rs:178:12",
    "    |",
    "162 | impl Harness {",
    "    | ------------ method in this implementation",
    "...",
    "178 |     pub fn assemble(&self, test_src: &str, meta: &Test262Meta, strict: bool) -> io::Result<String> {",
    "    |            ^^^^^^^^",
    "",
    "warning: variant `Unsupported` is never constructed",
    "   --> ports\\genet-wpt\\src\\test262.rs:270:5",
    "    |",
    "263 | pub enum Observation {",
    "    |          ----------- variant in this enum",
    "...",
    "270 |     Unsupported,",
    "    |     ^^^^^^^^^^^",
    "    |",
    "    = note: `Observation` has derived impls for the traits `Debug` and `Clone`, but these are intentionally ignored during dead code analysis",
    "",
    "warning: `genet-wpt` (bin \"genet-wpt\") generated 3 warnings",
    "    Finished `dev` profile [unoptimized] target(s) in 2m 01s"
  ]
}
```

## 2026-11-01: matched selected WPT - after RTC

```json
{
  "schema": 1,
  "phase": "after-rtc",
  "source_revision": "74ddda90ccc0af8f2a9760dedb91102e60c2c1d0",
  "binary_sha256": "7e02706f1b72e239a431053048f2861e0ce1e467f7bb663f3a83f574cbe315ac",
  "outcomes": [
    {
      "detail": null,
      "failures": [],
      "reason": null,
      "status": "pass",
      "subtests": [
        {
          "name": "RTCPeerConnection.length",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection()",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection(null)",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection(undefined)",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({})",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: null })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: undefined })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: [] })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: [null] })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: [undefined] })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ iceCandidatePoolSize: toNumberThrows })",
          "status": "pass"
        },
        {
          "name": "localDescription initial value",
          "status": "pass"
        },
        {
          "name": "currentLocalDescription initial value",
          "status": "pass"
        },
        {
          "name": "pendingLocalDescription initial value",
          "status": "pass"
        },
        {
          "name": "remoteDescription initial value",
          "status": "pass"
        },
        {
          "name": "currentRemoteDescription initial value",
          "status": "pass"
        },
        {
          "name": "pendingRemoteDescription initial value",
          "status": "pass"
        },
        {
          "name": "signalingState initial value",
          "status": "pass"
        },
        {
          "name": "iceGatheringState initial value",
          "status": "pass"
        },
        {
          "name": "iceConnectionState initial value",
          "status": "pass"
        },
        {
          "name": "connectionState initial value",
          "status": "pass"
        },
        {
          "name": "canTrickleIceCandidates initial value",
          "status": "pass"
        }
      ],
      "subtests_passed": 22,
      "subtests_total": 22,
      "engine": "boa",
      "test": "webrtc/RTCPeerConnection-constructor.html",
      "exit_code": 0,
      "command": [
        "C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe",
        "testharness-one",
        "webrtc/RTCPeerConnection-constructor.html",
        "--test-path",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCPeerConnection-constructor.html",
        "--tests-root",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests",
        "--engine",
        "boa",
        "--drive-deadline",
        "5"
      ]
    },
    {
      "detail": null,
      "failures": [],
      "reason": null,
      "status": "pass",
      "subtests": [
        {
          "name": "new RTCIceCandidate()",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({})",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with manually filled default values",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ sdpMid: null, sdpMLineIndex: null })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ candidate: '' })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ candidate: null })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with valid candidate string only",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ sdpMid: 'audio' })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ sdpMLineIndex: 0 })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ sdpMid: 'audio', sdpMLineIndex: 0 })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ candidate: '', sdpMid: 'audio' }",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ candidate: null, sdpMLineIndex: 0 }",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with valid candidate string and sdpMid",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with invalid candidate string and sdpMid",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for most fields",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for most fields, tcp candidate",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for all fields",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for all fields, tcp candidate",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ relayProtocol, url }) cloned vs signaled",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with invalid sdpMid",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with invalid sdpMLineIndex",
          "status": "pass"
        }
      ],
      "subtests_passed": 21,
      "subtests_total": 21,
      "engine": "boa",
      "test": "webrtc/RTCIceCandidate-constructor.html",
      "exit_code": 0,
      "command": [
        "C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe",
        "testharness-one",
        "webrtc/RTCIceCandidate-constructor.html",
        "--test-path",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCIceCandidate-constructor.html",
        "--tests-root",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests",
        "--engine",
        "boa",
        "--drive-deadline",
        "5"
      ]
    },
    {
      "detail": null,
      "failures": [
        "        [1] addTrack, then creating datachannel, should negotiate properly promise_test: Unhandled rejection with value: object \"TypeError: cannot convert 'null' or 'undefined' to object\"",
        "        [1] addTrack, then creating datachannel, should negotiate properly when max-bundle is used promise_test: Unhandled rejection with value: object \"TypeError: cannot convert 'null' or 'undefined' to object\"",
        "        [1] Renegotiation with audio/video and datachannel should work properly promise_test: Unhandled rejection with value: object \"TypeError: cannot convert 'null' or 'undefined' to object\"",
        "        [1] addTrack, then creating negotiated datachannel, should negotiate properly promise_test: Unhandled rejection with value: object \"TypeError: cannot convert 'null' or 'undefined' to object\"",
        "        [1] addTrack, then creating negotiated datachannel, should negotiate properly when max-bundle is used promise_test: Unhandled rejection with value: object \"TypeError: cannot convert 'null' or 'undefined' to object\"",
        "        [1] Renegotiation with audio/video and negotiated datachannel should work properly promise_test: Unhandled rejection with value: object \"TypeError: cannot convert 'null' or 'undefined' to object\""
      ],
      "reason": null,
      "status": "fail",
      "subtests": [
        {
          "name": "createDataChannel with no argument should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with closed connection should throw InvalidStateError",
          "status": "pass"
        },
        {
          "name": "createDataChannel attribute default values",
          "status": "pass"
        },
        {
          "name": "createDataChannel with provided parameters should initialize attributes to provided values",
          "status": "pass"
        },
        {
          "name": "createDataChannel with label \"foo\" should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with label null should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with label undefined should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with label lone surrogate should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with ordered false should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with ordered null/undefined should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with maxPacketLifeTime 0 should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with maxRetransmits 0 should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with both maxPacketLifeTime and maxRetransmits undefined should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with both maxPacketLifeTime and maxRetransmits should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with protocol \"foo\" should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with protocol null should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with protocol undefined should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with protocol lone surrogate should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 0 and negotiated not set should succeed, but not set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 1 and negotiated not set should succeed, but not set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65534 and negotiated not set should succeed, but not set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65535 and negotiated not set should succeed, but not set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 0 and negotiated true should succeed, and set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 1 and negotiated true should succeed, and set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65534 and negotiated true should succeed, and set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id -1 and negotiated not set should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65536 and negotiated not set should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id -1 should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65535 should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65536 should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with too long label should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with too long label (2 byte unicode) should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with same label used twice should not throw",
          "status": "pass"
        },
        {
          "name": "createDataChannel with negotiated true and id should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with too long protocol should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with too long protocol (2 byte unicode) should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with maximum length label and protocol should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with negotiated false should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with negotiated false and id 42 should ignore the id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with negotiated true and id not defined should throw TypeError",
          "status": "pass"
        },
        {
          "name": "Channels created (after SCTP connected) should have id assigned",
          "status": "pass"
        },
        {
          "name": "Reusing a data channel id that is in use should throw OperationError",
          "status": "pass"
        },
        {
          "name": "Reusing a data channel id that is in use (after setRemoteDescription) should throw OperationError",
          "status": "pass"
        },
        {
          "name": "Reusing a data channel id that is in use (after setRemoteDescription, negotiated via DCEP) should throw OperationError",
          "status": "pass"
        },
        {
          "name": "New datachannel should be in the connecting state after creation (after connection establishment)",
          "status": "pass"
        },
        {
          "name": "addTrack, then creating datachannel, should negotiate properly",
          "status": "fail"
        },
        {
          "name": "addTrack, then creating datachannel, should negotiate properly when max-bundle is used",
          "status": "fail"
        },
        {
          "name": "Renegotiation with audio/video and datachannel should work properly",
          "status": "fail"
        },
        {
          "name": "New negotiated datachannel should be in the connecting state after creation (after connection establishment)",
          "status": "pass"
        },
        {
          "name": "addTrack, then creating negotiated datachannel, should negotiate properly",
          "status": "fail"
        },
        {
          "name": "addTrack, then creating negotiated datachannel, should negotiate properly when max-bundle is used",
          "status": "fail"
        },
        {
          "name": "Renegotiation with audio/video and negotiated datachannel should work properly",
          "status": "fail"
        }
      ],
      "subtests_passed": 46,
      "subtests_total": 52,
      "engine": "boa",
      "test": "webrtc/RTCPeerConnection-createDataChannel.html",
      "exit_code": 0,
      "command": [
        "C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe",
        "testharness-one",
        "webrtc/RTCPeerConnection-createDataChannel.html",
        "--test-path",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCPeerConnection-createDataChannel.html",
        "--tests-root",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests",
        "--engine",
        "boa",
        "--drive-deadline",
        "5"
      ]
    },
    {
      "detail": null,
      "failures": [],
      "reason": null,
      "status": "pass",
      "subtests": [
        {
          "name": "RTCPeerConnection.length",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection()",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection(null)",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection(undefined)",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({})",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: null })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: undefined })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: [] })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: [null] })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: [undefined] })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ iceCandidatePoolSize: toNumberThrows })",
          "status": "pass"
        },
        {
          "name": "localDescription initial value",
          "status": "pass"
        },
        {
          "name": "currentLocalDescription initial value",
          "status": "pass"
        },
        {
          "name": "pendingLocalDescription initial value",
          "status": "pass"
        },
        {
          "name": "remoteDescription initial value",
          "status": "pass"
        },
        {
          "name": "currentRemoteDescription initial value",
          "status": "pass"
        },
        {
          "name": "pendingRemoteDescription initial value",
          "status": "pass"
        },
        {
          "name": "signalingState initial value",
          "status": "pass"
        },
        {
          "name": "iceGatheringState initial value",
          "status": "pass"
        },
        {
          "name": "iceConnectionState initial value",
          "status": "pass"
        },
        {
          "name": "connectionState initial value",
          "status": "pass"
        },
        {
          "name": "canTrickleIceCandidates initial value",
          "status": "pass"
        }
      ],
      "subtests_passed": 22,
      "subtests_total": 22,
      "engine": "nova",
      "test": "webrtc/RTCPeerConnection-constructor.html",
      "exit_code": 0,
      "command": [
        "C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe",
        "testharness-one",
        "webrtc/RTCPeerConnection-constructor.html",
        "--test-path",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCPeerConnection-constructor.html",
        "--tests-root",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests",
        "--engine",
        "nova",
        "--drive-deadline",
        "5"
      ]
    },
    {
      "detail": null,
      "failures": [],
      "reason": null,
      "status": "pass",
      "subtests": [
        {
          "name": "new RTCIceCandidate()",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({})",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with manually filled default values",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ sdpMid: null, sdpMLineIndex: null })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ candidate: '' })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ candidate: null })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with valid candidate string only",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ sdpMid: 'audio' })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ sdpMLineIndex: 0 })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ sdpMid: 'audio', sdpMLineIndex: 0 })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ candidate: '', sdpMid: 'audio' }",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ candidate: null, sdpMLineIndex: 0 }",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with valid candidate string and sdpMid",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with invalid candidate string and sdpMid",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for most fields",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for most fields, tcp candidate",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for all fields",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for all fields, tcp candidate",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ relayProtocol, url }) cloned vs signaled",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with invalid sdpMid",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with invalid sdpMLineIndex",
          "status": "pass"
        }
      ],
      "subtests_passed": 21,
      "subtests_total": 21,
      "engine": "nova",
      "test": "webrtc/RTCIceCandidate-constructor.html",
      "exit_code": 0,
      "command": [
        "C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe",
        "testharness-one",
        "webrtc/RTCIceCandidate-constructor.html",
        "--test-path",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCIceCandidate-constructor.html",
        "--tests-root",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests",
        "--engine",
        "nova",
        "--drive-deadline",
        "5"
      ]
    },
    {
      "detail": null,
      "failures": [
        "        [1] addTrack, then creating datachannel, should negotiate properly promise_test: Unhandled rejection with value: object \"TypeError: Cannot read property 'getUserMedia' of undefined.\"",
        "        [1] addTrack, then creating datachannel, should negotiate properly when max-bundle is used promise_test: Unhandled rejection with value: object \"TypeError: Cannot read property 'getUserMedia' of undefined.\"",
        "        [1] Renegotiation with audio/video and datachannel should work properly promise_test: Unhandled rejection with value: object \"TypeError: Cannot read property 'getUserMedia' of undefined.\"",
        "        [1] addTrack, then creating negotiated datachannel, should negotiate properly promise_test: Unhandled rejection with value: object \"TypeError: Cannot read property 'getUserMedia' of undefined.\"",
        "        [1] addTrack, then creating negotiated datachannel, should negotiate properly when max-bundle is used promise_test: Unhandled rejection with value: object \"TypeError: Cannot read property 'getUserMedia' of undefined.\"",
        "        [1] Renegotiation with audio/video and negotiated datachannel should work properly promise_test: Unhandled rejection with value: object \"TypeError: Cannot read property 'getUserMedia' of undefined.\""
      ],
      "reason": null,
      "status": "fail",
      "subtests": [
        {
          "name": "createDataChannel with no argument should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with closed connection should throw InvalidStateError",
          "status": "pass"
        },
        {
          "name": "createDataChannel attribute default values",
          "status": "pass"
        },
        {
          "name": "createDataChannel with provided parameters should initialize attributes to provided values",
          "status": "pass"
        },
        {
          "name": "createDataChannel with label \"foo\" should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with label null should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with label undefined should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with label lone surrogate should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with ordered false should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with ordered null/undefined should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with maxPacketLifeTime 0 should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with maxRetransmits 0 should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with both maxPacketLifeTime and maxRetransmits undefined should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with both maxPacketLifeTime and maxRetransmits should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with protocol \"foo\" should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with protocol null should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with protocol undefined should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with protocol lone surrogate should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 0 and negotiated not set should succeed, but not set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 1 and negotiated not set should succeed, but not set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65534 and negotiated not set should succeed, but not set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65535 and negotiated not set should succeed, but not set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 0 and negotiated true should succeed, and set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 1 and negotiated true should succeed, and set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65534 and negotiated true should succeed, and set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id -1 and negotiated not set should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65536 and negotiated not set should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id -1 should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65535 should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65536 should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with too long label should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with too long label (2 byte unicode) should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with same label used twice should not throw",
          "status": "pass"
        },
        {
          "name": "createDataChannel with negotiated true and id should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with too long protocol should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with too long protocol (2 byte unicode) should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with maximum length label and protocol should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with negotiated false should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with negotiated false and id 42 should ignore the id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with negotiated true and id not defined should throw TypeError",
          "status": "pass"
        },
        {
          "name": "Channels created (after SCTP connected) should have id assigned",
          "status": "pass"
        },
        {
          "name": "Reusing a data channel id that is in use should throw OperationError",
          "status": "pass"
        },
        {
          "name": "Reusing a data channel id that is in use (after setRemoteDescription) should throw OperationError",
          "status": "pass"
        },
        {
          "name": "Reusing a data channel id that is in use (after setRemoteDescription, negotiated via DCEP) should throw OperationError",
          "status": "pass"
        },
        {
          "name": "New datachannel should be in the connecting state after creation (after connection establishment)",
          "status": "pass"
        },
        {
          "name": "addTrack, then creating datachannel, should negotiate properly",
          "status": "fail"
        },
        {
          "name": "addTrack, then creating datachannel, should negotiate properly when max-bundle is used",
          "status": "fail"
        },
        {
          "name": "Renegotiation with audio/video and datachannel should work properly",
          "status": "fail"
        },
        {
          "name": "New negotiated datachannel should be in the connecting state after creation (after connection establishment)",
          "status": "pass"
        },
        {
          "name": "addTrack, then creating negotiated datachannel, should negotiate properly",
          "status": "fail"
        },
        {
          "name": "addTrack, then creating negotiated datachannel, should negotiate properly when max-bundle is used",
          "status": "fail"
        },
        {
          "name": "Renegotiation with audio/video and negotiated datachannel should work properly",
          "status": "fail"
        }
      ],
      "subtests_passed": 46,
      "subtests_total": 52,
      "engine": "nova",
      "test": "webrtc/RTCPeerConnection-createDataChannel.html",
      "exit_code": 0,
      "command": [
        "C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe",
        "testharness-one",
        "webrtc/RTCPeerConnection-createDataChannel.html",
        "--test-path",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCPeerConnection-createDataChannel.html",
        "--tests-root",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests",
        "--engine",
        "nova",
        "--drive-deadline",
        "5"
      ]
    }
  ],
  "matched_cases": [
    {
      "engine": "boa",
      "test": "webrtc/RTCPeerConnection-constructor.html",
      "before_cases": 22,
      "after_cases": 22,
      "missing": [],
      "added": [],
      "improved": 22,
      "regressed": 0
    },
    {
      "engine": "boa",
      "test": "webrtc/RTCIceCandidate-constructor.html",
      "before_cases": 21,
      "after_cases": 21,
      "missing": [],
      "added": [],
      "improved": 21,
      "regressed": 0
    },
    {
      "engine": "boa",
      "test": "webrtc/RTCPeerConnection-createDataChannel.html",
      "before_cases": 52,
      "after_cases": 52,
      "missing": [],
      "added": [],
      "improved": 46,
      "regressed": 0
    },
    {
      "engine": "nova",
      "test": "webrtc/RTCPeerConnection-constructor.html",
      "before_cases": 22,
      "after_cases": 22,
      "missing": [],
      "added": [],
      "improved": 22,
      "regressed": 0
    },
    {
      "engine": "nova",
      "test": "webrtc/RTCIceCandidate-constructor.html",
      "before_cases": 21,
      "after_cases": 21,
      "missing": [],
      "added": [],
      "improved": 21,
      "regressed": 0
    },
    {
      "engine": "nova",
      "test": "webrtc/RTCPeerConnection-createDataChannel.html",
      "before_cases": 52,
      "after_cases": 52,
      "missing": [],
      "added": [],
      "improved": 46,
      "regressed": 0
    }
  ],
  "source_sha256": {
    "tests/wpt/tests/resources/testharness.js": "d2399236c2a09c429804ff2299ad6629e17e2b53f17a74dd341e936adb11ae3e",
    "tests/wpt/tests/webrtc/RTCPeerConnection-helper.js": "9709abd9c5a024c8bdbedd1cfab8a8f2c47b9358a987905cedc2a770ded69d2d",
    "tests/wpt/tests/webrtc/RTCPeerConnection-constructor.html": "31cf6a8f9f9d9c36d4a00a6ca1c224b493b6ee39b848dc4e380ce5b96988d074",
    "tests/wpt/tests/webrtc/RTCIceCandidate-constructor.html": "9bab537b6f3618bda3410d37bba9d63bbdec6116ebe9958759e67fb9fe7d5449",
    "tests/wpt/tests/webrtc/RTCPeerConnection-createDataChannel.html": "0356665f81d466804a4fd06d73075cc03767a5b4944e3f7d62e1da002c49c71f"
  },
  "scope": "Same three unmodified source files at unvaried URLs on both engines. Native host route only; not full RTC, manifest coverage or headed interoperability."
}
```

## 2026-11-01: receiving qualification - check

```json
{
  "source_revision": "74ddda90ccc0af8f2a9760dedb91102e60c2c1d0",
  "recorded_at_utc": "2026-11-01T23:04:39.019921+00:00",
  "command": [
    "cargo",
    "check",
    "-p",
    "ortet",
    "--features",
    "scripted-nova",
    "--all-targets",
    "--locked",
    "--offline",
    "-j",
    "1"
  ],
  "exit_code": 0,
  "stdout": [],
  "stderr": [
    "    Checking genet-webrtc v0.2.0 (C:\\Users\\mark_\\Code\\repos\\genet\\components\\genet-webrtc)",
    "    Checking ortet v0.2.0 (C:\\Users\\mark_\\Code\\repos\\genet\\ports\\ortet)",
    "warning: unused import: `SessionClick`",
    "  --> ports\\ortet\\src\\shell.rs:19:42",
    "   |",
    "19 |     DocumentSession, SessionButtonState, SessionClick, SessionCursor, SessionEffect, SessionEngine,",
    "   |                                          ^^^^^^^^^^^^",
    "   |",
    "   = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default",
    "",
    "warning: method `navigate` is never used",
    "   --> ports\\ortet\\src\\shell.rs:754:8",
    "    |",
    "462 | impl Ortet {",
    "    | ---------- method in this implementation",
    "...",
    "754 |     fn navigate(&mut self, target: &str) {",
    "    |        ^^^^^^^^",
    "    |",
    "    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default",
    "",
    "warning: `ortet` (lib) generated 2 warnings (run `cargo fix --lib -p ortet` to apply 1 suggestion)",
    "warning: `ortet` (lib test) generated 1 warning (1 duplicate)",
    "    Finished `dev` profile [unoptimized] target(s) in 1m 01s"
  ]
}
```

## 2026-11-01: receiving qualification - clippy

```json
{
  "source_revision": "74ddda90ccc0af8f2a9760dedb91102e60c2c1d0",
  "recorded_at_utc": "2026-11-01T23:05:37.400847+00:00",
  "command": [
    "cargo",
    "clippy",
    "-p",
    "genet-webrtc",
    "--lib",
    "--no-deps",
    "--locked",
    "--offline",
    "-j",
    "1",
    "--",
    "-D",
    "warnings"
  ],
  "exit_code": 101,
  "stdout": [],
  "stderr": [
    "error: this `if` statement can be collapsed",
    "   --> components\\genet-webrtc\\src\\lib.rs:261:13",
    "    |",
    "261 | /             if let Some(channel) = self.endpoint.core_mut().data_channel(id) {",
    "262 | |                 if let Some(meta) = self.channels.get_mut(&id) {",
    "263 | |                     meta[\"id\"] = json!(channel.stream_id());",
    "264 | |                     meta[\"readyState\"] = json!(channel.ready_state().to_string());",
    "265 | |                 }",
    "266 | |             }",
    "    | |_____________^",
    "    |",
    "    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#collapsible_if",
    "    = note: `-D clippy::collapsible-if` implied by `-D warnings`",
    "    = help: to override `-D warnings` add `#[allow(clippy::collapsible_if)]`",
    "help: collapse nested if block",
    "    |",
    "261 ~             if let Some(channel) = self.endpoint.core_mut().data_channel(id)",
    "262 ~                 && let Some(meta) = self.channels.get_mut(&id) {",
    "263 |                     meta[\"id\"] = json!(channel.stream_id());",
    "264 |                     meta[\"readyState\"] = json!(channel.ready_state().to_string());",
    "265 ~                 }",
    "    |",
    "",
    "error: could not compile `genet-webrtc` (lib) due to 2 previous errors"
  ]
}
```

## 2026-11-01: final receiving qualification - clippy

```json
{
  "source_revision": "835dca0f37cf7efbd6d791f94752a2e40733c2fa",
  "recorded_at_utc": "2026-11-01T23:08:55.494767+00:00",
  "command": [
    "cargo",
    "clippy",
    "-p",
    "genet-webrtc",
    "--lib",
    "--no-deps",
    "--locked",
    "--offline",
    "-j",
    "1",
    "--",
    "-D",
    "warnings"
  ],
  "exit_code": 0,
  "stdout": [],
  "stderr": [
    "22 +         .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))",
    "   |",
    "",
    "warning: unused import: `crate::OwnerResolvedCx`",
    " --> components\\script-runtime-api\\dom\\xpath_eval.rs:9:5",
    "  |",
    "9 | use crate::OwnerResolvedCx as _;",
    "  |     ^^^^^^^^^^^^^^^^^^^^^^",
    "  |",
    "  = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default",
    "",
    "warning: unused import: `crate::OwnerResolvedCx`",
    "  --> components\\script-runtime-api\\dom\\shadow.rs:14:5",
    "   |",
    "14 | use crate::OwnerResolvedCx as _;",
    "   |     ^^^^^^^^^^^^^^^^^^^^^^",
    "",
    "warning: unused import: `crate::OwnerResolvedCx`",
    "  --> components\\script-runtime-api\\dom\\markup_insertion.rs:37:5",
    "   |",
    "37 | use crate::OwnerResolvedCx as _;",
    "   |     ^^^^^^^^^^^^^^^^^^^^^^",
    "",
    "warning: `genet-runtime` (lib) generated 4 warnings (run `cargo clippy --fix --lib -p genet-runtime -- --no-deps -D warnings` to apply 1 suggestion)",
    "    Checking genet-webrtc v0.2.0 (C:\\Users\\mark_\\Code\\repos\\genet\\components\\genet-webrtc)",
    "    Finished `dev` profile [unoptimized] target(s) in 3.57s"
  ]
}
```

## 2026-11-01: final receiving qualification - test

```json
{
  "source_revision": "835dca0f37cf7efbd6d791f94752a2e40733c2fa",
  "recorded_at_utc": "2026-11-01T23:09:25.434209+00:00",
  "command": [
    "cargo",
    "test",
    "-p",
    "genet-webrtc",
    "--test",
    "browser",
    "--locked",
    "--offline",
    "-j",
    "1"
  ],
  "exit_code": 0,
  "stdout": [
    "",
    "running 14 tests",
    "test vano::policy_and_budgets ... ok",
    "test vano::rollback ... ok",
    "test vano::strings_and_utf8_limits ... ok",
    "test vano::transfer ... ok",
    "test vano::navigation_revokes_old_handles ... ok",
    "test vano::document_teardown ... ok",
    "test vano::constructors ... ok",
    "test boa::rollback ... ok",
    "test boa::policy_and_budgets ... ok",
    "test boa::strings_and_utf8_limits ... ok",
    "test boa::transfer ... ok",
    "test boa::document_teardown ... ok",
    "test boa::navigation_revokes_old_handles ... ok",
    "test boa::constructors ... ok",
    "",
    "test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 13.08s",
    ""
  ],
  "stderr": [
    "   |",
    "",
    "warning: unused import: `crate::OwnerResolvedCx`",
    " --> components\\script-runtime-api\\dom\\xpath_eval.rs:9:5",
    "  |",
    "9 | use crate::OwnerResolvedCx as _;",
    "  |     ^^^^^^^^^^^^^^^^^^^^^^",
    "  |",
    "  = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default",
    "",
    "warning: unused import: `crate::OwnerResolvedCx`",
    "  --> components\\script-runtime-api\\dom\\shadow.rs:14:5",
    "   |",
    "14 | use crate::OwnerResolvedCx as _;",
    "   |     ^^^^^^^^^^^^^^^^^^^^^^",
    "",
    "warning: unused import: `crate::OwnerResolvedCx`",
    "  --> components\\script-runtime-api\\dom\\markup_insertion.rs:37:5",
    "   |",
    "37 | use crate::OwnerResolvedCx as _;",
    "   |     ^^^^^^^^^^^^^^^^^^^^^^",
    "",
    "warning: `genet-runtime` (lib) generated 4 warnings (run `cargo fix --lib -p genet-runtime` to apply 1 suggestion)",
    "   Compiling genet-webrtc v0.2.0 (C:\\Users\\mark_\\Code\\repos\\genet\\components\\genet-webrtc)",
    "    Finished `test` profile [unoptimized] target(s) in 15.80s",
    "     Running tests\\browser.rs (C:\\t\\cargo-targets\\genet\\build\\debug\\deps\\browser-9acfd357843eb5f3.exe)"
  ]
}
```

## 2026-11-01: final receiving qualification - build

```json
{
  "source_revision": "835dca0f37cf7efbd6d791f94752a2e40733c2fa",
  "recorded_at_utc": "2026-11-01T23:13:33.608218+00:00",
  "command": [
    "cargo",
    "build",
    "-p",
    "genet-wpt",
    "--no-default-features",
    "--features",
    "webrtc",
    "--locked",
    "--offline",
    "-j",
    "1"
  ],
  "exit_code": 0,
  "stdout": [],
  "stderr": [
    "    |        ^^^^^^^^^^^^^^^^^^^^^^^^^^",
    "    |",
    "    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default",
    "",
    "warning: method `assemble` is never used",
    "   --> ports\\genet-wpt\\src\\test262.rs:178:12",
    "    |",
    "162 | impl Harness {",
    "    | ------------ method in this implementation",
    "...",
    "178 |     pub fn assemble(&self, test_src: &str, meta: &Test262Meta, strict: bool) -> io::Result<String> {",
    "    |            ^^^^^^^^",
    "",
    "warning: variant `Unsupported` is never constructed",
    "   --> ports\\genet-wpt\\src\\test262.rs:270:5",
    "    |",
    "263 | pub enum Observation {",
    "    |          ----------- variant in this enum",
    "...",
    "270 |     Unsupported,",
    "    |     ^^^^^^^^^^^",
    "    |",
    "    = note: `Observation` has derived impls for the traits `Debug` and `Clone`, but these are intentionally ignored during dead code analysis",
    "",
    "warning: `genet-wpt` (bin \"genet-wpt\") generated 3 warnings",
    "    Finished `dev` profile [unoptimized] target(s) in 4m 07s"
  ]
}
```

## 2026-11-01: final matched selected WPT - after RTC

```json
{
  "schema": 1,
  "phase": "after-rtc",
  "final": true,
  "source_revision": "835dca0f37cf7efbd6d791f94752a2e40733c2fa",
  "binary_sha256": "246f0e49876fd7669bba40c5dc2c5663e3c82d0bc7b27f780066893f5dff97b1",
  "outcomes": [
    {
      "detail": null,
      "failures": [],
      "reason": null,
      "status": "pass",
      "subtests": [
        {
          "name": "RTCPeerConnection.length",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection()",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection(null)",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection(undefined)",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({})",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: null })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: undefined })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: [] })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: [null] })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: [undefined] })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ iceCandidatePoolSize: toNumberThrows })",
          "status": "pass"
        },
        {
          "name": "localDescription initial value",
          "status": "pass"
        },
        {
          "name": "currentLocalDescription initial value",
          "status": "pass"
        },
        {
          "name": "pendingLocalDescription initial value",
          "status": "pass"
        },
        {
          "name": "remoteDescription initial value",
          "status": "pass"
        },
        {
          "name": "currentRemoteDescription initial value",
          "status": "pass"
        },
        {
          "name": "pendingRemoteDescription initial value",
          "status": "pass"
        },
        {
          "name": "signalingState initial value",
          "status": "pass"
        },
        {
          "name": "iceGatheringState initial value",
          "status": "pass"
        },
        {
          "name": "iceConnectionState initial value",
          "status": "pass"
        },
        {
          "name": "connectionState initial value",
          "status": "pass"
        },
        {
          "name": "canTrickleIceCandidates initial value",
          "status": "pass"
        }
      ],
      "subtests_passed": 22,
      "subtests_total": 22,
      "engine": "boa",
      "test": "webrtc/RTCPeerConnection-constructor.html",
      "exit_code": 0,
      "command": [
        "C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe",
        "testharness-one",
        "webrtc/RTCPeerConnection-constructor.html",
        "--test-path",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCPeerConnection-constructor.html",
        "--tests-root",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests",
        "--engine",
        "boa",
        "--drive-deadline",
        "5"
      ]
    },
    {
      "detail": null,
      "failures": [],
      "reason": null,
      "status": "pass",
      "subtests": [
        {
          "name": "new RTCIceCandidate()",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({})",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with manually filled default values",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ sdpMid: null, sdpMLineIndex: null })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ candidate: '' })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ candidate: null })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with valid candidate string only",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ sdpMid: 'audio' })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ sdpMLineIndex: 0 })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ sdpMid: 'audio', sdpMLineIndex: 0 })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ candidate: '', sdpMid: 'audio' }",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ candidate: null, sdpMLineIndex: 0 }",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with valid candidate string and sdpMid",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with invalid candidate string and sdpMid",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for most fields",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for most fields, tcp candidate",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for all fields",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for all fields, tcp candidate",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ relayProtocol, url }) cloned vs signaled",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with invalid sdpMid",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with invalid sdpMLineIndex",
          "status": "pass"
        }
      ],
      "subtests_passed": 21,
      "subtests_total": 21,
      "engine": "boa",
      "test": "webrtc/RTCIceCandidate-constructor.html",
      "exit_code": 0,
      "command": [
        "C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe",
        "testharness-one",
        "webrtc/RTCIceCandidate-constructor.html",
        "--test-path",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCIceCandidate-constructor.html",
        "--tests-root",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests",
        "--engine",
        "boa",
        "--drive-deadline",
        "5"
      ]
    },
    {
      "detail": null,
      "failures": [
        "        [1] addTrack, then creating datachannel, should negotiate properly promise_test: Unhandled rejection with value: object \"TypeError: cannot convert 'null' or 'undefined' to object\"",
        "        [1] addTrack, then creating datachannel, should negotiate properly when max-bundle is used promise_test: Unhandled rejection with value: object \"TypeError: cannot convert 'null' or 'undefined' to object\"",
        "        [1] Renegotiation with audio/video and datachannel should work properly promise_test: Unhandled rejection with value: object \"TypeError: cannot convert 'null' or 'undefined' to object\"",
        "        [1] addTrack, then creating negotiated datachannel, should negotiate properly promise_test: Unhandled rejection with value: object \"TypeError: cannot convert 'null' or 'undefined' to object\"",
        "        [1] addTrack, then creating negotiated datachannel, should negotiate properly when max-bundle is used promise_test: Unhandled rejection with value: object \"TypeError: cannot convert 'null' or 'undefined' to object\"",
        "        [1] Renegotiation with audio/video and negotiated datachannel should work properly promise_test: Unhandled rejection with value: object \"TypeError: cannot convert 'null' or 'undefined' to object\""
      ],
      "reason": null,
      "status": "fail",
      "subtests": [
        {
          "name": "createDataChannel with no argument should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with closed connection should throw InvalidStateError",
          "status": "pass"
        },
        {
          "name": "createDataChannel attribute default values",
          "status": "pass"
        },
        {
          "name": "createDataChannel with provided parameters should initialize attributes to provided values",
          "status": "pass"
        },
        {
          "name": "createDataChannel with label \"foo\" should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with label null should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with label undefined should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with label lone surrogate should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with ordered false should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with ordered null/undefined should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with maxPacketLifeTime 0 should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with maxRetransmits 0 should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with both maxPacketLifeTime and maxRetransmits undefined should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with both maxPacketLifeTime and maxRetransmits should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with protocol \"foo\" should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with protocol null should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with protocol undefined should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with protocol lone surrogate should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 0 and negotiated not set should succeed, but not set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 1 and negotiated not set should succeed, but not set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65534 and negotiated not set should succeed, but not set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65535 and negotiated not set should succeed, but not set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 0 and negotiated true should succeed, and set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 1 and negotiated true should succeed, and set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65534 and negotiated true should succeed, and set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id -1 and negotiated not set should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65536 and negotiated not set should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id -1 should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65535 should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65536 should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with too long label should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with too long label (2 byte unicode) should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with same label used twice should not throw",
          "status": "pass"
        },
        {
          "name": "createDataChannel with negotiated true and id should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with too long protocol should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with too long protocol (2 byte unicode) should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with maximum length label and protocol should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with negotiated false should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with negotiated false and id 42 should ignore the id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with negotiated true and id not defined should throw TypeError",
          "status": "pass"
        },
        {
          "name": "Channels created (after SCTP connected) should have id assigned",
          "status": "pass"
        },
        {
          "name": "Reusing a data channel id that is in use should throw OperationError",
          "status": "pass"
        },
        {
          "name": "Reusing a data channel id that is in use (after setRemoteDescription) should throw OperationError",
          "status": "pass"
        },
        {
          "name": "Reusing a data channel id that is in use (after setRemoteDescription, negotiated via DCEP) should throw OperationError",
          "status": "pass"
        },
        {
          "name": "New datachannel should be in the connecting state after creation (after connection establishment)",
          "status": "pass"
        },
        {
          "name": "addTrack, then creating datachannel, should negotiate properly",
          "status": "fail"
        },
        {
          "name": "addTrack, then creating datachannel, should negotiate properly when max-bundle is used",
          "status": "fail"
        },
        {
          "name": "Renegotiation with audio/video and datachannel should work properly",
          "status": "fail"
        },
        {
          "name": "New negotiated datachannel should be in the connecting state after creation (after connection establishment)",
          "status": "pass"
        },
        {
          "name": "addTrack, then creating negotiated datachannel, should negotiate properly",
          "status": "fail"
        },
        {
          "name": "addTrack, then creating negotiated datachannel, should negotiate properly when max-bundle is used",
          "status": "fail"
        },
        {
          "name": "Renegotiation with audio/video and negotiated datachannel should work properly",
          "status": "fail"
        }
      ],
      "subtests_passed": 46,
      "subtests_total": 52,
      "engine": "boa",
      "test": "webrtc/RTCPeerConnection-createDataChannel.html",
      "exit_code": 0,
      "command": [
        "C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe",
        "testharness-one",
        "webrtc/RTCPeerConnection-createDataChannel.html",
        "--test-path",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCPeerConnection-createDataChannel.html",
        "--tests-root",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests",
        "--engine",
        "boa",
        "--drive-deadline",
        "5"
      ]
    },
    {
      "detail": null,
      "failures": [],
      "reason": null,
      "status": "pass",
      "subtests": [
        {
          "name": "RTCPeerConnection.length",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection()",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection(null)",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection(undefined)",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({})",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: null })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: undefined })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: [] })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: [null] })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: [undefined] })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ iceCandidatePoolSize: toNumberThrows })",
          "status": "pass"
        },
        {
          "name": "localDescription initial value",
          "status": "pass"
        },
        {
          "name": "currentLocalDescription initial value",
          "status": "pass"
        },
        {
          "name": "pendingLocalDescription initial value",
          "status": "pass"
        },
        {
          "name": "remoteDescription initial value",
          "status": "pass"
        },
        {
          "name": "currentRemoteDescription initial value",
          "status": "pass"
        },
        {
          "name": "pendingRemoteDescription initial value",
          "status": "pass"
        },
        {
          "name": "signalingState initial value",
          "status": "pass"
        },
        {
          "name": "iceGatheringState initial value",
          "status": "pass"
        },
        {
          "name": "iceConnectionState initial value",
          "status": "pass"
        },
        {
          "name": "connectionState initial value",
          "status": "pass"
        },
        {
          "name": "canTrickleIceCandidates initial value",
          "status": "pass"
        }
      ],
      "subtests_passed": 22,
      "subtests_total": 22,
      "engine": "nova",
      "test": "webrtc/RTCPeerConnection-constructor.html",
      "exit_code": 0,
      "command": [
        "C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe",
        "testharness-one",
        "webrtc/RTCPeerConnection-constructor.html",
        "--test-path",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCPeerConnection-constructor.html",
        "--tests-root",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests",
        "--engine",
        "nova",
        "--drive-deadline",
        "5"
      ]
    },
    {
      "detail": null,
      "failures": [],
      "reason": null,
      "status": "pass",
      "subtests": [
        {
          "name": "new RTCIceCandidate()",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({})",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with manually filled default values",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ sdpMid: null, sdpMLineIndex: null })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ candidate: '' })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ candidate: null })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with valid candidate string only",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ sdpMid: 'audio' })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ sdpMLineIndex: 0 })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ sdpMid: 'audio', sdpMLineIndex: 0 })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ candidate: '', sdpMid: 'audio' }",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ candidate: null, sdpMLineIndex: 0 }",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with valid candidate string and sdpMid",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with invalid candidate string and sdpMid",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for most fields",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for most fields, tcp candidate",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for all fields",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for all fields, tcp candidate",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ relayProtocol, url }) cloned vs signaled",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with invalid sdpMid",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with invalid sdpMLineIndex",
          "status": "pass"
        }
      ],
      "subtests_passed": 21,
      "subtests_total": 21,
      "engine": "nova",
      "test": "webrtc/RTCIceCandidate-constructor.html",
      "exit_code": 0,
      "command": [
        "C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe",
        "testharness-one",
        "webrtc/RTCIceCandidate-constructor.html",
        "--test-path",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCIceCandidate-constructor.html",
        "--tests-root",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests",
        "--engine",
        "nova",
        "--drive-deadline",
        "5"
      ]
    },
    {
      "detail": null,
      "failures": [
        "        [1] addTrack, then creating datachannel, should negotiate properly promise_test: Unhandled rejection with value: object \"TypeError: Cannot read property 'getUserMedia' of undefined.\"",
        "        [1] addTrack, then creating datachannel, should negotiate properly when max-bundle is used promise_test: Unhandled rejection with value: object \"TypeError: Cannot read property 'getUserMedia' of undefined.\"",
        "        [1] Renegotiation with audio/video and datachannel should work properly promise_test: Unhandled rejection with value: object \"TypeError: Cannot read property 'getUserMedia' of undefined.\"",
        "        [1] addTrack, then creating negotiated datachannel, should negotiate properly promise_test: Unhandled rejection with value: object \"TypeError: Cannot read property 'getUserMedia' of undefined.\"",
        "        [1] addTrack, then creating negotiated datachannel, should negotiate properly when max-bundle is used promise_test: Unhandled rejection with value: object \"TypeError: Cannot read property 'getUserMedia' of undefined.\"",
        "        [1] Renegotiation with audio/video and negotiated datachannel should work properly promise_test: Unhandled rejection with value: object \"TypeError: Cannot read property 'getUserMedia' of undefined.\""
      ],
      "reason": null,
      "status": "fail",
      "subtests": [
        {
          "name": "createDataChannel with no argument should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with closed connection should throw InvalidStateError",
          "status": "pass"
        },
        {
          "name": "createDataChannel attribute default values",
          "status": "pass"
        },
        {
          "name": "createDataChannel with provided parameters should initialize attributes to provided values",
          "status": "pass"
        },
        {
          "name": "createDataChannel with label \"foo\" should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with label null should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with label undefined should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with label lone surrogate should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with ordered false should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with ordered null/undefined should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with maxPacketLifeTime 0 should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with maxRetransmits 0 should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with both maxPacketLifeTime and maxRetransmits undefined should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with both maxPacketLifeTime and maxRetransmits should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with protocol \"foo\" should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with protocol null should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with protocol undefined should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with protocol lone surrogate should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 0 and negotiated not set should succeed, but not set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 1 and negotiated not set should succeed, but not set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65534 and negotiated not set should succeed, but not set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65535 and negotiated not set should succeed, but not set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 0 and negotiated true should succeed, and set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 1 and negotiated true should succeed, and set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65534 and negotiated true should succeed, and set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id -1 and negotiated not set should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65536 and negotiated not set should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id -1 should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65535 should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65536 should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with too long label should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with too long label (2 byte unicode) should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with same label used twice should not throw",
          "status": "pass"
        },
        {
          "name": "createDataChannel with negotiated true and id should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with too long protocol should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with too long protocol (2 byte unicode) should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with maximum length label and protocol should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with negotiated false should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with negotiated false and id 42 should ignore the id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with negotiated true and id not defined should throw TypeError",
          "status": "pass"
        },
        {
          "name": "Channels created (after SCTP connected) should have id assigned",
          "status": "pass"
        },
        {
          "name": "Reusing a data channel id that is in use should throw OperationError",
          "status": "pass"
        },
        {
          "name": "Reusing a data channel id that is in use (after setRemoteDescription) should throw OperationError",
          "status": "pass"
        },
        {
          "name": "Reusing a data channel id that is in use (after setRemoteDescription, negotiated via DCEP) should throw OperationError",
          "status": "pass"
        },
        {
          "name": "New datachannel should be in the connecting state after creation (after connection establishment)",
          "status": "pass"
        },
        {
          "name": "addTrack, then creating datachannel, should negotiate properly",
          "status": "fail"
        },
        {
          "name": "addTrack, then creating datachannel, should negotiate properly when max-bundle is used",
          "status": "fail"
        },
        {
          "name": "Renegotiation with audio/video and datachannel should work properly",
          "status": "fail"
        },
        {
          "name": "New negotiated datachannel should be in the connecting state after creation (after connection establishment)",
          "status": "pass"
        },
        {
          "name": "addTrack, then creating negotiated datachannel, should negotiate properly",
          "status": "fail"
        },
        {
          "name": "addTrack, then creating negotiated datachannel, should negotiate properly when max-bundle is used",
          "status": "fail"
        },
        {
          "name": "Renegotiation with audio/video and negotiated datachannel should work properly",
          "status": "fail"
        }
      ],
      "subtests_passed": 46,
      "subtests_total": 52,
      "engine": "nova",
      "test": "webrtc/RTCPeerConnection-createDataChannel.html",
      "exit_code": 0,
      "command": [
        "C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe",
        "testharness-one",
        "webrtc/RTCPeerConnection-createDataChannel.html",
        "--test-path",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCPeerConnection-createDataChannel.html",
        "--tests-root",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests",
        "--engine",
        "nova",
        "--drive-deadline",
        "5"
      ]
    }
  ],
  "matched_cases": [
    {
      "engine": "boa",
      "test": "webrtc/RTCPeerConnection-constructor.html",
      "before_cases": 22,
      "after_cases": 22,
      "missing": [],
      "added": [],
      "improved": 22,
      "regressed": 0
    },
    {
      "engine": "boa",
      "test": "webrtc/RTCIceCandidate-constructor.html",
      "before_cases": 21,
      "after_cases": 21,
      "missing": [],
      "added": [],
      "improved": 21,
      "regressed": 0
    },
    {
      "engine": "boa",
      "test": "webrtc/RTCPeerConnection-createDataChannel.html",
      "before_cases": 52,
      "after_cases": 52,
      "missing": [],
      "added": [],
      "improved": 46,
      "regressed": 0
    },
    {
      "engine": "nova",
      "test": "webrtc/RTCPeerConnection-constructor.html",
      "before_cases": 22,
      "after_cases": 22,
      "missing": [],
      "added": [],
      "improved": 22,
      "regressed": 0
    },
    {
      "engine": "nova",
      "test": "webrtc/RTCIceCandidate-constructor.html",
      "before_cases": 21,
      "after_cases": 21,
      "missing": [],
      "added": [],
      "improved": 21,
      "regressed": 0
    },
    {
      "engine": "nova",
      "test": "webrtc/RTCPeerConnection-createDataChannel.html",
      "before_cases": 52,
      "after_cases": 52,
      "missing": [],
      "added": [],
      "improved": 46,
      "regressed": 0
    }
  ],
  "source_sha256": {
    "tests/wpt/tests/resources/testharness.js": "d2399236c2a09c429804ff2299ad6629e17e2b53f17a74dd341e936adb11ae3e",
    "tests/wpt/tests/webrtc/RTCPeerConnection-helper.js": "9709abd9c5a024c8bdbedd1cfab8a8f2c47b9358a987905cedc2a770ded69d2d",
    "tests/wpt/tests/webrtc/RTCPeerConnection-constructor.html": "31cf6a8f9f9d9c36d4a00a6ca1c224b493b6ee39b848dc4e380ce5b96988d074",
    "tests/wpt/tests/webrtc/RTCIceCandidate-constructor.html": "9bab537b6f3618bda3410d37bba9d63bbdec6116ebe9958759e67fb9fe7d5449",
    "tests/wpt/tests/webrtc/RTCPeerConnection-createDataChannel.html": "0356665f81d466804a4fd06d73075cc03767a5b4944e3f7d62e1da002c49c71f"
  },
  "scope": "Same three unmodified source files at unvaried URLs on both engines. Native host route only; not full RTC, manifest coverage or headed interoperability."
}
```

## 2026-11-01: final receiving qualification - check

```json
{
  "source_revision": "835dca0f37cf7efbd6d791f94752a2e40733c2fa",
  "recorded_at_utc": "2026-11-01T23:18:13.285112+00:00",
  "command": [
    "cargo",
    "check",
    "-p",
    "ortet",
    "--features",
    "scripted-nova",
    "--all-targets",
    "--locked",
    "--offline",
    "-j",
    "1"
  ],
  "exit_code": 0,
  "stdout": [],
  "stderr": [
    "",
    "warning: `genet-documents` (lib) generated 3 warnings (run `cargo fix --lib -p genet-documents` to apply 2 suggestions)",
    "    Checking genet-webrtc v0.2.0 (C:\\Users\\mark_\\Code\\repos\\genet\\components\\genet-webrtc)",
    "    Checking ortet v0.2.0 (C:\\Users\\mark_\\Code\\repos\\genet\\ports\\ortet)",
    "warning: unused import: `SessionClick`",
    "  --> ports\\ortet\\src\\shell.rs:19:42",
    "   |",
    "19 |     DocumentSession, SessionButtonState, SessionClick, SessionCursor, SessionEffect, SessionEngine,",
    "   |                                          ^^^^^^^^^^^^",
    "   |",
    "   = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default",
    "",
    "warning: method `navigate` is never used",
    "   --> ports\\ortet\\src\\shell.rs:754:8",
    "    |",
    "462 | impl Ortet {",
    "    | ---------- method in this implementation",
    "...",
    "754 |     fn navigate(&mut self, target: &str) {",
    "    |        ^^^^^^^^",
    "    |",
    "    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default",
    "",
    "warning: `ortet` (lib) generated 2 warnings (run `cargo fix --lib -p ortet` to apply 1 suggestion)",
    "warning: `ortet` (lib test) generated 1 warning (1 duplicate)",
    "    Finished `dev` profile [unoptimized] target(s) in 3m 13s"
  ]
}
```

## 2026-11-01: final source, dependency and toolchain audit

Native Windows qualification only. Ordinary runtime dependencies exclude RTC
and Tokio; the separate native adapter selects the exact Murm supplier without
Tokio. Existing targets were reused; no isolated target, home or worktree remains.
The root lock is ignored by repository policy, so its final contents are retained
here for reproduction. Original before/after test inputs stay byte-identical.

```json
{
  "schema": 1,
  "source_revision": "835dca0f37cf7efbd6d791f94752a2e40733c2fa",
  "recorded_at_utc": "2026-11-01T23:21:08.298493+00:00",
  "rustc": "rustc 1.99.1 (58cf8f9c3 2026-10-13)\nbinary: rustc\ncommit-hash: 58cf8f9c35b355199727627d07ae34a4e2846d67\ncommit-date: 2026-10-13\nhost: x86_64-pc-windows-msvc\nrelease: 1.99.1\nLLVM version: 23.1.1\n",
  "target": "C:\\t\\cargo-targets\\genet",
  "build_dir": "C:\\t\\cargo-targets\\genet\\build",
  "profile_dev_debug": "0",
  "graphs": {
    "genet-runtime": {
      "command": [
        "cargo",
        "tree",
        "-p",
        "genet-runtime",
        "-e",
        "normal,build",
        "--prefix",
        "none",
        "--charset",
        "ascii",
        "--locked",
        "--offline"
      ],
      "exit_code": 0,
      "protocol_runtime_rows": [],
      "normal_build_line_count": 237
    },
    "genet-webrtc": {
      "command": [
        "cargo",
        "tree",
        "-p",
        "genet-webrtc",
        "-e",
        "normal,build",
        "--prefix",
        "none",
        "--charset",
        "ascii",
        "--locked",
        "--offline"
      ],
      "exit_code": 0,
      "protocol_runtime_rows": [
        "murm-rtc v0.1.0 (https://github.com/merely-made/murm.git?rev=c58b70421d31ed9f2288dc44152dcb6d719552a9#c58b7042)",
        "rtc v0.21.1 (https://github.com/webrtc-rs/rtc?rev=70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38#70901c3d)",
        "rtc-crypto v0.21.1 (https://github.com/webrtc-rs/rtc?rev=70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38#70901c3d)",
        "rtc-crypto v0.21.1 (https://github.com/webrtc-rs/rtc?rev=70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38#70901c3d) (*)",
        "rtc-datachannel v0.21.1 (https://github.com/webrtc-rs/rtc?rev=70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38#70901c3d)",
        "rtc-dtls v0.21.1 (https://github.com/webrtc-rs/rtc?rev=70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38#70901c3d)",
        "rtc-ice v0.21.1 (https://github.com/webrtc-rs/rtc?rev=70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38#70901c3d)",
        "rtc-interceptor v0.21.1 (https://github.com/webrtc-rs/rtc?rev=70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38#70901c3d)",
        "rtc-mdns v0.21.1 (https://github.com/webrtc-rs/rtc?rev=70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38#70901c3d)",
        "rtc-mdns v0.21.1 (https://github.com/webrtc-rs/rtc?rev=70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38#70901c3d) (*)",
        "rtc-media v0.21.1 (https://github.com/webrtc-rs/rtc?rev=70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38#70901c3d)",
        "rtc-rtcp v0.21.1 (https://github.com/webrtc-rs/rtc?rev=70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38#70901c3d)",
        "rtc-rtcp v0.21.1 (https://github.com/webrtc-rs/rtc?rev=70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38#70901c3d) (*)",
        "rtc-rtp v0.21.1 (https://github.com/webrtc-rs/rtc?rev=70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38#70901c3d)",
        "rtc-rtp v0.21.1 (https://github.com/webrtc-rs/rtc?rev=70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38#70901c3d) (*)",
        "rtc-sctp v0.21.1 (https://github.com/webrtc-rs/rtc?rev=70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38#70901c3d)",
        "rtc-sctp v0.21.1 (https://github.com/webrtc-rs/rtc?rev=70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38#70901c3d) (*)",
        "rtc-sdp v0.21.1 (https://github.com/webrtc-rs/rtc?rev=70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38#70901c3d)",
        "rtc-shared v0.21.1 (https://github.com/webrtc-rs/rtc?rev=70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38#70901c3d)",
        "rtc-shared v0.21.1 (https://github.com/webrtc-rs/rtc?rev=70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38#70901c3d) (*)",
        "rtc-srtp v0.21.1 (https://github.com/webrtc-rs/rtc?rev=70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38#70901c3d)",
        "rtc-stun v0.21.1 (https://github.com/webrtc-rs/rtc?rev=70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38#70901c3d)",
        "rtc-stun v0.21.1 (https://github.com/webrtc-rs/rtc?rev=70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38#70901c3d) (*)",
        "rtc-turn v0.21.1 (https://github.com/webrtc-rs/rtc?rev=70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38#70901c3d)"
      ],
      "normal_build_line_count": 524
    }
  },
  "format": [
    {
      "command": [
        "rustfmt",
        "--check",
        "--edition",
        "2024",
        "components/genet-webrtc/src/lib.rs",
        "components/genet-webrtc/tests/browser.rs"
      ],
      "exit_code": 0
    },
    {
      "command": [
        "rustfmt",
        "--check",
        "--edition",
        "2021",
        "components/script-runtime-api/webrtc.rs"
      ],
      "exit_code": 0
    }
  ],
  "source_sha256": {
    "Cargo.toml": "2d9606a19b2402e39318dd956a9456f356dc1f378d1f8dfeaad73ac9ffb36076",
    "Cargo.lock": "a985187e120dcea861874dd5242939cd1874ee9d2f3e5305f02f14f4768e26c3",
    "components/genet-webrtc/Cargo.toml": "3b8756e5472df27424e840746e066ac2b162ce358ffb1e4d28a0821e16fb3fcd",
    "components/genet-webrtc/src/lib.rs": "0c1e8202ac556e6d09088a441fd0609576e704b40538ca82defc6dc468776f90",
    "components/genet-webrtc/tests/browser.rs": "a21ce47c1efdd3348f7b4448b2cc6aacbe1a9f77df98414f14a20830c2175e07",
    "components/script-runtime-api/Cargo.toml": "8f431144410f1563a04715ead7eeea6a0da293eae3626ac49e333fee453b648d",
    "components/script-runtime-api/lib.rs": "0e73f3e5ef9ba6b333bde5ec736b147bb9de47ab073006124ea8b4b654b75330",
    "components/script-runtime-api/frames.rs": "b6379f495ba981ec86c61281844a8ba90011f21d4bebd4c04f3d61ceff3cb6b5",
    "components/script-runtime-api/webrtc.rs": "6e7ec34cab01fa5f41bb6a2b10da3f3c2e81985025e28003939c68aebbd93486",
    "components/script-runtime-api/webrtc/bootstrap.js": "8410ef0b257b49c787cb9f511e48d496ae4aaab6618fbe1dfb1ab333c289de1b",
    "components/genet-scripted/host.rs": "de53953e8bafea34ed1aedfdc33c38fd71913176bcd6f90980be7688b6b7540f",
    "components/genet-scripted/document.rs": "cf58a1e2c866e38b2d2cf9b8cc79f9f50d46e9f382ac19a30a6e975fa7896c50",
    "ports/genet-wpt/Cargo.toml": "48c13b0ad0f3994c5316bb108f9d151906eb0b09e87c24260db6f430b7b77b91",
    "ports/genet-wpt/src/harness.rs": "c2d4349338280b4d8cde7a1d563a4bcdc0c50cc889a8053c817b5703a1cdfdcf",
    "ports/ortet/Cargo.toml": "3c31c03e13a57358b1cd127c05c3a259df7c0a934e6885ff0f7634dad2efa245",
    "ports/ortet/src/args.rs": "aac2c98948d0a4cb21822912c14e8a01917ee4283e1ce9c840eedd6afb18c049",
    "ports/ortet/src/shell.rs": "6b197e3425a8f30396a9ea38a26504a1ec37adb1052d0e706cb8f9e8aead7244"
  },
  "binary_sha256": {
    "C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe": "246f0e49876fd7669bba40c5dc2c5663e3c82d0bc7b27f780066893f5dff97b1",
    "C:\\t\\cargo-targets\\genet\\build\\debug\\deps\\browser-9acfd357843eb5f3.exe": "8b6caf3a0e8050b7d5a73106c71ad2dcaefed40b30d6fa614977a4d96e523324"
  }
}
```

### Final receiving Cargo.lock

```toml
# This file is automatically @generated by Cargo.
# It is not intended for manual editing.
version = 4

[[package]]
name = "ab_glyph"
version = "0.2.32"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "01c0457472c38ea5bd1c3b5ada5e368271cb550be7a4ca4a0b4634e9913f6cc2"
dependencies = [
 "ab_glyph_rasterizer",
 "owned_ttf_parser",
]

[[package]]
name = "ab_glyph_rasterizer"
version = "0.1.10"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "366ffbaa4442f4684d91e2cd7c5ea7c4ed8add41959a31447066e279e432b618"

[[package]]
name = "accesskit"
version = "0.25.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ad442f58ee04714aaa0ba0a2768c1ea1935b29507bb22ddece8cbbc76db02932"
dependencies = [
 "enumn",
 "serde",
 "uuid",
]

[[package]]
name = "accesskit_atspi_common"
version = "0.21.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "52c182f9c282ac9c5638d876d551d15e5f7d397ec263349a0c6a2b61595dd5e4"
dependencies = [
 "accesskit",
 "accesskit_consumer",
 "atspi-common",
 "phf 0.14.0",
 "serde",
 "zvariant",
]

[[package]]
name = "accesskit_consumer"
version = "0.39.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8a45e2bb300bea6d5f33a82c3e051c3ddc6f41c209750f68a0acbe991a284344"
dependencies = [
 "accesskit",
 "hashbrown 0.17.1",
]

[[package]]
name = "accesskit_macos"
version = "0.27.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5701624bf6a11efb6d4c8e922bf44126ed96250e5faa9a7a0468c979635839a4"
dependencies = [
 "accesskit",
 "accesskit_consumer",
 "hashbrown 0.17.1",
 "objc2 0.5.3",
 "objc2-app-kit 0.2.2",
 "objc2-foundation 0.2.2",
]

[[package]]
name = "accesskit_unix"
version = "0.24.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "202f24df034a7476d07b7f74284de84f6d62aabd858dbe7ee9cad3b7ad6f8f9d"
dependencies = [
 "accesskit",
 "accesskit_atspi_common",
 "async-channel",
 "async-executor",
 "async-task",
 "atspi",
 "futures-lite",
 "futures-util",
 "serde",
 "zbus",
]

[[package]]
name = "accesskit_windows"
version = "0.35.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ce63f35d6bdcf59f26b76b3379063f738e6412cef46999cc772d46aa3de35adb"
dependencies = [
 "accesskit",
 "accesskit_consumer",
 "hashbrown 0.17.1",
 "static_assertions",
 "windows",
 "windows-core",
]

[[package]]
name = "adler2"
version = "2.0.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "320119579fcad9c21884f5c4861d16174d0e06250625266f50fe6898340abefa"

[[package]]
name = "aead"
version = "0.5.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d122413f284cf2d62fb1b7db97e02edb8cda96d769b16e443a4f6195e35662b0"
dependencies = [
 "crypto-common 0.1.7",
 "generic-array",
]

[[package]]
name = "aes"
version = "0.8.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b169f7a6d4742236a0a00c541b845991d0ac43e546831af1249753ab4c3aa3a0"
dependencies = [
 "cfg-if",
 "cipher",
 "cpufeatures 0.2.17",
]

[[package]]
name = "ahash"
version = "0.8.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5a15f179cd60c4584b8a8c596927aadc462e27f2ca70c04e0071964a73ba7a75"
dependencies = [
 "cfg-if",
 "const-random",
 "getrandom 0.3.4",
 "once_cell",
 "version_check",
 "zerocopy",
]

[[package]]
name = "aho-corasick"
version = "1.1.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c982642fa9e8606056828ee9a8505737230110bb1099153c79efe865c59d12ba"
dependencies = [
 "memchr",
]

[[package]]
name = "aligned"
version = "0.4.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ee4508988c62edf04abd8d92897fca0c2995d907ce1dfeaf369dac3716a40685"
dependencies = [
 "as-slice",
]

[[package]]
name = "aligned-vec"
version = "0.6.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "dc890384c8602f339876ded803c97ad529f3842aba97f6392b3dba0dd171769b"
dependencies = [
 "equator",
]

[[package]]
name = "alloc-no-stdlib"
version = "2.0.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cc7bb162ec39d46ab1ca8c77bf72e890535becd1751bb45f64c597edb4c8c6b3"

[[package]]
name = "alloc-no-stdlib"
version = "3.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c2fb6cfd47bf496ff64095c20eaba0c201404ee38714d4142fcfa1dc334fcc7a"

[[package]]
name = "alloc-stdlib"
version = "0.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0b5c1865780388bfa186411ab5f247819487fc4864c6e9c3106611fa347586e1"
dependencies = [
 "alloc-no-stdlib 3.0.0",
]

[[package]]
name = "allocator-api2"
version = "0.2.21"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "683d7910e743518b0e34f1186f92494becacb047c7b6bf616c96772180fef923"

[[package]]
name = "android-activity"
version = "0.6.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0f2a1bb052857d5dd49572219344a7332b31b76405648eabac5bc68978251bcd"
dependencies = [
 "android-properties",
 "bitflags 2.13.2",
 "cc",
 "jni",
 "libc",
 "log",
 "ndk",
 "ndk-context",
 "ndk-sys",
 "num_enum",
 "thiserror 2.0.21",
]

[[package]]
name = "android-properties"
version = "0.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "fc7eb209b1518d6bb87b283c20095f5228ecda460da70b44f0802523dea6da04"

[[package]]
name = "android_system_properties"
version = "0.1.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ae221649c9976a6f6c56ae1facf410f3ddb33cc661c4b7b61020a912d4237fbc"
dependencies = [
 "libc",
]

[[package]]
name = "anyhow"
version = "1.0.104"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "330a5ed07fa54e4702c9d6c4174f74427fc0ef6e214bbd677ae50a5099946470"

[[package]]
name = "app_units"
version = "0.7.8"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "467b60e4ee6761cd6fd4e03ea58acefc8eec0d1b1def995c1b3b783fa7be8a60"
dependencies = [
 "num-traits",
 "serde",
]

[[package]]
name = "ar_archive_writer"
version = "0.5.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "73cd58deff2140a0a8eae87e417bd01db68a33e148aa93d1e8cd837e55e312b6"
dependencies = [
 "object",
]

[[package]]
name = "arbitrary"
version = "1.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3bc62ac97cc33321f50863d514c3bc38a453947a8f9e781137e47c7401020aed"

[[package]]
name = "arboard"
version = "3.7.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "aeb9553e62312c8756a5979254fc6d7da8bbeb5986a1e17037fab932f7037917"
dependencies = [
 "clipboard-win",
 "image",
 "log",
 "objc2 0.6.5",
 "objc2-app-kit 0.3.2",
 "objc2-core-foundation",
 "objc2-core-graphics",
 "objc2-foundation 0.3.2",
 "parking_lot",
 "percent-encoding",
 "windows-sys 0.61.2",
 "x11rb",
]

[[package]]
name = "arg_enum_proc_macro"
version = "0.3.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0ae92a5119aa49cdbcf6b9f893fe4e1d98b04ccbf82ee0584ad948a44a734dea"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "arrayref"
version = "0.3.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "76a2e8124351fda1ef8aaaa3bbd7ebbcb486bbcd4225aca0aa0d84bb2db8fecb"

[[package]]
name = "arrayvec"
version = "0.7.8"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d3fb67a6e08acf24fdeccbac2cb6ac4305825bd1f117462e0e6f2f193345ad56"

[[package]]
name = "as-raw-xcb-connection"
version = "1.0.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "175571dd1d178ced59193a6fc02dde1b972eb0bc56c892cde9beeceac5bf0f6b"

[[package]]
name = "as-slice"
version = "0.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "516b6b4f0e40d50dcda9365d53964ec74560ad4284da2e7fc97122cd83174516"
dependencies = [
 "stable_deref_trait",
]

[[package]]
name = "ash"
version = "0.38.0+1.3.281"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0bb44936d800fea8f016d7f2311c6a4f97aebd5dc86f09906139ec848cf3a46f"
dependencies = [
 "libloading",
]

[[package]]
name = "asn1-rs"
version = "0.6.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5493c3bedbacf7fd7382c6346bbd66687d12bbaad3a89a2d2c303ee6cf20b048"
dependencies = [
 "asn1-rs-derive 0.5.1",
 "asn1-rs-impl",
 "displaydoc",
 "nom 7.1.3",
 "num-traits",
 "rusticata-macros",
 "thiserror 1.0.69",
 "time",
]

[[package]]
name = "asn1-rs"
version = "0.7.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b7f43a50ac4fdca5df8e885c21b835997f0a1cdee65494a6847694a98652d9d8"
dependencies = [
 "asn1-rs-derive 0.6.0",
 "asn1-rs-impl",
 "displaydoc",
 "nom 7.1.3",
 "num-traits",
 "rusticata-macros",
 "thiserror 2.0.21",
 "time",
]

[[package]]
name = "asn1-rs-derive"
version = "0.5.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "965c2d33e53cb6b267e148a4cb0760bc01f4904c1cd4bb4002a085bb016d1490"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 2.0.119",
 "synstructure 0.13.2",
]

[[package]]
name = "asn1-rs-derive"
version = "0.6.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3109e49b1e4909e9db6515a30c633684d68cdeaa252f215214cb4fa1a5bfee2c"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 2.0.119",
 "synstructure 0.13.2",
]

[[package]]
name = "asn1-rs-impl"
version = "0.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7b18050c2cd6fe86c3a76584ef5e0baf286d038cda203eb6223df2cc413565f7"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "assert-json-diff"
version = "2.0.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "47e4f2b81832e72834d7518d8487a0396a28cc408186a2e8854c0f98011faf12"
dependencies = [
 "serde",
 "serde_json",
]

[[package]]
name = "async-broadcast"
version = "0.7.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "435a87a52755b8f27fcf321ac4f04b2802e337c8c4872923137471ec39c37532"
dependencies = [
 "event-listener",
 "event-listener-strategy",
 "futures-core",
 "pin-project-lite",
]

[[package]]
name = "async-channel"
version = "2.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "924ed96dd52d1b75e9c1a3e6275715fd320f5f9439fb5a4a11fa51f4221158d2"
dependencies = [
 "concurrent-queue",
 "event-listener-strategy",
 "futures-core",
 "pin-project-lite",
]

[[package]]
name = "async-compression"
version = "0.4.50"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ee19bd99b43e3691acbad4e840420a4881cea6c0b66a208125a824f8fd53f5a1"
dependencies = [
 "compression-codecs",
 "compression-core",
 "pin-project-lite",
 "tokio",
]

[[package]]
name = "async-executor"
version = "1.14.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c96bf972d85afc50bf5ab8fe2d54d1586b4e0b46c97c50a0c9e71e2f7bcd812a"
dependencies = [
 "async-task",
 "concurrent-queue",
 "fastrand",
 "futures-lite",
 "pin-project-lite",
 "slab",
]

[[package]]
name = "async-io"
version = "2.6.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "456b8a8feb6f42d237746d4b3e9a178494627745c3c56c6ea55d92ba50d026fc"
dependencies = [
 "autocfg",
 "cfg-if",
 "concurrent-queue",
 "futures-io",
 "futures-lite",
 "parking",
 "polling",
 "rustix 1.1.5",
 "slab",
 "windows-sys 0.61.2",
]

[[package]]
name = "async-lock"
version = "3.4.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "290f7f2596bd5b78a9fec8088ccd89180d7f9f55b94b0576823bbbdc72ee8311"
dependencies = [
 "event-listener",
 "event-listener-strategy",
 "pin-project-lite",
]

[[package]]
name = "async-process"
version = "2.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "fc50921ec0055cdd8a16de48773bfeec5c972598674347252c0399676be7da75"
dependencies = [
 "async-channel",
 "async-io",
 "async-lock",
 "async-signal",
 "async-task",
 "blocking",
 "cfg-if",
 "event-listener",
 "futures-lite",
 "rustix 1.1.5",
]

[[package]]
name = "async-recursion"
version = "1.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4f8abc12baad266b1c8cec146854c195b5864b4221d4b2ca7296a7ae82d9e451"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 3.0.6",
]

[[package]]
name = "async-signal"
version = "0.2.14"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "52b5aaafa020cf5053a01f2a60e8ff5dccf550f0f77ec54a4e47285ac2bab485"
dependencies = [
 "async-io",
 "async-lock",
 "atomic-waker",
 "cfg-if",
 "futures-core",
 "futures-io",
 "rustix 1.1.5",
 "signal-hook-registry",
 "slab",
 "windows-sys 0.61.2",
]

[[package]]
name = "async-task"
version = "4.7.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8b75356056920673b02621b35afd0f7dda9306d03c79a30f5c56c44cf256e3de"

[[package]]
name = "async-trait"
version = "0.1.92"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "82f6aeea286b8eb4dd3431a1be1b59d290ace00f5bfd8e2a159bc2a05e2c1667"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 3.0.6",
]

[[package]]
name = "atomic-polyfill"
version = "1.0.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8cf2bce30dfe09ef0bfaef228b9d414faaf7e563035494d7fe092dba54b300f4"
dependencies = [
 "critical-section",
]

[[package]]
name = "atomic-waker"
version = "1.1.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1505bd5d3d116872e7271a6d4e16d81d0c8570876c8de68093a09ac269d8aac0"

[[package]]
name = "atomic_refcell"
version = "0.1.14"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "21e4227379beff4205943696e6c3e0cd809bacdf3f0edd6e3dd153e2269571a4"

[[package]]
name = "atspi"
version = "0.29.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c77886257be21c9cd89a4ae7e64860c6f0eefca799bb79127913052bd0eefb3d"
dependencies = [
 "atspi-common",
 "atspi-proxies",
]

[[package]]
name = "atspi-common"
version = "0.13.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "20c5617155740c98003016429ad13fe43ce7a77b007479350a9f8bf95a29f63d"
dependencies = [
 "enumflags2",
 "serde",
 "static_assertions",
 "zbus",
 "zbus-lockstep",
 "zbus-lockstep-macros",
 "zbus_names",
 "zvariant",
]

[[package]]
name = "atspi-proxies"
version = "0.13.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2230e48787ed3eb4088996eab66a32ca20c0b67bbd4fd6cdfe79f04f1f04c9fc"
dependencies = [
 "atspi-common",
 "serde",
 "zbus",
]

[[package]]
name = "autocfg"
version = "1.5.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f2032f911046de80f0a198e0901378627c33f59ea0ac00e363d481118bd70a53"

[[package]]
name = "av-scenechange"
version = "0.14.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0f321d77c20e19b92c39e7471cf986812cbb46659d2af674adc4331ef3f18394"
dependencies = [
 "aligned",
 "anyhow",
 "arg_enum_proc_macro",
 "arrayvec",
 "log",
 "num-rational",
 "num-traits",
 "pastey 0.1.1",
 "rayon",
 "thiserror 2.0.21",
 "v_frame",
 "y4m",
]

[[package]]
name = "av1-grain"
version = "0.2.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8cfddb07216410377231960af4fcab838eaa12e013417781b78bd95ee22077f8"
dependencies = [
 "anyhow",
 "arrayvec",
 "log",
 "nom 8.0.0",
 "num-rational",
 "v_frame",
]

[[package]]
name = "avif-serialize"
version = "0.8.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e7178fe5f7d460b13895ebb9dcb28a3a6216d2df2574a0806cb51b555d297f38"
dependencies = [
 "arrayvec",
]

[[package]]
name = "base64"
version = "0.22.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "72b3254f16251a8381aa12e40e3c4d2f0199f8c6508fbecb9d91f575e0fbb8c6"

[[package]]
name = "base64"
version = "0.23.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ac07cdecf99051d9a5238b80f35af32cdeba5b336e55d957b318b50137e18da5"

[[package]]
name = "bit-set"
version = "0.10.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "09ec2f926cc3060f09db9ebc5b52823d85268d24bb917e472c0c4bea35780a7d"
dependencies = [
 "bit-vec",
]

[[package]]
name = "bit-vec"
version = "0.9.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b71798fca2c1fe1086445a7258a4bc81e6e49dcd24c8d0dd9a1e57395b603f51"
dependencies = [
 "serde",
]

[[package]]
name = "bit_field"
version = "0.10.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1e4b40c7323adcfc0a41c4b88143ed58346ff65a288fc144329c5c45e05d70c6"

[[package]]
name = "bitflags"
version = "1.3.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bef38d45163c2f1dde094a7dfd33ccf595c92905c8f8f4fdc18d06fb1037718a"

[[package]]
name = "bitflags"
version = "2.13.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3ded4057c258ba199e2d26386d3af3780957ecaee6c4ef4041c6b4b8b97c0b06"
dependencies = [
 "serde_core",
]

[[package]]
name = "bitstream-io"
version = "4.10.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7eff00be299a18769011411c9def0d827e8f2d7bf0c3dbf53633147a8867fd1f"
dependencies = [
 "no_std_io2",
]

[[package]]
name = "bitvec"
version = "1.1.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ddcec3d12c579d40898fe0a9a358a803c23e9c52ca3c425707f81c9436211837"
dependencies = [
 "funty",
 "radium",
 "tap",
 "wyz",
]

[[package]]
name = "block-buffer"
version = "0.10.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3078c7629b62d3f0439517fa394996acacc5cbc91c5a20d8c658e77abd503a71"
dependencies = [
 "generic-array",
]

[[package]]
name = "block-buffer"
version = "0.12.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d2f6c7dbe95a6ed67ad9f18e57daf93a2f034c524b99fd2b76d18fdfeb6660aa"
dependencies = [
 "hybrid-array",
]

[[package]]
name = "block2"
version = "0.5.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2c132eebf10f5cad5289222520a4a058514204aed6d791f1cf4fe8088b82d15f"
dependencies = [
 "objc2 0.5.3",
]

[[package]]
name = "block2"
version = "0.6.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cdeb9d870516001442e364c5220d3574d2da8dc765554b4a617230d33fa58ef5"
dependencies = [
 "objc2 0.6.5",
]

[[package]]
name = "blocking"
version = "1.7.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a70e4329df6cb94385eed412ec92375c3cdd8a6e502493d1229b6414e4036dfa"
dependencies = [
 "async-channel",
 "async-task",
 "futures-io",
 "futures-lite",
 "piper",
]

[[package]]
name = "boa_ast"
version = "1.0.0-dev"
source = "git+https://github.com/merely-made/bole.git?rev=ddf4e31b0d6cf86c11ea8907b0d07e4799d62368#ddf4e31b0d6cf86c11ea8907b0d07e4799d62368"
dependencies = [
 "bitflags 2.13.2",
 "boa_interner",
 "boa_macros",
 "boa_string",
 "indexmap",
 "num-bigint 0.5.1",
 "rustc-hash 2.1.3",
 "strum",
]

[[package]]
name = "boa_engine"
version = "1.0.0-dev"
source = "git+https://github.com/merely-made/bole.git?rev=ddf4e31b0d6cf86c11ea8907b0d07e4799d62368#ddf4e31b0d6cf86c11ea8907b0d07e4799d62368"
dependencies = [
 "aligned-vec",
 "arrayvec",
 "async-channel",
 "bitflags 2.13.2",
 "boa_ast",
 "boa_gc",
 "boa_interner",
 "boa_macros",
 "boa_parser",
 "boa_string",
 "bytemuck",
 "cfg-if",
 "cow-utils",
 "dashmap",
 "dynify",
 "fast-float2",
 "float16",
 "futures-concurrency",
 "futures-lite",
 "getrandom 0.4.3",
 "hashbrown 0.17.1",
 "icu_calendar",
 "icu_normalizer",
 "indexmap",
 "intrusive-collections",
 "itertools 0.15.0",
 "num-bigint 0.5.1",
 "num-integer",
 "num-traits",
 "num_enum",
 "oneshot",
 "pastey 0.2.3",
 "portable-atomic",
 "rand 0.10.3",
 "regress 0.12.0",
 "rustc-hash 2.1.3",
 "ryu-js",
 "serde",
 "serde_json",
 "small_btree",
 "static_assertions",
 "tag_ptr",
 "tap",
 "temporal_rs",
 "thin-vec",
 "thiserror 2.0.21",
 "time",
 "timezone_provider",
 "web-time",
 "xsum",
]

[[package]]
name = "boa_gc"
version = "1.0.0-dev"
source = "git+https://github.com/merely-made/bole.git?rev=ddf4e31b0d6cf86c11ea8907b0d07e4799d62368#ddf4e31b0d6cf86c11ea8907b0d07e4799d62368"
dependencies = [
 "arrayvec",
 "boa_macros",
 "boa_string",
 "hashbrown 0.17.1",
 "thin-vec",
]

[[package]]
name = "boa_interner"
version = "1.0.0-dev"
source = "git+https://github.com/merely-made/bole.git?rev=ddf4e31b0d6cf86c11ea8907b0d07e4799d62368#ddf4e31b0d6cf86c11ea8907b0d07e4799d62368"
dependencies = [
 "boa_gc",
 "boa_macros",
 "hashbrown 0.17.1",
 "indexmap",
 "once_cell",
 "phf 0.14.0",
 "rustc-hash 2.1.3",
 "static_assertions",
]

[[package]]
name = "boa_macros"
version = "1.0.0-dev"
source = "git+https://github.com/merely-made/bole.git?rev=ddf4e31b0d6cf86c11ea8907b0d07e4799d62368#ddf4e31b0d6cf86c11ea8907b0d07e4799d62368"
dependencies = [
 "cfg-if",
 "cow-utils",
 "proc-macro2",
 "quote",
 "syn 3.0.6",
 "synstructure 0.14.0",
]

[[package]]
name = "boa_parser"
version = "1.0.0-dev"
source = "git+https://github.com/merely-made/bole.git?rev=ddf4e31b0d6cf86c11ea8907b0d07e4799d62368#ddf4e31b0d6cf86c11ea8907b0d07e4799d62368"
dependencies = [
 "bitflags 2.13.2",
 "boa_ast",
 "boa_interner",
 "boa_macros",
 "fast-float2",
 "icu_properties",
 "num-bigint 0.5.1",
 "num-traits",
 "regress 0.12.0",
 "rustc-hash 2.1.3",
]

[[package]]
name = "boa_string"
version = "1.0.0-dev"
source = "git+https://github.com/merely-made/bole.git?rev=ddf4e31b0d6cf86c11ea8907b0d07e4799d62368#ddf4e31b0d6cf86c11ea8907b0d07e4799d62368"
dependencies = [
 "fast-float2",
 "itoa",
 "pastey 0.2.3",
 "rustc-hash 2.1.3",
 "ryu-js",
 "static_assertions",
]

[[package]]
name = "brotli"
version = "9.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f8b851b75c23ca7873623d612fe49bd1989aeb03d08fb9432187eb253d3d4c6b"
dependencies = [
 "alloc-no-stdlib 3.0.0",
 "alloc-stdlib",
 "brotli-decompressor 6.0.1",
]

[[package]]
name = "brotli-decompressor"
version = "5.0.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3a32acac15fe1967bc3986b2a6347dffc965602354ea6f450ad07e8bfd253583"
dependencies = [
 "alloc-no-stdlib 2.0.4",
]

[[package]]
name = "brotli-decompressor"
version = "6.0.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "941cd9bd4ddab83cb46fa5a2d428f1c857b24ac78cb876cf7beb710840934bd7"
dependencies = [
 "alloc-no-stdlib 3.0.0",
 "alloc-stdlib",
]

[[package]]
name = "buckram"
version = "0.0.3"
dependencies = [
 "genet-flow",
 "genet-taffy",
 "stacker",
]

[[package]]
name = "built"
version = "0.8.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5c0e531d93d39c34eef561e929e8a7f86d77a5af08aac4f6d6e39976c51858e9"

[[package]]
name = "bumpalo"
version = "3.20.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "72f5acc6cb2ba439de613abc23857ec3d78374d8ed5ac84e9d11336e87da8649"

[[package]]
name = "byte-slice-cast"
version = "1.2.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7575182f7272186991736b70173b0ea045398f984bf5ebbb3804736ce1330c9d"

[[package]]
name = "bytecheck"
version = "0.8.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "26333eeac754f0ad8a6bcd0eb0ac012156302e4e16b852b72ee399aea4f12c29"
dependencies = [
 "bytecheck_derive",
 "ptr_meta",
 "rancor",
 "simdutf8",
]

[[package]]
name = "bytecheck_derive"
version = "0.8.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "46d07918caa9eeaaf06b7873925c53a61daac173539b4f7715090745e44e4e69"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 3.0.6",
]

[[package]]
name = "bytemuck"
version = "1.25.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "95832e849adfb21180ccb6826a99da14e5d266ae5c2e668e1602cf234f153797"
dependencies = [
 "bytemuck_derive",
]

[[package]]
name = "bytemuck_derive"
version = "1.12.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6a1f896587b6f2c069c73d2f0913e2d590c3990285cd2f0b6aa02b786b4c679c"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 3.0.6",
]

[[package]]
name = "byteorder"
version = "1.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1fd0f2584146f6f2ef48085050886acf353beff7305ebd1ae69500e27c67f64b"

[[package]]
name = "byteorder-lite"
version = "0.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8f1fe948ff07f4bd06c30984e69f5b4899c516a3ef74f34df92a2df2ab535495"

[[package]]
name = "bytes"
version = "1.12.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "fc652a48c352aef3ea3aed32080501cf3ef6ed5da78602a020c991775b0aff04"

[[package]]
name = "cadency"
version = "0.0.1"
dependencies = [
 "cssparser",
]

[[package]]
name = "calendrical_calculations"
version = "0.2.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5abbd6eeda6885048d357edc66748eea6e0268e3dd11f326fff5bd248d779c26"
dependencies = [
 "core_maths",
 "displaydoc",
]

[[package]]
name = "calloop"
version = "0.13.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b99da2f8558ca23c71f4fd15dc57c906239752dd27ff3c00a1d56b685b7cbfec"
dependencies = [
 "bitflags 2.13.2",
 "log",
 "polling",
 "rustix 0.38.44",
 "slab",
 "thiserror 1.0.69",
]

[[package]]
name = "calloop-wayland-source"
version = "0.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "95a66a987056935f7efce4ab5668920b5d0dac4a7c99991a67395f13702ddd20"
dependencies = [
 "calloop",
 "rustix 0.38.44",
 "wayland-backend",
 "wayland-client",
]

[[package]]
name = "castaway"
version = "0.2.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "dec551ab6e7578819132c713a93c022a05d60159dc86e7a7050223577484c55a"
dependencies = [
 "rustversion",
]

[[package]]
name = "cc"
version = "1.6.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f74872d07caf508b30a21f6836e7d7016a2eaf7d9ff4f48deaa58cd8a0407630"
dependencies = [
 "find-msvc-tools",
 "jobserver",
 "libc",
 "shlex",
]

[[package]]
name = "ccm"
version = "0.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9ae3c82e4355234767756212c570e29833699ab63e6ffd161887314cc5b43847"
dependencies = [
 "aead",
 "cipher",
 "ctr",
 "subtle",
]

[[package]]
name = "cfg-expr"
version = "0.20.10"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9ba9e9ec16c447027685b1f897b720e18e9a8afd00bd7332c483537e38086c9f"
dependencies = [
 "smallvec",
 "target-lexicon",
]

[[package]]
name = "cfg-if"
version = "1.0.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4e7648175b45a9a48536d676f68d918270699102aa8dab5496df06904c914600"

[[package]]
name = "cfg_aliases"
version = "0.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f079e83a288787bcd14a6aea84cee5c87a67c5a3e660c30f557a3d24761b3527"

[[package]]
name = "chacha20"
version = "0.10.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "65c35e4b699c7e15ccbe7ee35c005e4fc0a278d22238a2857e6ce2dadeda1b06"
dependencies = [
 "cfg-if",
 "cpufeatures 0.3.1",
 "rand_core 0.10.1",
]

[[package]]
name = "cipher"
version = "0.4.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "773f3b9af64447d2ce9850330c473515014aa235e6a783b02db81ff39e4a3dad"
dependencies = [
 "crypto-common 0.1.7",
 "inout",
]

[[package]]
name = "clipboard-win"
version = "5.4.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bde03770d3df201d4fb868f2c9c59e66a3e4e2bd06692a0fe701e7103c7e84d4"
dependencies = [
 "error-code",
]

[[package]]
name = "cmov"
version = "0.5.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0c9ea0ac24bc397ab3c98583a3c9ba74fa56b09a4449bbe172b9b1ddb016027a"

[[package]]
name = "cobs"
version = "0.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0fa961b519f0b462e3a3b4a34b64d119eeaca1d59af726fe450bbba07a9fc0a1"
dependencies = [
 "thiserror 2.0.21",
]

[[package]]
name = "codespan-reporting"
version = "0.13.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "af491d569909a7e4dee0ad7db7f5341fef5c614d5b8ec8cf765732aba3cff681"
dependencies = [
 "serde",
 "termcolor",
 "unicode-width",
]

[[package]]
name = "color"
version = "0.3.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2ec7c5eb7a16992b1904d76c517d170ab353b0e0b3d5a0c81a8a0cd1037893cf"
dependencies = [
 "bytemuck",
 "libm",
]

[[package]]
name = "color_quant"
version = "1.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3d7b894f5411737b7867f4827955924d7c254fc9f4d91a6aad6b097804b1018b"

[[package]]
name = "colored"
version = "3.1.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "faf9468729b8cbcea668e36183cb69d317348c2e08e994829fb56ebfdfbaac34"
dependencies = [
 "windows-sys 0.61.2",
]

[[package]]
name = "combine"
version = "4.6.8"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cfc320937d09e6de266b31b9afb480f197d7a861be86be7cb2ea7e5d1bfffc5e"
dependencies = [
 "bytes",
 "memchr",
]

[[package]]
name = "compact_str"
version = "0.9.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9dfdd1c2274d9aa354115b09dc9a901d6c5576818cdf70d14cae2bdb47df00ab"
dependencies = [
 "castaway",
 "cfg-if",
 "itoa",
 "rustversion",
 "ryu",
 "static_assertions",
]

[[package]]
name = "compression-codecs"
version = "0.4.45"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "98fc98460ba0ad5317075d3632b8dfc45d0be8c4a49347c2a38272019717614a"
dependencies = [
 "brotli",
 "compression-core",
 "flate2",
 "memchr",
 "zstd",
 "zstd-safe",
]

[[package]]
name = "compression-core"
version = "0.4.33"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6e8ccc4ea9f6acc32d102c0f6d471d11d913ad15f20c04de743374861fa1d414"

[[package]]
name = "concurrent-queue"
version = "2.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4ca0197aee26d1ae37445ee532fefce43251d24cc7c166799f4d46817f1d3973"
dependencies = [
 "crossbeam-utils",
]

[[package]]
name = "const-oid"
version = "0.10.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a6ef517f0926dd24a1582492c791b6a4818a4d94e789a334894aa15b0d12f55c"

[[package]]
name = "const-random"
version = "0.1.18"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "87e00182fe74b066627d63b85fd550ac2998d4b0bd86bfed477a0ae4c7c71359"
dependencies = [
 "const-random-macro",
]

[[package]]
name = "const-random-macro"
version = "0.1.16"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f9d839f2a20b0aee515dc581a6172f2321f96cab76c1a38a4c584a194955390e"
dependencies = [
 "getrandom 0.2.17",
 "once_cell",
 "tiny-keccak",
]

[[package]]
name = "cookie"
version = "0.16.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e859cd57d0710d9e06c381b550c06e76992472a8c6d527aecd2fc673dcc231fb"
dependencies = [
 "time",
 "version_check",
]

[[package]]
name = "cookie"
version = "0.18.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1a373e3602691c3cdea496d2f0ee5935151e6168fe87739483c463db1b2f2f87"
dependencies = [
 "time",
 "version_check",
]

[[package]]
name = "core-foundation"
version = "0.9.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "91e195e091a93c46f7102ec7818a2aa394e1e1771c3ab4825963fa03e45afb8f"
dependencies = [
 "core-foundation-sys",
 "libc",
]

[[package]]
name = "core-foundation"
version = "0.10.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b2a6cd9ae233e7f62ba4e9353e81a88df7fc8a5987b8d445b4d90c879bd156f6"
dependencies = [
 "core-foundation-sys",
 "libc",
]

[[package]]
name = "core-foundation-sys"
version = "0.8.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "773648b94d0e5d620f64f280777445740e61fe701025087ec8b57f45c791888b"

[[package]]
name = "core-graphics"
version = "0.23.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c07782be35f9e1140080c6b96f0d44b739e2278479f64e02fdab4e32dfd8b081"
dependencies = [
 "bitflags 1.3.2",
 "core-foundation 0.9.4",
 "core-graphics-types",
 "foreign-types",
 "libc",
]

[[package]]
name = "core-graphics-types"
version = "0.1.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "45390e6114f68f718cc7a830514a96f903cccd70d02a8f6d9f643ac4ba45afaf"
dependencies = [
 "bitflags 1.3.2",
 "core-foundation 0.9.4",
 "libc",
]

[[package]]
name = "core_detect"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7f8f80099a98041a3d1622845c271458a2d73e688351bf3cb999266764b81d48"

[[package]]
name = "core_maths"
version = "0.1.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "77745e017f5edba1a9c1d854f6f3a52dac8a12dd5af5d2f54aecf61e43d80d30"
dependencies = [
 "libm",
]

[[package]]
name = "cow-utils"
version = "0.1.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "417bef24afe1460300965a25ff4a24b8b45ad011948302ec221e8a0a81eb2c79"

[[package]]
name = "cpufeatures"
version = "0.2.17"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "59ed5838eebb26a2bb2e58f6d5b5316989ae9d08bab10e0e6d103e656d1b0280"
dependencies = [
 "libc",
]

[[package]]
name = "cpufeatures"
version = "0.3.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5ca28b0ae3115b884660db4118d803791fd6756b6e88f39c0f3f7859060d7566"
dependencies = [
 "libc",
]

[[package]]
name = "crc"
version = "3.4.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5eb8a2a1cd12ab0d987a5d5e825195d372001a4094a0376319d5a0ad71c1ba0d"
dependencies = [
 "crc-catalog",
]

[[package]]
name = "crc-catalog"
version = "2.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "217698eaf96b4a3f0bc4f3662aaa55bdf913cd54d7204591faa790070c6d0853"

[[package]]
name = "crc32c"
version = "0.6.8"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3a47af21622d091a8f0fb295b88bc886ac74efcc613efc19f5d0b21de5c89e47"
dependencies = [
 "rustc_version",
]

[[package]]
name = "crc32fast"
version = "1.5.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "01a7799fd6b852db0e61728dde9a204c423b44d689dbd432522543614b490e78"
dependencies = [
 "cfg-if",
]

[[package]]
name = "critical-section"
version = "1.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "790eea4361631c5e7d22598ecd5723ff611904e3344ce8720784c93e3d83d40b"

[[package]]
name = "crossbeam-channel"
version = "0.5.17"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "98b0cc327b5bc766e7fda9c9260cc0fa81b43a8e240440422dff70788e3f9ef1"
dependencies = [
 "crossbeam-utils",
]

[[package]]
name = "crossbeam-deque"
version = "0.8.8"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "622f3fc73690be383c7214310406f28a90e6edeadc3cea882f9d71e495b9711a"
dependencies = [
 "crossbeam-epoch",
 "crossbeam-utils",
]

[[package]]
name = "crossbeam-epoch"
version = "0.9.21"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "dc74980687109a3b14c72fd458107bf0baa1da1a1a805e178d15501ba9b86d9d"
dependencies = [
 "crossbeam-utils",
]

[[package]]
name = "crossbeam-utils"
version = "0.8.23"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a31eee39dddec8330830986fcd7625edb5a24ec90ea038215273bbc3adb08ac6"

[[package]]
name = "crunchy"
version = "0.2.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "460fbee9c2c2f33933d720630a6a0bac33ba7053db5344fac858d4b8952d77d5"

[[package]]
name = "crypto-common"
version = "0.1.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "78c8292055d1c1df0cce5d180393dc8cce0abec0a7102adb6c7b1eef6016d60a"
dependencies = [
 "generic-array",
 "rand_core 0.6.4",
 "typenum",
]

[[package]]
name = "crypto-common"
version = "0.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ce6e4c961d6cd6c9a86db418387425e8bdeaf05b3c8bc1411e6dca4c252f1453"
dependencies = [
 "hybrid-array",
]

[[package]]
name = "cssparser"
version = "0.38.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "11119743ad110e8c1bdccd930d7f5c30c99e5fc76a7b63ec9807e84eef0c5f59"
dependencies = [
 "cssparser-macros",
 "dtoa-short",
 "itoa",
 "phf 0.14.0",
 "serde",
 "smallvec",
]

[[package]]
name = "cssparser-macros"
version = "0.7.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d045de693cb712d0b22c6a64be5b953f67b3ce00ab5ad3dd5d8b441886ab8e1a"
dependencies = [
 "quote",
 "syn 3.0.6",
]

[[package]]
name = "ctr"
version = "0.9.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0369ee1ad671834580515889b80f2ea915f23b8be8d0daa4bbaf2ac5c7590835"
dependencies = [
 "cipher",
]

[[package]]
name = "ctutils"
version = "0.4.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "03bb0e1cc970d482d121d9a1744999169b69a07470b3d644a7894e53fcaf4574"
dependencies = [
 "cmov",
]

[[package]]
name = "cursor-icon"
version = "1.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f27ae1dd37df86211c42e150270f82743308803d90a6f6e6651cd730d5e1732f"

[[package]]
name = "dashmap"
version = "6.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e6361d5c062261c78a176addb82d4c821ae42bed6089de0e12603cd25de2059c"
dependencies = [
 "cfg-if",
 "crossbeam-utils",
 "hashbrown 0.14.5",
 "lock_api",
 "once_cell",
 "parking_lot_core",
]

[[package]]
name = "data-encoding"
version = "2.11.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4583a4551df46e2792f82ceeac45e850d2e2d5debba0b91f102385cda5b11f06"

[[package]]
name = "data-url"
version = "0.3.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "be1e0bca6c3637f992fc1cc7cbc52a78c1ef6db076dbf1059c4323d6a2048376"

[[package]]
name = "der-parser"
version = "9.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5cd0a5c643689626bec213c4d8bd4d96acc8ffdb4ad4bb6bc16abf27d5f4b553"
dependencies = [
 "asn1-rs 0.6.2",
 "displaydoc",
 "nom 7.1.3",
 "num-bigint 0.4.8",
 "num-traits",
 "rusticata-macros",
]

[[package]]
name = "der-parser"
version = "10.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "07da5016415d5a3c4dd39b11ed26f915f52fc4e0dc197d87908bc916e51bc1a6"
dependencies = [
 "asn1-rs 0.7.2",
 "displaydoc",
 "nom 7.1.3",
 "num-bigint 0.4.8",
 "num-traits",
 "rusticata-macros",
]

[[package]]
name = "deranged"
version = "0.5.8"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7cd812cc2bc1d69d4764bd80df88b4317eaef9e773c75226407d9bc0876b211c"

[[package]]
name = "digest"
version = "0.10.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9ed9a281f7bc9b7576e61468ba615a66a5c8cfdff42420a70aa82701a3b1e292"
dependencies = [
 "block-buffer 0.10.4",
 "crypto-common 0.1.7",
 "subtle",
]

[[package]]
name = "digest"
version = "0.11.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f1dd6dbb5841937940781866fa1281a1ff7bd3bf827091440879f9994983d5c2"
dependencies = [
 "block-buffer 0.12.1",
 "const-oid",
 "crypto-common 0.2.2",
 "ctutils",
]

[[package]]
name = "dispatch"
version = "0.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bd0c93bb4b0c6d9b77f4435b0ae98c24d17f1c45b2ff844c6151a07256ca923b"

[[package]]
name = "dispatch2"
version = "0.3.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1e0e367e4e7da84520dedcac1901e4da967309406d1e51017ae1abfb97adbd38"
dependencies = [
 "bitflags 2.13.2",
 "objc2 0.6.5",
]

[[package]]
name = "displaydoc"
version = "0.2.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c6232dd377dcc64799954cbd3a9bb882e9cdc1308ccd87b1c098f1fb2eaf82a8"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 3.0.6",
]

[[package]]
name = "dlib"
version = "0.5.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ab8ecd87370524b461f8557c119c405552c396ed91fc0a8eec68679eab26f94a"
dependencies = [
 "libloading",
]

[[package]]
name = "document-features"
version = "0.2.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d4b8a88685455ed29a21542a33abd9cb6510b6b129abadabdcef0f4c55bc8f61"
dependencies = [
 "litrs",
]

[[package]]
name = "document-session-api"
version = "0.1.1"
dependencies = [
 "accesskit",
 "serde",
]

[[package]]
name = "dof"
version = "0.4.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0ed9b77e3c2a83995eedff2fbf992eef44c9f319b8016254f68108e27a4d06e7"
dependencies = [
 "goblin",
 "pretty-hex",
 "serde",
 "serde_json",
 "thiserror 2.0.21",
 "zerocopy",
]

[[package]]
name = "downcast-rs"
version = "1.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "75b325c5dbd37f80359721ad39aca5a29fb04c89279657cffdda8736d0c0b9d2"

[[package]]
name = "dpi"
version = "0.1.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d8b14ccef22fc6f5a8f4d7d768562a182c04ce9a3b3157b91390b52ddfdf1a76"

[[package]]
name = "dragonbox_ecma"
version = "0.1.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "fd8e701084c37e7ef62d3f9e453b618130cbc0ef3573847785952a3ac3f746bf"

[[package]]
name = "dtoa"
version = "1.0.11"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4c3cf4824e2d5f025c7b531afcb2325364084a16806f6d47fbc1f5fbd9960590"

[[package]]
name = "dtoa-short"
version = "0.3.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cd1511a7b6a56299bd043a9c167a6d2bfb37bf84a6dfceaba651168adfb43c87"
dependencies = [
 "dtoa",
]

[[package]]
name = "dtrace-parser"
version = "0.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "dc09b90bda5770641457f1c0a42c8203c48f5a3d9799dcf1bafbd84e30ccf080"
dependencies = [
 "pest",
 "pest_derive",
 "thiserror 2.0.21",
]

[[package]]
name = "dynify"
version = "0.1.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "81acb15628a3e22358bf73de5e7e62360b8a777dbcb5fc9ac7dfa9ae73723747"
dependencies = [
 "dynify-macros",
]

[[package]]
name = "dynify-macros"
version = "0.1.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1ec431cd708430d5029356535259c5d645d60edd3d39c54e5eea9782d46caa7d"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "ecmascript_atomics"
version = "0.2.3"
source = "git+https://github.com/merely-made/vano.git?rev=b1f3d25d4b7c62ebfc4977346f55dc152ca2cc88#b1f3d25d4b7c62ebfc4977346f55dc152ca2cc88"
dependencies = [
 "ecmascript_atomics_gen_copy",
]

[[package]]
name = "ecmascript_atomics_gen_copy"
version = "0.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "51a3f9ed30f00d410e138929ee98104d8e951eed49e0fb3e84234ec818193d63"

[[package]]
name = "either"
version = "1.19.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0e9c71c2167ca323c882b99918929403426e2373ea17242ff5653e0d5e1058be"

[[package]]
name = "embedded-io"
version = "0.4.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ef1a6892d9eef45c8fa6b9e0086428a2cca8491aca8f787c534a3d6d0bcb3ced"

[[package]]
name = "embedded-io"
version = "0.6.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "edd0f118536f44f5ccd48bcb8b111bdc3de888b58c74639dfb034a357d0f206d"

[[package]]
name = "encoding_rs"
version = "0.8.42"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8e985e0451871ad22fb8d2b6b076e2028a502a0d3950998c2c5c0a4f9b5d9679"
dependencies = [
 "cfg-if",
 "core_detect",
 "multiversion_no_op",
 "rustversion",
 "scopeguard",
 "serde",
 "simdutf8",
]

[[package]]
name = "endi"
version = "1.1.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "66b7e2430c6dff6a955451e2cfc438f09cea1965a9d6f87f7e3b90decc014099"

[[package]]
name = "engine-observables-api"
version = "0.2.0"
dependencies = [
 "genet-paint-types",
 "serde",
 "serde_json",
]

[[package]]
name = "enumflags2"
version = "0.7.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1027f7680c853e056ebcec683615fb6fbbc07dbaa13b4d5d9442b146ded4ecef"
dependencies = [
 "enumflags2_derive",
 "serde",
]

[[package]]
name = "enumflags2_derive"
version = "0.7.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "67c78a4d8fdf9953a5c9d458f9efe940fd97a0cab0941c075a813ac594733827"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "enumn"
version = "0.1.14"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2f9ed6b3789237c8a0c1c505af1c7eb2c560df6186f01b098c3a1064ea532f38"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "equator"
version = "0.4.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4711b213838dfee0117e3be6ac926007d7f433d7bbe33595975d4190cb07e6fc"
dependencies = [
 "equator-macro",
]

[[package]]
name = "equator-macro"
version = "0.4.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "44f23cf4b44bfce11a86ace86f8a73ffdec849c9fd00a386a53d278bd9e81fb3"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "equivalent"
version = "1.0.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "877a4ace8713b0bcf2a4e7eec82529c029f1d0619886d18145fea96c3ffe5c0f"

[[package]]
name = "errno"
version = "0.3.14"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "39cab71617ae0d63f51a36d69f866391735b51691dbda63cf6f96d042b63efeb"
dependencies = [
 "libc",
 "windows-sys 0.61.2",
]

[[package]]
name = "error-code"
version = "3.4.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0b5343afd4a8365a643ac588dab4cf234a190c7f6c88c9f6dd6ffe00837661b7"

[[package]]
name = "euclid"
version = "0.22.14"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f1a05365e3b1c6d1650318537c7460c6923f1abdd272ad6842baa2b509957a06"
dependencies = [
 "num-traits",
 "serde",
]

[[package]]
name = "event-listener"
version = "5.4.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5a23add41df1562121a9393cb065eab5146a1242410f23a644851e90cfd669d2"
dependencies = [
 "parking",
 "pin-project-lite",
]

[[package]]
name = "event-listener-strategy"
version = "0.5.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8be9f3dfaaffdae2972880079a491a1a8bb7cbed0b8dd7a347f668b4150a3b93"
dependencies = [
 "event-listener",
 "pin-project-lite",
]

[[package]]
name = "exr"
version = "1.74.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "711fe42c9964295e01ee3fba3f9fe0e1d24b98886950d68efe81b1c76e21adf3"
dependencies = [
 "bit_field",
 "half",
 "lebe",
 "miniz_oxide 0.8.9",
 "num-complex",
 "pulp",
 "rayon-core",
 "smallvec",
 "zune-inflate",
]

[[package]]
name = "fast-float"
version = "0.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "95765f67b4b18863968b4a1bd5bb576f732b29a4a28c7cd84c09fa3e2875f33c"

[[package]]
name = "fast-float2"
version = "0.2.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c6e8948ce679d00a02a94739ea185595dca7118ed04feb991127e443bd3d761f"

[[package]]
name = "fastbloom"
version = "0.17.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ef975e30683b2d965054bb0a836f8973857c4ebf6acf274fe46617cd285060d8"
dependencies = [
 "foldhash 0.2.0",
 "libm",
 "portable-atomic",
 "siphasher",
]

[[package]]
name = "fastrand"
version = "2.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "da7c62ceae207dd37ea5b845da6a0696c799f85e97da1ab5b7910be3c1c80223"

[[package]]
name = "faststr"
version = "0.2.34"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1ca7d44d22004409a61c393afb3369c8f7bb74abcae49fe249ee01dcc3002113"
dependencies = [
 "bytes",
 "rkyv",
 "serde",
 "simdutf8",
]

[[package]]
name = "fax"
version = "0.2.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "caf1079563223d5d59d83c85886a56e586cfd5c1a26292e971a0fa266531ac5a"

[[package]]
name = "fdeflate"
version = "0.3.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1e6853b52649d4ac5c0bd02320cddc5ba956bdb407c4b75a2c6b75bf51500f8c"
dependencies = [
 "simd-adler32",
]

[[package]]
name = "fearless_simd"
version = "0.7.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f4beca3cb2444e3304ac30843cc091f44ed58932353cd492ce740067bfce6b12"

[[package]]
name = "find-msvc-tools"
version = "0.1.14"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "aedcfb3409746eddb02b9e19ebda1c3394f759a152e48ee875a0844d1b955484"

[[package]]
name = "fixedbitset"
version = "0.1.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "86d4de0081402f5e88cdac65c8dcdcc73118c1a7a465e2a05f0da05843a8ea33"

[[package]]
name = "fixedbitset"
version = "0.5.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1d674e81391d1e1ab681a28d99df07927c6d4aa5b027d7da16ba32d1d21ecd99"

[[package]]
name = "flate2"
version = "1.1.10"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6e634e2e0ebac1ee034020da1ca582e17ffe4e0f5e985823721e168928136dcb"
dependencies = [
 "crc32fast",
 "miniz_oxide 0.9.1",
 "zlib-rs",
]

[[package]]
name = "fleece"
version = "0.5.0"
dependencies = [
 "genet-static-dom",
 "layout-dom-api",
 "serde",
 "serde_json",
 "sha2",
 "unicode-bidi",
 "unicode-segmentation",
]

[[package]]
name = "float16"
version = "0.1.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a9442b5e9a7997eace4990debae01c1b69a1a601b9eeef7fada6c3f3edeff4a1"
dependencies = [
 "cfg-if",
 "rustc_version",
]

[[package]]
name = "fnv"
version = "1.0.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3f9eec918d3f24069decb9af1554cad7c880e2da24a9afd88aca000531ab82c1"

[[package]]
name = "foldhash"
version = "0.1.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d9c4f5dac5e15c24eb999c26181a6ca40b39fe946cbe4c263c7209467bc83af2"

[[package]]
name = "foldhash"
version = "0.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "77ce24cb58228fbb8aa041425bb1050850ac19177686ea6e0f41a70416f56fdb"

[[package]]
name = "font-types"
version = "0.12.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6c92d9081c7ab3bd83ef1eecdfad090f3d966e3ce74106779664114cfe1a6274"
dependencies = [
 "bytemuck",
]

[[package]]
name = "fontsan"
version = "0.7.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cff76797a10ce7f8439f6e0a5ed06783e26e60d97e28c8c1c623a608ca67cff9"
dependencies = [
 "cc",
 "glob",
 "libc",
 "libz-sys",
 "wuff-capi",
]

[[package]]
name = "foreign-types"
version = "0.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d737d9aa519fb7b749cbc3b962edcf310a8dd1f4b67c91c4f83975dbdd17d965"
dependencies = [
 "foreign-types-macros",
 "foreign-types-shared",
]

[[package]]
name = "foreign-types-macros"
version = "0.2.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ea5190182e6915eb873ddbc16e23b711b6eb1f9c00a0d0a3a91b5f6228475225"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 3.0.6",
]

[[package]]
name = "foreign-types-shared"
version = "0.3.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "aa9a19cbb55df58761df49b23516a86d432839add4af60fc256da840f66ed35b"

[[package]]
name = "form_urlencoded"
version = "1.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cb4cb245038516f5f85277875cdaa4f7d2c9a0fa0468de06ed190163b1581fcf"
dependencies = [
 "percent-encoding",
]

[[package]]
name = "funty"
version = "2.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e6d5a32815ae3f33302d95fdcb2ce17862f8c65363dcfd29360480ba1001fc9c"

[[package]]
name = "futures"
version = "0.3.34"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9a31d2a3fbaaeb2af2368bbdd904aa8e812d3c04a1ee10d3171f52d556e5d0a3"
dependencies = [
 "futures-channel",
 "futures-core",
 "futures-executor",
 "futures-io",
 "futures-sink",
 "futures-task",
 "futures-util",
]

[[package]]
name = "futures-channel"
version = "0.3.34"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b1f9e3d69d39e4862ffed03ed071a76f9a13ba1d9109d355b0f0aa6b15e393c4"
dependencies = [
 "futures-core",
 "futures-sink",
]

[[package]]
name = "futures-concurrency"
version = "7.7.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "175cd8cca9e1d45b87f18ffa75088f2099e3c4fe5e2f83e42de112560bea8ea6"
dependencies = [
 "fixedbitset 0.5.7",
 "futures-core",
 "futures-lite",
 "pin-project",
 "smallvec",
]

[[package]]
name = "futures-core"
version = "0.3.34"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "92d699e522242e69e3003b94ecc1f960f3a5e015aa7c5d7486e65ad01dd94f5e"

[[package]]
name = "futures-executor"
version = "0.3.34"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "031b47cf1a3c6cc8bc2fc76cd437f521619387907d469316e7c0bc278f1f5432"
dependencies = [
 "futures-core",
 "futures-task",
 "futures-util",
]

[[package]]
name = "futures-intrusive"
version = "0.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1d930c203dd0b6ff06e0201a4a2fe9149b43c684fd4420555b26d21b1a02956f"
dependencies = [
 "futures-core",
 "lock_api",
 "parking_lot",
]

[[package]]
name = "futures-io"
version = "0.3.34"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "53c0fa8157de1303bfffdaa1cc2a673bfffb60102f76b0ef4441659124373fed"

[[package]]
name = "futures-lite"
version = "2.6.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f78e10609fe0e0b3f4157ffab1876319b5b0db102a2c60dc4626306dc46b44ad"
dependencies = [
 "fastrand",
 "futures-core",
 "futures-io",
 "parking",
 "pin-project-lite",
]

[[package]]
name = "futures-macro"
version = "0.3.34"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9fb9654ba8355388abeb8dcb4fc62f511300867002afc858860463bdd9fe0c44"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 3.0.6",
]

[[package]]
name = "futures-sink"
version = "0.3.34"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1944426bf7d03f1d14f708785e4b33efd750b36d48a157b836b3efc15ede8e1d"

[[package]]
name = "futures-task"
version = "0.3.34"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cd417de3d1d015fc3bfd2b1ea46dfc7bab72ef86f1cc7cc9c78e728b34a6d1fd"

[[package]]
name = "futures-util"
version = "0.3.34"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0d50a92467f8ba5dd6e3ee5d4bd04d73ab2e4e1c44474a0674821dfce14b79bc"
dependencies = [
 "futures-channel",
 "futures-core",
 "futures-io",
 "futures-macro",
 "futures-sink",
 "futures-task",
 "memchr",
 "pin-project-lite",
 "slab",
]

[[package]]
name = "gc-arena"
version = "0.5.3"
source = "git+https://github.com/kyren/gc-arena?rev=5a7534b883b703f23cfb8c3cfdf033460aa77ea9#5a7534b883b703f23cfb8c3cfdf033460aa77ea9"
dependencies = [
 "allocator-api2",
 "gc-arena-derive",
 "hashbrown 0.14.5",
 "sptr",
]

[[package]]
name = "gc-arena-derive"
version = "0.5.3"
source = "git+https://github.com/kyren/gc-arena?rev=5a7534b883b703f23cfb8c3cfdf033460aa77ea9#5a7534b883b703f23cfb8c3cfdf033460aa77ea9"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 2.0.119",
 "synstructure 0.13.2",
]

[[package]]
name = "generic-array"
version = "0.14.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "85649ca51fd72272d7821adaf274ad91c288277713d9c18820d8499a7ff69e9a"
dependencies = [
 "typenum",
 "version_check",
]

[[package]]
name = "genet-bole"
version = "0.0.0"
dependencies = [
 "boa_engine",
 "boa_gc",
 "script-engine-api",
]

[[package]]
name = "genet-browsing"
version = "0.1.0"
dependencies = [
 "url",
]

[[package]]
name = "genet-clipboard"
version = "0.1.0"
dependencies = [
 "arboard",
]

[[package]]
name = "genet-compositor"
version = "0.2.0"
dependencies = [
 "ash",
 "bytemuck",
 "dpi",
 "euclid",
 "genet-paint-types",
 "image",
 "log",
 "netrender_device",
 "objc2 0.6.5",
 "objc2-app-kit 0.3.2",
 "objc2-core-foundation",
 "objc2-foundation 0.3.2",
 "objc2-io-surface",
 "objc2-metal 0.3.2",
 "objc2-quartz-core 0.3.2",
 "pollster",
 "raw-window-handle",
 "rustc-hash 2.1.3",
 "smallvec",
 "wayland-client",
 "wayland-protocols",
 "wgpu",
 "windows",
]

[[package]]
name = "genet-didge"
version = "0.0.0"
dependencies = [
 "gc-arena",
 "piccolo",
 "script-engine-api",
]

[[package]]
name = "genet-document-resources"
version = "0.2.1"
dependencies = [
 "genet-host-api",
 "genet-static-dom",
 "layout-dom-api",
]

[[package]]
name = "genet-documents"
version = "0.1.0"
dependencies = [
 "data-url",
 "document-session-api",
 "fleece",
 "genet-bole",
 "genet-browsing",
 "genet-document-resources",
 "genet-host-api",
 "genet-livery",
 "genet-render",
 "genet-scripted",
 "genet-scripted-dom",
 "genet-static-dom",
 "genet-vano",
 "image",
 "layout-dom-api",
 "netrender",
 "paint_list_api",
 "paint_list_render",
 "script-engine-api",
 "serde_json",
 "url",
]

[[package]]
name = "genet-essl"
version = "0.2.0"
dependencies = [
 "naga",
 "rspirv",
]

[[package]]
name = "genet-events"
version = "0.0.0"
dependencies = [
 "genet-scripted-dom",
 "layout-dom-api",
]

[[package]]
name = "genet-flow"
version = "0.1.0"

[[package]]
name = "genet-fontique"
version = "0.10.0"
dependencies = [
 "hashbrown 0.17.1",
 "linebender_resource_handle",
 "memmap2",
 "objc2 0.6.5",
 "objc2-core-foundation",
 "objc2-core-text",
 "objc2-foundation 0.3.2",
 "parlance",
 "read-fonts",
 "roxmltree",
 "smallvec",
 "windows",
 "windows-core",
 "yeslogic-fontconfig-sys",
]

[[package]]
name = "genet-host-api"
version = "0.2.0"
dependencies = [
 "serde",
]

[[package]]
name = "genet-idl"
version = "0.0.0"

[[package]]
name = "genet-livery"
version = "0.0.4"
dependencies = [
 "base64 0.22.1",
 "buckram",
 "cssparser",
 "data-url",
 "euclid",
 "fontsan",
 "genet-document-resources",
 "genet-parley",
 "genet-scripted-dom",
 "genet-static-dom",
 "genet-taffy",
 "icu_properties",
 "icu_segmenter 2.3.0",
 "image",
 "layout-dom-api",
 "livery",
 "netrender",
 "paint_list_api",
 "paint_list_render",
 "postcard",
 "read-fonts",
 "serde",
 "stacker",
]

[[package]]
name = "genet-paint-types"
version = "0.2.0"
dependencies = [
 "app_units",
 "euclid",
 "serde",
]

[[package]]
name = "genet-parley"
version = "0.10.0"
dependencies = [
 "accesskit",
 "core_maths",
 "genet-fontique",
 "harfrust",
 "hashbrown 0.17.1",
 "icu_normalizer",
 "icu_properties",
 "icu_provider 2.3.1",
 "icu_segmenter 2.3.0",
 "linebender_resource_handle",
 "oxipng",
 "parlance",
 "parley_data",
 "peniko",
 "skrifa",
 "zerovec 0.11.8",
]

[[package]]
name = "genet-presenter"
version = "0.2.0"
dependencies = [
 "genet-livery",
 "genet-scripted-dom",
 "gloo-timers",
 "image",
 "js-sys",
 "netrender",
 "paint_list_api",
 "paint_list_render",
 "tracing",
 "wasm-bindgen",
 "wasm-bindgen-futures",
 "web-time",
 "wgpu",
]

[[package]]
name = "genet-render"
version = "0.2.0"
dependencies = [
 "accesskit",
 "document-session-api",
 "genet-livery",
 "genet-scripted-dom",
 "genet-static-dom",
 "layout-dom-api",
 "netrender",
 "paint_list_api",
 "paint_list_render",
]

[[package]]
name = "genet-runtime"
version = "0.0.0"
dependencies = [
 "cadency",
 "encoding_rs",
 "genet-bole",
 "genet-browsing",
 "genet-events",
 "genet-host-api",
 "genet-idl",
 "genet-scripted-dom",
 "genet-static-dom",
 "genet-vano",
 "genet-webgl",
 "getrandom 0.4.3",
 "hkdf",
 "hmac 0.13.0",
 "layout-dom-api",
 "libc",
 "livery",
 "markup5ever",
 "pbkdf2",
 "pollster",
 "script-engine-api",
 "serde",
 "serde_json",
 "servo-xpath",
 "sha1 0.11.0",
 "sha2",
 "url",
 "wgpu",
 "windows-sys 0.61.2",
 "zeroize",
]

[[package]]
name = "genet-scripted"
version = "0.2.0"
dependencies = [
 "base64 0.22.1",
 "encoding_rs",
 "engine-observables-api",
 "fleece",
 "genet-bole",
 "genet-document-resources",
 "genet-events",
 "genet-host-api",
 "genet-livery",
 "genet-render",
 "genet-runtime",
 "genet-scripted-dom",
 "genet-static-dom",
 "genet-vano",
 "layout-dom-api",
 "livery",
 "netrender",
 "paint_list_api",
 "paint_list_render",
 "postcard",
 "script-engine-api",
 "serde",
 "sha2",
 "url",
]

[[package]]
name = "genet-scripted-dom"
version = "0.1.2"
dependencies = [
 "encoding_rs",
 "engine-observables-api",
 "genet-static-dom",
 "genet-text",
 "html5ever",
 "layout-dom-api",
 "markup5ever",
 "postcard",
 "regress 0.11.1",
 "serde",
 "url",
]

[[package]]
name = "genet-scriptworker"
version = "0.2.0"
dependencies = [
 "genet-bole",
 "genet-scripted",
 "genet-vano",
 "getrandom 0.4.3",
 "wasm-bindgen",
]

[[package]]
name = "genet-static-dom"
version = "0.1.1"
dependencies = [
 "html5ever",
 "layout-dom-api",
 "xml5ever",
]

[[package]]
name = "genet-taffy"
version = "0.16.0"
dependencies = [
 "serde",
 "slotmap",
 "smallvec",
]

[[package]]
name = "genet-text"
version = "0.1.0"
dependencies = [
 "unicode-segmentation",
]

[[package]]
name = "genet-vano"
version = "0.0.0"
dependencies = [
 "nova_vm",
 "script-engine-api",
]

[[package]]
name = "genet-webgl"
version = "0.2.0"
dependencies = [
 "genet-compositor",
 "genet-essl",
 "pollster",
 "wgpu",
]

[[package]]
name = "genet-webrtc"
version = "0.2.0"
dependencies = [
 "bytes",
 "genet-bole",
 "genet-runtime",
 "genet-vano",
 "murm-rtc",
 "script-engine-api",
 "serde_json",
]

[[package]]
name = "genet-winit"
version = "0.2.0"
dependencies = [
 "accesskit",
 "accesskit_consumer",
 "accesskit_macos",
 "accesskit_unix",
 "accesskit_windows",
 "genet-presenter",
 "netrender",
 "raw-window-handle",
 "tracing",
 "wgpu",
 "winit",
]

[[package]]
name = "genet-worker"
version = "0.0.0"
dependencies = [
 "genet-bole",
 "genet-host-api",
 "genet-runtime",
 "genet-scripted",
 "genet-vano",
 "script-engine-api",
]

[[package]]
name = "genet-wpt"
version = "0.2.0"
dependencies = [
 "bitflags 2.13.2",
 "bytes",
 "dpi",
 "euclid",
 "genet-bole",
 "genet-document-resources",
 "genet-livery",
 "genet-paint-types",
 "genet-presenter",
 "genet-render",
 "genet-runtime",
 "genet-scripted",
 "genet-scripted-dom",
 "genet-static-dom",
 "genet-vano",
 "genet-webgl",
 "genet-webrtc",
 "image",
 "keyboard-types",
 "layout-dom-api",
 "livery",
 "mockito",
 "netfetcher",
 "netrender",
 "paint_list_api",
 "paint_list_render",
 "pollster",
 "script-engine-api",
 "serde",
 "serde_json",
 "sha2",
 "tokio",
 "url",
 "webdriver",
 "wgpu",
]

[[package]]
name = "gethostname"
version = "1.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1bd49230192a3797a9a4d6abe9b3eed6f7fa4c8a8a4947977c6f80025f92cbd8"
dependencies = [
 "rustix 1.1.5",
 "windows-link",
]

[[package]]
name = "getrandom"
version = "0.2.17"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ff2abc00be7fca6ebc474524697ae276ad847ad0a6b3faa4bcb027e9a4614ad0"
dependencies = [
 "cfg-if",
 "js-sys",
 "libc",
 "wasi",
 "wasm-bindgen",
]

[[package]]
name = "getrandom"
version = "0.3.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "899def5c37c4fd7b2664648c28120ecec138e4d395b459e5ca34f9cce2dd77fd"
dependencies = [
 "cfg-if",
 "libc",
 "r-efi 5.3.0",
 "wasip2",
]

[[package]]
name = "getrandom"
version = "0.4.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "300e883d756b2e4ec94e02791f39b04b522276138852cfc41d9fb7e904106099"
dependencies = [
 "cfg-if",
 "js-sys",
 "libc",
 "r-efi 6.0.0",
 "rand_core 0.10.1",
 "wasm-bindgen",
]

[[package]]
name = "gif"
version = "0.14.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ee8cfcc411d9adbbaba82fb72661cc1bcca13e8bba98b364e62b2dba8f960159"
dependencies = [
 "color_quant",
 "weezl",
]

[[package]]
name = "gio-sys"
version = "0.22.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6c28739f914c15b87a9856000bed9cbd5b3a8afbca5e982d9a299e1601422cb5"
dependencies = [
 "glib-sys",
 "gobject-sys",
 "libc",
 "system-deps",
 "windows-sys 0.61.2",
]

[[package]]
name = "gl_generator"
version = "0.14.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1a95dfc23a2b4a9a2f5ab41d194f8bfda3cabec42af4e39f08c339eb2a0c124d"
dependencies = [
 "khronos_api",
 "log",
 "xml-rs",
]

[[package]]
name = "glib"
version = "0.22.10"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e4cf9e5bd8713f1990a92c422998826e0fb34824277b46bc59757373483c6694"
dependencies = [
 "bitflags 2.13.2",
 "futures-channel",
 "futures-core",
 "futures-executor",
 "futures-task",
 "futures-util",
 "gio-sys",
 "glib-macros",
 "glib-sys",
 "gobject-sys",
 "libc",
 "memchr",
 "smallvec",
]

[[package]]
name = "glib-macros"
version = "0.22.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c9597c68fa7bdf4154a3079642cd21c4dccac0b0bdd2897d82861523bf91e7b9"
dependencies = [
 "heck",
 "proc-macro2",
 "quote",
 "syn 3.0.6",
]

[[package]]
name = "glib-sys"
version = "0.22.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "99b38907e67e40dec9b60f858bcada952598719cb512a13eb13d6cdcd16f8295"
dependencies = [
 "libc",
 "system-deps",
]

[[package]]
name = "glifo"
version = "0.4.0"
source = "git+https://github.com/mark-ik/vello?rev=276b46648c08f3d2062e70559afa999842e1873c#276b46648c08f3d2062e70559afa999842e1873c"
dependencies = [
 "bytemuck",
 "foldhash 0.2.0",
 "hashbrown 0.17.1",
 "log",
 "peniko",
 "skrifa",
 "smallvec",
 "vello_common",
]

[[package]]
name = "glob"
version = "0.3.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e4eba85ea1d0a966a983acd07deee566e67395d2d96b6fb39e62b5a833f1eb0b"

[[package]]
name = "gloo-timers"
version = "0.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bbb143cf96099802033e0d4f4963b19fd2e0b728bcf076cd9cf7f6634f092994"
dependencies = [
 "futures-channel",
 "futures-core",
 "js-sys",
 "wasm-bindgen",
]

[[package]]
name = "glow"
version = "0.17.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "29038e1c483364cc6bb3cf78feee1816002e127c331a1eec55a4d202b9e1adb5"
dependencies = [
 "js-sys",
 "slotmap",
 "wasm-bindgen",
 "web-sys",
]

[[package]]
name = "glutin_wgl_sys"
version = "0.6.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2c4ee00b289aba7a9e5306d57c2d05499b2e5dc427f84ac708bd2c090212cf3e"
dependencies = [
 "gl_generator",
]

[[package]]
name = "gobject-sys"
version = "0.22.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "07595c9ba696bd9819cb6a8df39f08078f3dd679508fe052dd558492c2053834"
dependencies = [
 "glib-sys",
 "libc",
 "system-deps",
]

[[package]]
name = "goblin"
version = "0.10.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "17582616a7718cca54cec18e534a76c7c4aec11a8b9a85695712f262fd15a4c8"
dependencies = [
 "log",
 "plain",
 "scroll",
]

[[package]]
name = "gpu-allocator"
version = "0.28.0"
dependencies = [
 "ash",
 "hashbrown 0.16.1",
 "log",
 "presser",
 "thiserror 2.0.21",
 "windows",
]

[[package]]
name = "gstreamer"
version = "0.25.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f4237e2066a25ecb7e54be6c066409e30615920265b9799b8d560ef8ac9857c6"
dependencies = [
 "cfg-if",
 "futures-channel",
 "futures-core",
 "futures-util",
 "glib",
 "gstreamer-sys",
 "itertools 0.15.0",
 "kstring",
 "libc",
 "muldiv",
 "num-integer",
 "num-rational",
 "option-operations",
 "pastey 0.2.3",
 "pin-project-lite",
 "smallvec",
 "thiserror 2.0.21",
]

[[package]]
name = "gstreamer-app"
version = "0.25.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "97f8ae9238c2352398dcc084de28df3f7099af216ac6c160b52318d23f25c010"
dependencies = [
 "futures-core",
 "futures-sink",
 "glib",
 "gstreamer",
 "gstreamer-app-sys",
 "gstreamer-base",
 "libc",
]

[[package]]
name = "gstreamer-app-sys"
version = "0.25.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0798b8aef6694341d40bb81edc759f8979b896b59f9f0f45d8cfa3bb7bf2e743"
dependencies = [
 "glib-sys",
 "gstreamer-base-sys",
 "gstreamer-sys",
 "libc",
 "system-deps",
]

[[package]]
name = "gstreamer-audio"
version = "0.25.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "544f14e19e3b7221845be0e5c2f2cf284576b7baa16835ac16def6e734bfa866"
dependencies = [
 "cfg-if",
 "glib",
 "gstreamer",
 "gstreamer-audio-sys",
 "gstreamer-base",
 "libc",
 "smallvec",
]

[[package]]
name = "gstreamer-audio-sys"
version = "0.25.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "09a010deb715bc29953a07c6714249132e736b3daf3a529e70586b58a7063888"
dependencies = [
 "glib-sys",
 "gobject-sys",
 "gstreamer-base-sys",
 "gstreamer-sys",
 "libc",
 "system-deps",
]

[[package]]
name = "gstreamer-base"
version = "0.25.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "17cbd8f59586a369c40d4b0d2f3a61cf9e9287b3cf5503cdcd8e7c5faa99265e"
dependencies = [
 "atomic_refcell",
 "cfg-if",
 "glib",
 "gstreamer",
 "gstreamer-base-sys",
 "libc",
]

[[package]]
name = "gstreamer-base-sys"
version = "0.25.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3b05f77bfd76b5afee6b29c27dae19eaf5018b5cf8d0d17520f3de44c6385c32"
dependencies = [
 "glib-sys",
 "gobject-sys",
 "gstreamer-sys",
 "libc",
 "system-deps",
]

[[package]]
name = "gstreamer-gl"
version = "0.25.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "653f035dde811c0c63d0f6b180f30213614d870732848f93c53203a2f3baffb8"
dependencies = [
 "glib",
 "gstreamer",
 "gstreamer-base",
 "gstreamer-gl-sys",
 "gstreamer-video",
 "libc",
]

[[package]]
name = "gstreamer-gl-egl"
version = "0.25.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ff3c29e6d4b92ef22b0bacc2cff02902d40516942fead63c656d7fccc75221e8"
dependencies = [
 "glib",
 "gstreamer",
 "gstreamer-gl",
 "gstreamer-gl-egl-sys",
 "libc",
]

[[package]]
name = "gstreamer-gl-egl-sys"
version = "0.25.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3ed05197968a7377c9ca94bf53a24915c9d763d16e7a264044627d3a74b93b7e"
dependencies = [
 "glib-sys",
 "gstreamer-gl-sys",
 "libc",
 "system-deps",
]

[[package]]
name = "gstreamer-gl-sys"
version = "0.25.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f533eaf84f0155f105cadddc61e4191e933bd9cf784dcb121dc503ecefc0bc9e"
dependencies = [
 "glib-sys",
 "gobject-sys",
 "gstreamer-base-sys",
 "gstreamer-sys",
 "gstreamer-video-sys",
 "libc",
 "system-deps",
]

[[package]]
name = "gstreamer-gl-wayland"
version = "0.25.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9a83ca3d2030846f01be738b8b867ecb4b95269497bcfa459b8fc1ce897ad9f4"
dependencies = [
 "glib",
 "gstreamer",
 "gstreamer-gl",
 "gstreamer-gl-wayland-sys",
 "libc",
]

[[package]]
name = "gstreamer-gl-wayland-sys"
version = "0.25.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b9a3cdb454672cc1d50483bec972142bf450c42c1586f199115c0e75b8b35694"
dependencies = [
 "glib-sys",
 "gstreamer-gl-sys",
 "libc",
 "system-deps",
]

[[package]]
name = "gstreamer-gl-x11"
version = "0.25.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "642d86fed827466e2d1ff4f430313fdab5d113bc2bf1e9c9736a873e8832c6ca"
dependencies = [
 "glib",
 "gstreamer",
 "gstreamer-gl",
 "gstreamer-gl-x11-sys",
 "libc",
]

[[package]]
name = "gstreamer-gl-x11-sys"
version = "0.25.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "69e6467e79e878fee2c5ea7b93703934e2da32eb91e39fc09a37b24f4df6bda3"
dependencies = [
 "glib-sys",
 "gstreamer-gl-sys",
 "libc",
 "system-deps",
]

[[package]]
name = "gstreamer-play"
version = "0.25.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3355c2c37bfd824cbd974bf0b8c004d71ca3755f825065957bc16356d79e6e34"
dependencies = [
 "glib",
 "gstreamer",
 "gstreamer-play-sys",
 "gstreamer-video",
 "libc",
]

[[package]]
name = "gstreamer-play-sys"
version = "0.25.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4545c131079772ebbb3343761bf6f83672b2cf9dbae575a667f4a41a1fd26786"
dependencies = [
 "glib-sys",
 "gobject-sys",
 "gstreamer-sys",
 "gstreamer-video-sys",
 "libc",
 "system-deps",
]

[[package]]
name = "gstreamer-sdp"
version = "0.25.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5a7859290ba340f1bd45d4bc0ada958700e9d3b1a522b456fd7ee88da468cd13"
dependencies = [
 "glib",
 "gstreamer",
 "gstreamer-sdp-sys",
]

[[package]]
name = "gstreamer-sdp-sys"
version = "0.25.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7e5afafebe4a135392ec2455144cec78821681e007cb9b19c071a4c86b218f64"
dependencies = [
 "glib-sys",
 "gstreamer-sys",
 "libc",
 "system-deps",
]

[[package]]
name = "gstreamer-sys"
version = "0.25.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "436d9532847489a8285d57b8c3670762c12a18f777fdc2ab159b52e0e6f521b9"
dependencies = [
 "cfg-if",
 "glib-sys",
 "gobject-sys",
 "libc",
 "system-deps",
]

[[package]]
name = "gstreamer-video"
version = "0.25.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ffe4d77b2a5e679973d9c6b6c052da8cda86b83f5c268872f8691fde693fde84"
dependencies = [
 "cfg-if",
 "futures-channel",
 "glib",
 "gstreamer",
 "gstreamer-base",
 "gstreamer-video-sys",
 "libc",
 "thiserror 2.0.21",
]

[[package]]
name = "gstreamer-video-sys"
version = "0.25.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ef455da22edca97ac9299d682b868227f3d9217b303748ce0161b31ae82dc106"
dependencies = [
 "glib-sys",
 "gobject-sys",
 "gstreamer-base-sys",
 "gstreamer-sys",
 "libc",
 "system-deps",
]

[[package]]
name = "gstreamer-webrtc"
version = "0.25.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "de524e63b32f3f478b9f718ef6895bd50a1bb1d1d4d7c2134f0a30f36d046157"
dependencies = [
 "glib",
 "gstreamer",
 "gstreamer-sdp",
 "gstreamer-webrtc-sys",
 "libc",
]

[[package]]
name = "gstreamer-webrtc-sys"
version = "0.25.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cf77b07fd279bf060d938c7024c8a152a566b4b5e9ff401c7ba890d278a47d1b"
dependencies = [
 "glib-sys",
 "gstreamer-sdp-sys",
 "gstreamer-sys",
 "libc",
 "system-deps",
]

[[package]]
name = "guillotiere"
version = "0.7.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6b17e70c989c36bad147b27a58d148c0741c51448aa5653436547323e524d0ab"
dependencies = [
 "euclid",
 "svg_fmt",
]

[[package]]
name = "h2"
version = "0.4.20"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7d29020232d6aa3fb1daca64c1127cf662cf97f254ae16c18c05b8ab635fc118"
dependencies = [
 "atomic-waker",
 "bytes",
 "fnv",
 "futures-core",
 "futures-sink",
 "http 1.5.0",
 "indexmap",
 "slab",
 "tokio",
 "tokio-util",
 "tracing",
]

[[package]]
name = "h3"
version = "0.0.8"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "10872b55cfb02a821b69dc7cf8dc6a71d6af25eb9a79662bec4a9d016056b3be"
dependencies = [
 "bytes",
 "fastrand",
 "futures-util",
 "http 1.5.0",
 "pin-project-lite",
 "tokio",
]

[[package]]
name = "h3-quinn"
version = "0.0.10"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8b2e732c8d91a74731663ac8479ab505042fbf547b9a207213ab7fbcbfc4f8b4"
dependencies = [
 "bytes",
 "futures",
 "h3",
 "quinn",
 "tokio",
 "tokio-util",
]

[[package]]
name = "half"
version = "2.7.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6ea2d84b969582b4b1864a92dc5d27cd2b77b622a8d79306834f1be5ba20d84b"
dependencies = [
 "cfg-if",
 "crunchy",
 "num-traits",
 "zerocopy",
]

[[package]]
name = "harfrust"
version = "0.12.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c03d949a14aa089bbb282f7dd76a498a7f684428e4257202efc119ec010376f9"
dependencies = [
 "bitflags 2.13.2",
 "bytemuck",
 "core_maths",
 "read-fonts",
 "smallvec",
]

[[package]]
name = "hash32"
version = "0.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b0c35f58762feb77d74ebe43bdbc3210f09be9fe6742234d573bacc26ed92b67"
dependencies = [
 "byteorder",
]

[[package]]
name = "hashbrown"
version = "0.14.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e5274423e17b7c9fc20b6e7e208532f9b19825d82dfd615708b70edd83df41f1"
dependencies = [
 "ahash",
 "allocator-api2",
]

[[package]]
name = "hashbrown"
version = "0.15.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9229cfe53dfd69f0609a49f65461bd93001ea1ef889cd5529dd176593f5338a1"
dependencies = [
 "foldhash 0.1.5",
]

[[package]]
name = "hashbrown"
version = "0.16.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "841d1cc9bed7f9236f321df977030373f4a4163ae1a7dbfe1a51a2c1a51d9100"
dependencies = [
 "allocator-api2",
 "equivalent",
 "foldhash 0.2.0",
]

[[package]]
name = "hashbrown"
version = "0.17.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ed5909b6e89a2db4456e54cd5f673791d7eca6732202bbf2a9cc504fe2f9b84a"
dependencies = [
 "allocator-api2",
 "equivalent",
 "foldhash 0.2.0",
]

[[package]]
name = "heapless"
version = "0.7.17"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cdc6457c0eb62c71aac4bc17216026d8410337c4126773b9c5daba343f17964f"
dependencies = [
 "atomic-polyfill",
 "hash32",
 "rustc_version",
 "serde",
 "spin",
 "stable_deref_trait",
]

[[package]]
name = "heck"
version = "0.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2304e00983f87ffb38b55b444b5e3b60a884b5d30c0fca7d82fe33449bbe55ea"

[[package]]
name = "hermit-abi"
version = "0.5.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e17592d60ebacc7d5e169f4663c5f84f9161cc90328abcfe8456f41e4dfcb284"

[[package]]
name = "hex"
version = "0.4.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7f24254aa9a54b5c858eaee2f5bccdb46aaf0e486a595ed5fd8f86ba55232a70"

[[package]]
name = "hkdf"
version = "0.13.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4aaa26c720c68b866f2c96ef5c1264b3e6f473fe5d4ce61cd44bbe913e553018"
dependencies = [
 "hmac 0.13.0",
]

[[package]]
name = "hmac"
version = "0.12.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6c49c37c09c17a53d937dfbb742eb3a961d65a994e6bcdcf37e7399d0cc8ab5e"
dependencies = [
 "digest 0.10.7",
]

[[package]]
name = "hmac"
version = "0.13.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6303bc9732ae41b04cb554b844a762b4115a61bfaa81e3e83050991eeb56863f"
dependencies = [
 "digest 0.11.3",
]

[[package]]
name = "html5ever"
version = "0.40.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "456a1a377e608e555d22ddab27ac0114bc7a7b4199078108e34c2aeae6c9b130"
dependencies = [
 "log",
 "markup5ever",
 "memchr",
]

[[package]]
name = "http"
version = "0.2.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "601cbb57e577e2f5ef5be8e7b83f0f63994f25aa94d673e54a92d5c516d101f1"
dependencies = [
 "bytes",
 "fnv",
 "itoa",
]

[[package]]
name = "http"
version = "1.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "918d3568bebf352712bc2ef3d46a8bcf1a75b373be6539de198e9105cbbf9ce0"
dependencies = [
 "bytes",
 "itoa",
]

[[package]]
name = "http-body"
version = "1.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ca2a8f2913ee65f60facd6a5905613afaa448497a0230cc41ce022d93290bc2c"
dependencies = [
 "bytes",
 "http 1.5.0",
]

[[package]]
name = "http-body-util"
version = "0.1.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "23169fe34a5fbcdd3f3862e78fb9b6fccd5f02a6dc6f732547005d45631ce71c"
dependencies = [
 "bytes",
 "futures-core",
 "http 1.5.0",
 "http-body",
 "pin-project-lite",
]

[[package]]
name = "httparse"
version = "1.10.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6dbf3de79e51f3d586ab4cb9d5c3e2c14aa28ed23d180cf89b4df0454a69cc87"

[[package]]
name = "httpdate"
version = "1.0.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "df3b46402a9d5adb4c86a0cf463f42e19994e3ee891101b1841f30a545cb49a9"

[[package]]
name = "hybrid-array"
version = "0.4.15"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "27f864f10dfb56725ce5ce5472bc52252c8f93a4ab86327122cebf62c5f59a17"
dependencies = [
 "typenum",
]

[[package]]
name = "hyper"
version = "1.12.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2c3e324da4c95177d6291d4c8730197c0d1822f8a9766814a4a44fa5ab797c9c"
dependencies = [
 "atomic-waker",
 "bytes",
 "futures-channel",
 "futures-core",
 "h2",
 "http 1.5.0",
 "http-body",
 "httparse",
 "httpdate",
 "itoa",
 "pin-project-lite",
 "smallvec",
 "tokio",
 "want",
]

[[package]]
name = "hyper-rustls"
version = "0.27.10"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "dfa8e654703247911e29c23fbeaa261834bd9bb74efba2f9acddc37bfb127f53"
dependencies = [
 "http 1.5.0",
 "hyper",
 "hyper-util",
 "rustls",
 "tokio",
 "tokio-rustls",
 "tower-service",
 "webpki-roots",
]

[[package]]
name = "hyper-util"
version = "0.1.21"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ddc03d96684f9226b8a787cdb71488417b53ab5ea8fdb1dac946cb9431cc8bff"
dependencies = [
 "bytes",
 "futures-channel",
 "futures-util",
 "http 1.5.0",
 "http-body",
 "httparse",
 "hyper",
 "libc",
 "pin-project-lite",
 "socket2",
 "tokio",
 "tower-service",
 "tracing",
]

[[package]]
name = "iana-time-zone"
version = "0.1.65"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e31bc9ad994ba00e440a8aa5c9ef0ec67d5cb5e5cb0cc7f8b744a35b389cc470"
dependencies = [
 "android_system_properties",
 "core-foundation-sys",
 "iana-time-zone-haiku",
 "js-sys",
 "log",
 "wasm-bindgen",
 "windows-core",
]

[[package]]
name = "iana-time-zone-haiku"
version = "0.1.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f31827a206f56af32e590ba56d5d2d085f558508192593743f16b2306495269f"
dependencies = [
 "cc",
]

[[package]]
name = "icu_calendar"
version = "2.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "58655df2f728e46e4eee80bddebefacf52ec3e77cdb925014b47c5a5905eb55c"
dependencies = [
 "calendrical_calculations",
 "displaydoc",
 "icu_calendar_data",
 "icu_locale_core",
 "icu_locale_fallback",
 "icu_provider 2.3.1",
 "tinystr 0.8.4",
 "zerovec 0.11.8",
]

[[package]]
name = "icu_calendar_data"
version = "2.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "dc00caaa3fb3201ff7a18aa458e8c3ab042b9da154cfdf948bdde3936c31c36e"

[[package]]
name = "icu_collections"
version = "1.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "db2fa452206ebee18c4b5c2274dbf1de17008e874b4dc4f0aea9d01ca79e4526"
dependencies = [
 "displaydoc",
 "yoke 0.7.5",
 "zerofrom",
 "zerovec 0.10.4",
]

[[package]]
name = "icu_collections"
version = "2.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "fa68d21081c4a05d5a901a1c62add574c77048b6a1c67be3b50ce0b60d4ca513"
dependencies = [
 "displaydoc",
 "potential_utf",
 "utf8_iter",
 "yoke 0.8.3",
 "zerofrom",
 "zerovec 0.11.8",
]

[[package]]
name = "icu_locale_core"
version = "2.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d56e28588da92eee5c3201a6eff33fabdd49b62269c8938d4ff050ce4d900deb"
dependencies = [
 "displaydoc",
 "litemap 0.8.3",
 "serde",
 "tinystr 0.8.4",
 "writeable 0.6.4",
 "zerovec 0.11.8",
]

[[package]]
name = "icu_locale_fallback"
version = "2.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "251af8e57c9400e3eb58242fe5b8b1152b2a64fdf4cf632f923c38ccee6f2fa9"
dependencies = [
 "icu_locale_core",
 "icu_locale_fallback_data",
 "icu_provider 2.3.1",
 "potential_utf",
 "tinystr 0.8.4",
 "zerovec 0.11.8",
]

[[package]]
name = "icu_locale_fallback_data"
version = "2.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "decf2a22ec8fa68f1a0c1129a3f8583f8f8bc24e8b9ccbe98ead99f62a4dc3a8"

[[package]]
name = "icu_locid"
version = "1.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "13acbb8371917fc971be86fc8057c41a64b521c184808a698c02acc242dbf637"
dependencies = [
 "displaydoc",
 "litemap 0.7.5",
 "tinystr 0.7.6",
 "writeable 0.5.5",
]

[[package]]
name = "icu_normalizer"
version = "2.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "12f9cf5f235641ed274641dd81c3f28d870e276763d0797aeeab72317b1c646f"
dependencies = [
 "icu_collections 2.3.0",
 "icu_normalizer_data",
 "icu_properties",
 "icu_provider 2.3.1",
 "smallvec",
 "utf16_iter",
 "write16",
 "zerovec 0.11.8",
]

[[package]]
name = "icu_normalizer_data"
version = "2.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1563da1ed3e0b3bf3d74c9b85917ac9c56464d2f57242270c09c9e752f8021a0"

[[package]]
name = "icu_properties"
version = "2.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7e7ca276ad3145661a65914e6daf131ca5120cd3dcee8f8f3214b8875184a148"
dependencies = [
 "displaydoc",
 "icu_collections 2.3.0",
 "icu_locale_core",
 "icu_properties_data",
 "icu_provider 2.3.1",
 "zerotrie",
 "zerovec 0.11.8",
]

[[package]]
name = "icu_properties_data"
version = "2.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e590f038c1464a96894fd6d10127e90a8be4509f56ff7ecef851b15cee0b7caa"

[[package]]
name = "icu_provider"
version = "1.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6ed421c8a8ef78d3e2dbc98a973be2f3770cb42b606e3ab18d6237c4dfde68d9"
dependencies = [
 "displaydoc",
 "icu_locid",
 "icu_provider_macros",
 "stable_deref_trait",
 "tinystr 0.7.6",
 "writeable 0.5.5",
 "yoke 0.7.5",
 "zerofrom",
 "zerovec 0.10.4",
]

[[package]]
name = "icu_provider"
version = "2.3.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d27bbb9d3abbefac45d55f647c9de1d44aafcd1186eb91879afef17c396c3e73"
dependencies = [
 "displaydoc",
 "icu_locale_core",
 "serde",
 "stable_deref_trait",
 "writeable 0.6.4",
 "yoke 0.8.3",
 "zerofrom",
 "zerotrie",
 "zerovec 0.11.8",
]

[[package]]
name = "icu_provider_macros"
version = "1.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1ec89e9337638ecdc08744df490b221a7399bf8d164eb52a665454e60e075ad6"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "icu_segmenter"
version = "1.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a717725612346ffc2d7b42c94b820db6908048f39434504cb130e8b46256b0de"
dependencies = [
 "core_maths",
 "displaydoc",
 "icu_collections 1.5.0",
 "icu_locid",
 "icu_provider 1.5.0",
 "icu_segmenter_data 1.5.1",
 "utf8_iter",
 "zerovec 0.10.4",
]

[[package]]
name = "icu_segmenter"
version = "2.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "82d07aafccd67af15d02512a6adf5896fbc5ed00f2e99b471d2efa14016db3db"
dependencies = [
 "icu_collections 2.3.0",
 "icu_locale_fallback",
 "icu_provider 2.3.1",
 "icu_segmenter_data 2.3.0",
 "potential_utf",
 "smallvec",
 "utf8_iter",
 "zerovec 0.11.8",
]

[[package]]
name = "icu_segmenter_data"
version = "1.5.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a1e52775179941363cc594e49ce99284d13d6948928d8e72c755f55e98caa1eb"

[[package]]
name = "icu_segmenter_data"
version = "2.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ae293c039020f9ec10710af98d29ce6aa2051486638b49c9a6409f3b4a9e98ad"

[[package]]
name = "idna"
version = "1.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3b0875f23caa03898994f6ddc501886a45c7d3d62d04d2d90788d47be1b1e4de"
dependencies = [
 "idna_adapter",
 "smallvec",
 "utf8_iter",
]

[[package]]
name = "idna_adapter"
version = "1.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cb68373c0d6620ef8105e855e7745e18b0d00d3bdb07fb532e434244cdb9a714"
dependencies = [
 "icu_normalizer",
 "icu_properties",
]

[[package]]
name = "image"
version = "0.25.10"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "85ab80394333c02fe689eaf900ab500fbd0c2213da414687ebf995a65d5a6104"
dependencies = [
 "bytemuck",
 "byteorder-lite",
 "color_quant",
 "exr",
 "gif",
 "image-webp",
 "moxcms",
 "num-traits",
 "png",
 "ravif",
 "rayon",
 "tiff",
 "zune-core",
 "zune-jpeg",
]

[[package]]
name = "image-webp"
version = "0.2.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "525e9ff3e1a4be2fbea1fdf0e98686a6d98b4d8f937e1bf7402245af1909e8c3"
dependencies = [
 "byteorder-lite",
 "quick-error",
]

[[package]]
name = "imgref"
version = "1.12.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6e44b0a4eaa4c82f441d50a963f2d5f05a787240aeee097597033e72accfd22f"

[[package]]
name = "indexmap"
version = "2.14.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cc4e190f5d26ca7051642629da2c52fc03bde85a03197c99408dcd291734c855"
dependencies = [
 "equivalent",
 "hashbrown 0.17.1",
]

[[package]]
name = "inout"
version = "0.1.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "879f10e63c20629ecabbb64a8010319738c66a5cd0c29b02d63d272b03751d01"
dependencies = [
 "generic-array",
]

[[package]]
name = "interpolate_name"
version = "0.2.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c34819042dc3d3971c46c2190835914dfbe0c3c13f61449b2997f4e9722dfa60"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "intrusive-collections"
version = "0.10.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4275b20e6057cd7733fd8df8a5a31701e4fe44497dad0f3fa0e1c4fb971506be"

[[package]]
name = "ipc-channel"
version = "0.23.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cf1d9c442a2ec790209da8a49f99f7dae0fad31ba24c9947b01ce5e8de8bddcb"
dependencies = [
 "crossbeam-channel",
 "libc",
 "mio",
 "postcard",
 "rand 0.10.3",
 "rustc-hash 2.1.3",
 "serde_core",
 "tempfile",
 "thiserror 2.0.21",
 "uuid",
 "windows",
]

[[package]]
name = "itertools"
version = "0.14.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2b192c782037fadd9cfa75548310488aabdbf3d2da73885b31bd0abd03351285"
dependencies = [
 "either",
]

[[package]]
name = "itertools"
version = "0.15.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8b4baf93f58d4425749ca49a51c50ebab072c5df6994d08fed93541c331481dc"
dependencies = [
 "either",
]

[[package]]
name = "itoa"
version = "1.0.18"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8f42a60cbdf9a97f5d2305f08a87dc4e09308d1276d28c869c684d7777685682"

[[package]]
name = "ixdtf"
version = "0.6.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4d3667095d64c3ecffc96463a21157b04bf3e252f6e8d5750b20c02e33c194e3"

[[package]]
name = "jiff-tzdb"
version = "0.1.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "fa8377070c6bae868759445e5a77f66d84f0b72f3a054bfb00e6d038b8282da7"

[[package]]
name = "jni"
version = "0.22.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5efd9a482cf3a427f00d6b35f14332adc7902ce91efb778580e180ff90fa3498"
dependencies = [
 "cfg-if",
 "combine",
 "jni-macros",
 "jni-sys 0.4.1",
 "log",
 "simd_cesu8",
 "thiserror 2.0.21",
 "walkdir",
 "windows-link",
]

[[package]]
name = "jni-macros"
version = "0.22.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a00109accc170f0bdb141fed3e393c565b6f5e072365c3bd58f5b062591560a3"
dependencies = [
 "proc-macro2",
 "quote",
 "rustc_version",
 "simd_cesu8",
 "syn 2.0.119",
]

[[package]]
name = "jni-sys"
version = "0.3.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "41a652e1f9b6e0275df1f15b32661cf0d4b78d4d87ddec5e0c3c20f097433258"
dependencies = [
 "jni-sys 0.4.1",
]

[[package]]
name = "jni-sys"
version = "0.4.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c6377a88cb3910bee9b0fa88d4f42e1d2da8e79915598f65fb0c7ee14c878af2"
dependencies = [
 "jni-sys-macros",
]

[[package]]
name = "jni-sys-macros"
version = "0.4.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "38c0b942f458fe50cdac086d2f946512305e5631e720728f2a61aabcd47a6264"
dependencies = [
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "jobserver"
version = "0.1.35"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1c00acbd29eabad4a2392fa0e921c874934dbbf4194312ad20f04a0ed67a3cb3"
dependencies = [
 "getrandom 0.4.3",
 "libc",
]

[[package]]
name = "js-sys"
version = "0.3.106"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7883d941dae510fb2d978fc3fe018c71c9e2892fd38854de3e8b92c2e5ad9cc5"
dependencies = [
 "cfg-if",
 "futures-util",
 "wasm-bindgen",
]

[[package]]
name = "keyboard-types"
version = "0.8.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0fbe853b403ae61a04233030ae8a79d94975281ed9770a1f9e246732b534b28d"
dependencies = [
 "bitflags 2.13.2",
 "serde",
 "unicode-segmentation",
]

[[package]]
name = "khronos-egl"
version = "6.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6aae1df220ece3c0ada96b8153459b67eebe9ae9212258bb0134ae60416fdf76"
dependencies = [
 "libc",
 "libloading",
 "pkg-config",
]

[[package]]
name = "khronos_api"
version = "3.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e2db585e1d738fc771bf08a151420d3ed193d9d895a36df7f6f8a9456b911ddc"

[[package]]
name = "kstring"
version = "2.0.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7a09b82a7f771ed02dc0dd9b27130a0fa5499fa15ed3027116c1e5e4e591bd9e"
dependencies = [
 "static_assertions",
]

[[package]]
name = "kurbo"
version = "0.13.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4b60dfc32f652b926df6192e55525b16d186c69d47876c3ead4da5cc9f8450e2"
dependencies = [
 "arrayvec",
 "euclid",
 "libm",
 "polycool",
 "smallvec",
]

[[package]]
name = "layout-dom-api"
version = "0.1.1"
dependencies = [
 "markup5ever",
 "serde",
]

[[package]]
name = "lazy_static"
version = "1.5.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "20870f649af7073d53e38067b2a84312175d56ea15217e1b15bc83506ec50afb"

[[package]]
name = "lebe"
version = "0.5.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7a79a3332a6609480d7d0c9eab957bca6b455b91bb84e66d19f5ff66294b85b8"

[[package]]
name = "lexical"
version = "7.0.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1bc8a009b2ff1f419ccc62706f04fe0ca6e67b37460513964a3dfdb919bb37d6"
dependencies = [
 "lexical-core",
]

[[package]]
name = "lexical-core"
version = "1.0.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7d8d125a277f807e55a77304455eb7b1cb52f2b18c143b60e766c120bd64a594"
dependencies = [
 "lexical-parse-float",
 "lexical-parse-integer",
 "lexical-util",
 "lexical-write-float",
 "lexical-write-integer",
]

[[package]]
name = "lexical-parse-float"
version = "1.0.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "52a9f232fbd6f550bc0137dcb5f99ab674071ac2d690ac69704593cb4abbea56"
dependencies = [
 "lexical-parse-integer",
 "lexical-util",
]

[[package]]
name = "lexical-parse-integer"
version = "1.0.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9a7a039f8fb9c19c996cd7b2fcce303c1b2874fe1aca544edc85c4a5f8489b34"
dependencies = [
 "lexical-util",
]

[[package]]
name = "lexical-util"
version = "1.0.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2604dd126bb14f13fb5d1bd6a66155079cb9fa655b37f875b3a742c705dbed17"

[[package]]
name = "lexical-write-float"
version = "1.0.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "50c438c87c013188d415fbabbb1dceb44249ab81664efbd31b14ae55dabb6361"
dependencies = [
 "lexical-util",
 "lexical-write-integer",
]

[[package]]
name = "lexical-write-integer"
version = "1.0.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "409851a618475d2d5796377cad353802345cba92c867d9fbcde9cf4eac4e14df"
dependencies = [
 "lexical-util",
]

[[package]]
name = "libc"
version = "0.2.190"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ce5d3ddc6d3fa000eb1536d85e147bfe31aacaba692ed6a876f95cb7c855be78"

[[package]]
name = "libdeflate-sys"
version = "1.26.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d7870e5fbd2766179a937c725fb11f4ca0ef025d982beb61bd3ce755425bd19c"
dependencies = [
 "cc",
]

[[package]]
name = "libdeflater"
version = "1.26.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0cbb8285d2e2bd4b8d4155840ae81929354b4ba0d7b3c272c84761ffa2141e1f"
dependencies = [
 "libdeflate-sys",
]

[[package]]
name = "libfuzzer-sys"
version = "0.4.13"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a9fd2f41a1cba099f79a0b6b6c35656cf7c03351a7bae8ff0f28f25270f929d2"
dependencies = [
 "arbitrary",
 "cc",
]

[[package]]
name = "libloading"
version = "0.8.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d7c4b02199fee7c5d21a5ae7d8cfa79a6ef5bb2fc834d6e9058e89c825efdc55"
dependencies = [
 "cfg-if",
 "windows-link",
]

[[package]]
name = "libm"
version = "0.2.16"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b6d2cec3eae94f9f509c767b45932f1ada8350c4bdb85af2fcab4a3c14807981"

[[package]]
name = "libredox"
version = "0.1.25"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "61ff90caf6077a803a240f62fdbe88645a890bbca49ef8174c3cb0404362171d"
dependencies = [
 "bitflags 2.13.2",
 "libc",
 "plain",
 "redox_syscall 0.9.4",
]

[[package]]
name = "libz-sys"
version = "1.1.29"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "85bc9657773828b90eeb625adff10eeac83cc21bbfd8e23a03eaa8a33c9e28d9"
dependencies = [
 "cc",
 "libc",
 "pkg-config",
 "vcpkg",
]

[[package]]
name = "linebender_resource_handle"
version = "0.1.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d4a5ff6bcca6c4867b1c4fd4ef63e4db7436ef363e0ad7531d1558856bae64f4"

[[package]]
name = "linux-raw-sys"
version = "0.4.15"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d26c52dbd32dccf2d10cac7725f8eae5296885fb5703b261f7d0a0739ec807ab"

[[package]]
name = "linux-raw-sys"
version = "0.12.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "32a66949e030da00e8c7d4434b251670a91556f4144941d37452769c25d58a53"

[[package]]
name = "litemap"
version = "0.7.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "23fb14cb19457329c82206317a5663005a4d404783dc74f4252769b0d5f42856"

[[package]]
name = "litemap"
version = "0.8.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "47d9d19d1d6efa0109d2f65ff4c85cddd50bd572e5a00127ab10987290bcefae"

[[package]]
name = "litrs"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "11d3d7f243d5c5a8b9bb5d6dd2b1602c0cb0b9db1621bafc7ed66e35ff9fe092"

[[package]]
name = "livery"
version = "0.0.5"
dependencies = [
 "cadency",
 "cssparser",
 "genet-flow",
 "serde",
 "toml",
]

[[package]]
name = "lock_api"
version = "0.4.14"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "224399e74b87b5f3557511d98dff8b14089b3dadafcab6bb93eab67d3aace965"
dependencies = [
 "scopeguard",
]

[[package]]
name = "log"
version = "0.4.34"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f9f8bd3e56ce4dfc153cf470fffbfa98c7620958b312ca5c3a4b8d5181fd13c6"

[[package]]
name = "loop9"
version = "0.1.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0fae87c125b03c1d2c0150c90365d7d6bcc53fb73a9acaef207d2d065860f062"
dependencies = [
 "imgref",
]

[[package]]
name = "lru-slab"
version = "0.1.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4050469837a6ff301cd14c1f8f24f88549e6d548f24f64e2148eb0f72cebc51f"

[[package]]
name = "markup5ever"
version = "0.40.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0ab3dc68ac4a0f5719e560136778c1ee716e296030d75dbd4484e37e39e3a842"
dependencies = [
 "log",
 "tendril",
 "web_atoms",
]

[[package]]
name = "maybe-rayon"
version = "0.1.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8ea1f30cedd69f0a2954655f7188c6a834246d2bcf1e315e2ac40c4b24dc9519"
dependencies = [
 "cfg-if",
 "rayon",
]

[[package]]
name = "md-5"
version = "0.10.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d89e7ee0cfbedfc4da3340218492196241d89eefb6dab27de5df917a6d2e78cf"
dependencies = [
 "cfg-if",
 "digest 0.10.7",
]

[[package]]
name = "memchr"
version = "2.8.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cf8baf1c55e62ffcace7a9f06f4bd9cd3f0c4beb022d3b367256b91b87513d98"

[[package]]
name = "memmap2"
version = "0.9.11"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d1219ed1b7f229ee7104d281dd01d6802fe28bb6e95d292942c4daacdeb798c0"
dependencies = [
 "libc",
]

[[package]]
name = "memoffset"
version = "0.9.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "488016bfae457b036d996092f6cb448677611ce4449e970ceaf42695203f218a"
dependencies = [
 "autocfg",
]

[[package]]
name = "mime"
version = "0.3.17"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6877bb514081ee2a7ff5ef9de3281f14a4dd4bceac4c09388074a6b5df8a139a"

[[package]]
name = "minimal-lexical"
version = "0.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "68354c5c6bd36d73ff3feceb05efa59b6acb7626617f4962be322a825e61f79a"

[[package]]
name = "miniz_oxide"
version = "0.8.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1fa76a2c86f704bdb222d66965fb3d63269ce38518b83cb0575fca855ebb6316"
dependencies = [
 "adler2",
 "simd-adler32",
]

[[package]]
name = "miniz_oxide"
version = "0.9.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b63fbc4a50860e98e7b2aa7804ded1db5cbc3aff9193adaff57a6931bf7c4b4c"
dependencies = [
 "adler2",
 "simd-adler32",
]

[[package]]
name = "mio"
version = "1.2.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1788edb87fdc09c7e26304471e2f5be8cdefb1b6930d6e3985fc02ff53bf86ee"
dependencies = [
 "libc",
 "wasi",
 "windows-sys 0.61.2",
]

[[package]]
name = "mockito"
version = "1.7.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "90820618712cab19cfc46b274c6c22546a82affcb3c3bdf0f29e3db8e1bb92c0"
dependencies = [
 "assert-json-diff",
 "bytes",
 "colored",
 "futures-core",
 "http 1.5.0",
 "http-body",
 "http-body-util",
 "hyper",
 "hyper-util",
 "log",
 "pin-project-lite",
 "rand 0.9.5",
 "regex",
 "serde_json",
 "serde_urlencoded",
 "similar",
 "tokio",
]

[[package]]
name = "moxcms"
version = "0.8.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bb85c154ba489f01b25c0d36ae69a87e4a1c73a72631fc6c0eb6dde34a73e44b"
dependencies = [
 "num-traits",
 "pxfm",
]

[[package]]
name = "muldiv"
version = "1.0.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "956787520e75e9bd233246045d19f42fb73242759cc57fba9611d940ae96d4b0"

[[package]]
name = "multiversion_no_op"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "743fb55ba31b18fb1ecef6bdc9aa2743314978ac084044301a7eee33fb99a20d"

[[package]]
name = "munge"
version = "0.4.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5e17401f259eba956ca16491461b6e8f72913a0a114e39736ce404410f915a0c"
dependencies = [
 "munge_macro",
]

[[package]]
name = "munge_macro"
version = "0.4.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4568f25ccbd45ab5d5603dc34318c1ec56b117531781260002151b8530a9f931"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "murm-rtc"
version = "0.1.0"
source = "git+https://github.com/merely-made/murm.git?rev=c58b70421d31ed9f2288dc44152dcb6d719552a9#c58b70421d31ed9f2288dc44152dcb6d719552a9"
dependencies = [
 "bytes",
 "rtc",
 "thiserror 2.0.21",
]

[[package]]
name = "murm-wire"
version = "0.1.0"
source = "git+https://github.com/merely-made/murm.git?branch=main#ada9d0e2dadf6af8ef9f9ccbb4f93c2b2ec55dce"
dependencies = [
 "bytes",
 "futures-util",
 "h3",
 "h3-quinn",
 "http 1.5.0",
 "http-body-util",
 "hyper",
 "hyper-rustls",
 "hyper-util",
 "quinn",
 "rustls",
 "tokio",
 "tokio-tungstenite",
 "tower-service",
 "url",
 "webpki-roots",
]

[[package]]
name = "naga"
version = "30.0.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a616d2fb8c89516ac2723a581f69d6c18576046bed761bd6b305e5618e6ae130"
dependencies = [
 "arrayvec",
 "bit-set",
 "bitflags 2.13.2",
 "cfg-if",
 "cfg_aliases",
 "codespan-reporting",
 "half",
 "hashbrown 0.17.1",
 "indexmap",
 "libm",
 "log",
 "naga-types",
 "num-traits",
 "once_cell",
 "petgraph 0.8.3",
 "rustc-hash 1.1.0",
 "spirv",
 "thiserror 2.0.21",
 "unicode-ident",
]

[[package]]
name = "naga-types"
version = "30.0.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "590afbf58a6f4f62873cd5cff4468061844bafa1cdf399cc954537c22d768d49"
dependencies = [
 "hashbrown 0.17.1",
 "indexmap",
 "rustc-hash 1.1.0",
 "thiserror 2.0.21",
]

[[package]]
name = "ndk"
version = "0.9.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c3f42e7bbe13d351b6bead8286a43aac9534b82bd3cc43e47037f012ebfd62d4"
dependencies = [
 "bitflags 2.13.2",
 "jni-sys 0.3.1",
 "log",
 "ndk-sys",
 "num_enum",
 "raw-window-handle",
 "thiserror 1.0.69",
]

[[package]]
name = "ndk-context"
version = "0.1.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "27b02d87554356db9e9a873add8782d4ea6e3e58ea071a9adb9a2e8ddb884a8b"

[[package]]
name = "ndk-sys"
version = "0.6.0+11769913"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ee6cda3051665f1fb8d9e08fc35c96d5a244fb1be711a03b71118828afc9a873"
dependencies = [
 "jni-sys 0.3.1",
]

[[package]]
name = "netfetcher"
version = "0.1.1"
dependencies = [
 "async-compression",
 "base64 0.22.1",
 "bytes",
 "cookie 0.18.2",
 "flate2",
 "futures-util",
 "genet-host-api",
 "httpdate",
 "mockito",
 "murm-wire",
 "psl",
 "ring",
 "time",
 "tokio",
 "tokio-tungstenite",
 "tokio-util",
 "tracing",
 "url",
]

[[package]]
name = "netrender"
version = "0.1.2"
source = "git+https://github.com/merely-made/netrender.git?branch=main#d92050146768357ead4e4e42967866be7061262e"
dependencies = [
 "glifo",
 "log",
 "netrender-vello",
 "netrender_device",
 "skrifa",
 "tracing",
 "vello_common",
 "vello_cpu",
 "web-time",
 "wgpu",
]

[[package]]
name = "netrender-vello"
version = "0.11.0"
source = "git+https://github.com/mark-ik/vello?rev=276b46648c08f3d2062e70559afa999842e1873c#276b46648c08f3d2062e70559afa999842e1873c"
dependencies = [
 "bytemuck",
 "futures-intrusive",
 "log",
 "peniko",
 "png",
 "skrifa",
 "static_assertions",
 "thiserror 2.0.21",
 "vello_encoding",
 "vello_shaders",
 "wgpu",
]

[[package]]
name = "netrender_device"
version = "0.1.2"
source = "git+https://github.com/merely-made/netrender.git?branch=main#d92050146768357ead4e4e42967866be7061262e"
dependencies = [
 "log",
 "pollster",
 "web-time",
 "wgpu",
]

[[package]]
name = "new_debug_unreachable"
version = "1.0.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "650eef8c711430f1a879fdd01d4745a7deea475becfb90269c06775983bbf086"

[[package]]
name = "nix"
version = "0.31.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cf20d2fde8ff38632c426f1165ed7436270b44f199fc55284c38276f9db47c3d"
dependencies = [
 "bitflags 2.13.2",
 "cfg-if",
 "cfg_aliases",
 "libc",
 "memoffset",
]

[[package]]
name = "no_std_io2"
version = "0.9.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "418abd1b6d34fbf6cae440dc874771b0525a604428704c76e48b29a5e67b8003"
dependencies = [
 "memchr",
]

[[package]]
name = "nom"
version = "7.1.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d273983c5a657a70a3e8f2a01329822f3b8c8172b73826411a55751e404a0a4a"
dependencies = [
 "memchr",
 "minimal-lexical",
]

[[package]]
name = "nom"
version = "8.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "df9761775871bdef83bee530e60050f7e54b1105350d6884eb0fb4f46c2f9405"
dependencies = [
 "memchr",
]

[[package]]
name = "nonmax"
version = "0.5.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "610a5acd306ec67f907abe5567859a3c693fb9886eb1f012ab8f2a47bef3db51"

[[package]]
name = "noop_proc_macro"
version = "0.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0676bb32a98c1a483ce53e500a81ad9c3d5b3f7c920c28c24e9cb0980d0b5bc8"

[[package]]
name = "nova_vm"
version = "1.0.0"
source = "git+https://github.com/merely-made/vano.git?rev=b1f3d25d4b7c62ebfc4977346f55dc152ca2cc88#b1f3d25d4b7c62ebfc4977346f55dc152ca2cc88"
dependencies = [
 "ahash",
 "ecmascript_atomics",
 "fast-float",
 "getrandom 0.4.3",
 "hashbrown 0.17.1",
 "lexical",
 "num-bigint 0.4.8",
 "num-traits",
 "oxc_allocator",
 "oxc_ast",
 "oxc_diagnostics",
 "oxc_ecmascript",
 "oxc_parser",
 "oxc_regular_expression",
 "oxc_semantic",
 "oxc_span",
 "oxc_syntax",
 "rand 0.10.3",
 "regress 0.11.1 (registry+https://github.com/rust-lang/crates.io-index)",
 "ryu-js",
 "small_string",
 "soavec",
 "soavec_derive",
 "sonic-rs",
 "temporal_rs",
 "unicode-normalization",
 "usdt",
 "wtf8",
 "xsum",
]

[[package]]
name = "num-bigint"
version = "0.4.8"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c89e69e7e0f03bea5ef08013795c25018e101932225a656383bd384495ecc367"
dependencies = [
 "num-integer",
 "num-traits",
]

[[package]]
name = "num-bigint"
version = "0.5.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "93e7820bc0a80a0238e650327316f929ba18d5be054b647490a3a6a339f3e7c0"
dependencies = [
 "num-integer",
 "num-traits",
 "serde",
]

[[package]]
name = "num-complex"
version = "0.4.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "73f88a1307638156682bada9d7604135552957b7818057dcef22705b4d509495"
dependencies = [
 "bytemuck",
 "num-traits",
]

[[package]]
name = "num-conv"
version = "0.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "521739c6d2bac4aa25192232afe6841231376b2b26d4d9fae5ecf8ca5772e441"

[[package]]
name = "num-derive"
version = "0.4.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ed3955f1a9c7c0c15e092f9c887db08b1fc683305fdf6eb6684f22555355e202"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "num-integer"
version = "0.1.47"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7ce2d95d4b3734dc35aa2f45e1aa22cd416814592a4f9d9205e11affd5b8e10b"
dependencies = [
 "num-traits",
]

[[package]]
name = "num-rational"
version = "0.4.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f83d14da390562dca69fc84082e73e548e1ad308d24accdedd2720017cb37824"
dependencies = [
 "num-bigint 0.4.8",
 "num-integer",
 "num-traits",
]

[[package]]
name = "num-traits"
version = "0.2.19"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "071dfc062690e90b734c0b2273ce72ad0ffa95f0c74596bc250dcfd960262841"
dependencies = [
 "autocfg",
 "libm",
]

[[package]]
name = "num_enum"
version = "0.7.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5d0bca838442ec211fa11de3a8b0e0e8f3a4522575b5c4c06ed722e005036f26"
dependencies = [
 "num_enum_derive",
 "rustversion",
]

[[package]]
name = "num_enum_derive"
version = "0.7.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "680998035259dcfcafe653688bf2aa6d3e2dc05e98be6ab46afb089dc84f1df8"
dependencies = [
 "proc-macro-crate",
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "num_threads"
version = "0.1.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5c7398b9c8b70908f6371f47ed36737907c87c52af34c268fed0bf0ceb92ead9"
dependencies = [
 "libc",
]

[[package]]
name = "objc-sys"
version = "0.3.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cdb91bdd390c7ce1a8607f35f3ca7151b65afc0ff5ff3b34fa350f7d7c7e4310"

[[package]]
name = "objc2"
version = "0.5.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "19d5490aaf8f1d7cf7688dfa9b0ce07900e168852c45cd2c03f534dfd27cfd0b"
dependencies = [
 "objc-sys",
 "objc2-encode",
]

[[package]]
name = "objc2"
version = "0.6.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "08849bbd4767dfae9457696856ae1c84fe4e0281bbe4a7abff2d0e06fb7981f8"
dependencies = [
 "objc2-encode",
]

[[package]]
name = "objc2-app-kit"
version = "0.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e4e89ad9e3d7d297152b17d39ed92cd50ca8063a89a9fa569046d41568891eff"
dependencies = [
 "bitflags 2.13.2",
 "block2 0.5.1",
 "libc",
 "objc2 0.5.3",
 "objc2-core-data 0.2.2",
 "objc2-core-image 0.2.2",
 "objc2-foundation 0.2.2",
 "objc2-quartz-core 0.2.2",
]

[[package]]
name = "objc2-app-kit"
version = "0.3.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d49e936b501e5c5bf01fda3a9452ff86dc3ea98ad5f283e1455153142d97518c"
dependencies = [
 "bitflags 2.13.2",
 "block2 0.6.2",
 "libc",
 "objc2 0.6.5",
 "objc2-cloud-kit 0.3.2",
 "objc2-core-data 0.3.2",
 "objc2-core-foundation",
 "objc2-core-graphics",
 "objc2-core-image 0.3.2",
 "objc2-core-text",
 "objc2-core-video",
 "objc2-foundation 0.3.2",
 "objc2-quartz-core 0.3.2",
]

[[package]]
name = "objc2-cloud-kit"
version = "0.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "74dd3b56391c7a0596a295029734d3c1c5e7e510a4cb30245f8221ccea96b009"
dependencies = [
 "bitflags 2.13.2",
 "block2 0.5.1",
 "objc2 0.5.3",
 "objc2-core-location",
 "objc2-foundation 0.2.2",
]

[[package]]
name = "objc2-cloud-kit"
version = "0.3.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "73ad74d880bb43877038da939b7427bba67e9dd42004a18b809ba7d87cee241c"
dependencies = [
 "bitflags 2.13.2",
 "objc2 0.6.5",
 "objc2-foundation 0.3.2",
]

[[package]]
name = "objc2-contacts"
version = "0.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a5ff520e9c33812fd374d8deecef01d4a840e7b41862d849513de77e44aa4889"
dependencies = [
 "block2 0.5.1",
 "objc2 0.5.3",
 "objc2-foundation 0.2.2",
]

[[package]]
name = "objc2-core-data"
version = "0.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "617fbf49e071c178c0b24c080767db52958f716d9eabdf0890523aeae54773ef"
dependencies = [
 "bitflags 2.13.2",
 "block2 0.5.1",
 "objc2 0.5.3",
 "objc2-foundation 0.2.2",
]

[[package]]
name = "objc2-core-data"
version = "0.3.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0b402a653efbb5e82ce4df10683b6b28027616a2715e90009947d50b8dd298fa"
dependencies = [
 "bitflags 2.13.2",
 "objc2 0.6.5",
 "objc2-foundation 0.3.2",
]

[[package]]
name = "objc2-core-foundation"
version = "0.3.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2a180dd8642fa45cdb7dd721cd4c11b1cadd4929ce112ebd8b9f5803cc79d536"
dependencies = [
 "bitflags 2.13.2",
 "block2 0.6.2",
 "dispatch2",
 "libc",
 "objc2 0.6.5",
]

[[package]]
name = "objc2-core-graphics"
version = "0.3.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e022c9d066895efa1345f8e33e584b9f958da2fd4cd116792e15e07e4720a807"
dependencies = [
 "bitflags 2.13.2",
 "dispatch2",
 "objc2 0.6.5",
 "objc2-core-foundation",
 "objc2-io-surface",
]

[[package]]
name = "objc2-core-image"
version = "0.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "55260963a527c99f1819c4f8e3b47fe04f9650694ef348ffd2227e8196d34c80"
dependencies = [
 "block2 0.5.1",
 "objc2 0.5.3",
 "objc2-foundation 0.2.2",
 "objc2-metal 0.2.2",
]

[[package]]
name = "objc2-core-image"
version = "0.3.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e5d563b38d2b97209f8e861173de434bd0214cf020e3423a52624cd1d989f006"
dependencies = [
 "objc2 0.6.5",
 "objc2-foundation 0.3.2",
]

[[package]]
name = "objc2-core-location"
version = "0.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "000cfee34e683244f284252ee206a27953279d370e309649dc3ee317b37e5781"
dependencies = [
 "block2 0.5.1",
 "objc2 0.5.3",
 "objc2-contacts",
 "objc2-foundation 0.2.2",
]

[[package]]
name = "objc2-core-text"
version = "0.3.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0cde0dfb48d25d2b4862161a4d5fcc0e3c24367869ad306b0c9ec0073bfed92d"
dependencies = [
 "bitflags 2.13.2",
 "objc2 0.6.5",
 "objc2-core-foundation",
 "objc2-core-graphics",
]

[[package]]
name = "objc2-core-video"
version = "0.3.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d425caf1df73233f29fd8a5c3e5edbc30d2d4307870f802d18f00d83dc5141a6"
dependencies = [
 "bitflags 2.13.2",
 "objc2 0.6.5",
 "objc2-core-foundation",
 "objc2-core-graphics",
 "objc2-io-surface",
]

[[package]]
name = "objc2-encode"
version = "4.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ef25abbcd74fb2609453eb695bd2f860d389e457f67dc17cafc8b8cbc89d0c33"

[[package]]
name = "objc2-foundation"
version = "0.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0ee638a5da3799329310ad4cfa62fbf045d5f56e3ef5ba4149e7452dcf89d5a8"
dependencies = [
 "bitflags 2.13.2",
 "block2 0.5.1",
 "dispatch",
 "libc",
 "objc2 0.5.3",
]

[[package]]
name = "objc2-foundation"
version = "0.3.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e3e0adef53c21f888deb4fa59fc59f7eb17404926ee8a6f59f5df0fd7f9f3272"
dependencies = [
 "bitflags 2.13.2",
 "block2 0.6.2",
 "libc",
 "objc2 0.6.5",
 "objc2-core-foundation",
]

[[package]]
name = "objc2-io-surface"
version = "0.3.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "180788110936d59bab6bd83b6060ffdfffb3b922ba1396b312ae795e1de9d81d"
dependencies = [
 "bitflags 2.13.2",
 "libc",
 "objc2 0.6.5",
 "objc2-core-foundation",
 "objc2-foundation 0.3.2",
]

[[package]]
name = "objc2-link-presentation"
version = "0.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a1a1ae721c5e35be65f01a03b6d2ac13a54cb4fa70d8a5da293d7b0020261398"
dependencies = [
 "block2 0.5.1",
 "objc2 0.5.3",
 "objc2-app-kit 0.2.2",
 "objc2-foundation 0.2.2",
]

[[package]]
name = "objc2-metal"
version = "0.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "dd0cba1276f6023976a406a14ffa85e1fdd19df6b0f737b063b95f6c8c7aadd6"
dependencies = [
 "bitflags 2.13.2",
 "block2 0.5.1",
 "objc2 0.5.3",
 "objc2-foundation 0.2.2",
]

[[package]]
name = "objc2-metal"
version = "0.3.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a0125f776a10d00af4152d74616409f0d4a2053a6f57fa5b7d6aa2854ac04794"
dependencies = [
 "bitflags 2.13.2",
 "block2 0.6.2",
 "dispatch2",
 "objc2 0.6.5",
 "objc2-core-foundation",
 "objc2-foundation 0.3.2",
 "objc2-io-surface",
]

[[package]]
name = "objc2-quartz-core"
version = "0.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e42bee7bff906b14b167da2bac5efe6b6a07e6f7c0a21a7308d40c960242dc7a"
dependencies = [
 "bitflags 2.13.2",
 "block2 0.5.1",
 "objc2 0.5.3",
 "objc2-foundation 0.2.2",
 "objc2-metal 0.2.2",
]

[[package]]
name = "objc2-quartz-core"
version = "0.3.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "96c1358452b371bf9f104e21ec536d37a650eb10f7ee379fff67d2e08d537f1f"
dependencies = [
 "bitflags 2.13.2",
 "block2 0.6.2",
 "libc",
 "objc2 0.6.5",
 "objc2-core-foundation",
 "objc2-core-graphics",
 "objc2-core-video",
 "objc2-foundation 0.3.2",
 "objc2-metal 0.3.2",
]

[[package]]
name = "objc2-symbols"
version = "0.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0a684efe3dec1b305badae1a28f6555f6ddd3bb2c2267896782858d5a78404dc"
dependencies = [
 "objc2 0.5.3",
 "objc2-foundation 0.2.2",
]

[[package]]
name = "objc2-ui-kit"
version = "0.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b8bb46798b20cd6b91cbd113524c490f1686f4c4e8f49502431415f3512e2b6f"
dependencies = [
 "bitflags 2.13.2",
 "block2 0.5.1",
 "objc2 0.5.3",
 "objc2-cloud-kit 0.2.2",
 "objc2-core-data 0.2.2",
 "objc2-core-image 0.2.2",
 "objc2-core-location",
 "objc2-foundation 0.2.2",
 "objc2-link-presentation",
 "objc2-quartz-core 0.2.2",
 "objc2-symbols",
 "objc2-uniform-type-identifiers",
 "objc2-user-notifications",
]

[[package]]
name = "objc2-uniform-type-identifiers"
version = "0.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "44fa5f9748dbfe1ca6c0b79ad20725a11eca7c2218bceb4b005cb1be26273bfe"
dependencies = [
 "block2 0.5.1",
 "objc2 0.5.3",
 "objc2-foundation 0.2.2",
]

[[package]]
name = "objc2-user-notifications"
version = "0.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "76cfcbf642358e8689af64cee815d139339f3ed8ad05103ed5eaf73db8d84cb3"
dependencies = [
 "bitflags 2.13.2",
 "block2 0.5.1",
 "objc2 0.5.3",
 "objc2-core-location",
 "objc2-foundation 0.2.2",
]

[[package]]
name = "object"
version = "0.39.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2e5a6c098c7a3b6547378093f5cc30bc54fd361ce711e05293a5cc589562739b"
dependencies = [
 "memchr",
]

[[package]]
name = "oid-registry"
version = "0.7.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a8d8034d9489cdaf79228eb9f6a3b8d7bb32ba00d6645ebd48eef4077ceb5bd9"
dependencies = [
 "asn1-rs 0.6.2",
]

[[package]]
name = "oid-registry"
version = "0.8.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "12f40cff3dde1b6087cc5d5f5d4d65712f34016a03ed60e9c08dcc392736b5b7"
dependencies = [
 "asn1-rs 0.7.2",
]

[[package]]
name = "once_cell"
version = "1.21.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9f7c3e4beb33f85d45ae3e3a1792185706c8e16d043238c593331cc7cd313b50"

[[package]]
name = "oneshot"
version = "0.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cfe21416a02c693fb9f980befcb230ecc70b0b3d1cc4abf88b9675c4c1457f0c"

[[package]]
name = "openssl-probe"
version = "0.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7c87def4c32ab89d880effc9e097653c8da5d6ef28e6b539d313baaacfbafcbe"

[[package]]
name = "option-operations"
version = "0.6.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "aca39cf52b03268400c16eeb9b56382ea3c3353409309b63f5c8f0b1faf42754"
dependencies = [
 "pastey 0.2.3",
]

[[package]]
name = "orbclient"
version = "0.3.55"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5df339f526ea9a60e371768d50efc2f2508c7203290731565d1f7a6f71d21747"
dependencies = [
 "libc",
 "libredox",
]

[[package]]
name = "ordered-float"
version = "5.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8c7c9e0d9b23589f26070720bac724174bfec1083e82f7854cdd0267518343c0"
dependencies = [
 "num-traits",
]

[[package]]
name = "ordered-stream"
version = "0.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9aa2b01e1d916879f73a53d01d1d6cee68adbb31d6d9177a8cfce093cced1d50"
dependencies = [
 "futures-core",
 "pin-project-lite",
]

[[package]]
name = "ordermap"
version = "0.3.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a86ed3f5f244b372d6b1a00b72ef7f8876d0bc6a78a4c9985c53614041512063"

[[package]]
name = "ortet"
version = "0.2.0"
dependencies = [
 "accesskit",
 "document-session-api",
 "genet-bole",
 "genet-document-resources",
 "genet-documents",
 "genet-host-api",
 "genet-presenter",
 "genet-runtime",
 "genet-scripted",
 "genet-vano",
 "genet-webgl",
 "genet-webrtc",
 "genet-winit",
 "gloo-timers",
 "js-sys",
 "netfetcher",
 "netrender",
 "png",
 "tokio",
 "url",
 "wasm-bindgen",
 "wasm-bindgen-futures",
 "web-sys",
 "wgpu",
 "winit",
]

[[package]]
name = "owned_ttf_parser"
version = "0.25.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "36820e9051aca1014ddc75770aab4d68bc1e9e632f0f5627c4086bc216fb583b"
dependencies = [
 "ttf-parser",
]

[[package]]
name = "owo-colors"
version = "4.4.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "13c45bb4a6ae1280ec0803b1ef9d3455eb50f01efbbe1447ab020f1d54fba9d8"

[[package]]
name = "oxc-miette"
version = "2.7.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4356a61f2ed4c9b3610245215fbf48970eb277126919f87db9d0efa93a74245c"
dependencies = [
 "cfg-if",
 "owo-colors",
 "oxc-miette-derive",
 "textwrap",
 "thiserror 2.0.21",
 "unicode-segmentation",
 "unicode-width",
]

[[package]]
name = "oxc-miette-derive"
version = "2.7.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b237422b014f8f8fff75bb9379e697d13f8d57551a22c88bebb39f073c1bf696"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "oxc_allocator"
version = "0.124.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7cce9493fc18c7f2b9274baba258555d88cc1fab3ac3c4b293433b4f85ad097b"
dependencies = [
 "allocator-api2",
 "hashbrown 0.16.1",
 "oxc_data_structures",
 "rustc-hash 2.1.3",
]

[[package]]
name = "oxc_ast"
version = "0.124.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "29366258930c55e2578e231995d2079cba12793429454fa892f01d985821a554"
dependencies = [
 "bitflags 2.13.2",
 "oxc_allocator",
 "oxc_ast_macros",
 "oxc_data_structures",
 "oxc_diagnostics",
 "oxc_estree",
 "oxc_regular_expression",
 "oxc_span",
 "oxc_syntax",
]

[[package]]
name = "oxc_ast_macros"
version = "0.124.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "617bf2f55d04db8d6fea9583569c7e4d9052297f76f2f8ae31b1f4ef8bcfd98e"
dependencies = [
 "phf 0.13.1",
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "oxc_ast_visit"
version = "0.124.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4344952280d3e8cbfed93da2775c460bbded12f388404daa662dc0ee731e051f"
dependencies = [
 "oxc_allocator",
 "oxc_ast",
 "oxc_span",
 "oxc_syntax",
]

[[package]]
name = "oxc_data_structures"
version = "0.124.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a3a309fcc491b31039bd2a77d8517278c198f566c284e9a18977dab801c05681"

[[package]]
name = "oxc_diagnostics"
version = "0.124.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a1c0f18571aac10db23d1ab681108102ac735c50142c5418ec8272e1d861219f"
dependencies = [
 "cow-utils",
 "oxc-miette",
 "percent-encoding",
]

[[package]]
name = "oxc_ecmascript"
version = "0.124.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4eaddc891449b4c7d8720714d6939c99fc531054c8f7decba9ffdd1c70a7b67b"
dependencies = [
 "cow-utils",
 "num-bigint 0.4.8",
 "num-traits",
 "oxc_allocator",
 "oxc_ast",
 "oxc_regular_expression",
 "oxc_span",
 "oxc_syntax",
]

[[package]]
name = "oxc_estree"
version = "0.124.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8dd0f39cc6f2014fc1a60a563903c6c6c88f856772d44f390fe876a575bd7c97"

[[package]]
name = "oxc_index"
version = "4.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "eb3e6120999627ec9703025eab7c9f410ebb7e95557632a8902ca48210416c2b"
dependencies = [
 "nonmax",
 "serde",
]

[[package]]
name = "oxc_parser"
version = "0.124.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ecf347b9ba5fd251f215f0c44602fbec98c01ea4cf13ae2682167f33d8a8d0b4"
dependencies = [
 "bitflags 2.13.2",
 "cow-utils",
 "memchr",
 "num-bigint 0.4.8",
 "num-traits",
 "oxc_allocator",
 "oxc_ast",
 "oxc_data_structures",
 "oxc_diagnostics",
 "oxc_ecmascript",
 "oxc_regular_expression",
 "oxc_span",
 "oxc_syntax",
 "rustc-hash 2.1.3",
 "seq-macro",
]

[[package]]
name = "oxc_regular_expression"
version = "0.124.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "922016d2def4d0a2b17c907bda16d6eb20516622ae818eb8662f69b353ba9f20"
dependencies = [
 "bitflags 2.13.2",
 "oxc_allocator",
 "oxc_ast_macros",
 "oxc_diagnostics",
 "oxc_span",
 "phf 0.13.1",
 "rustc-hash 2.1.3",
 "unicode-id-start",
]

[[package]]
name = "oxc_semantic"
version = "0.124.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4cb5b9082935c4b0e076bc9c2add9f335fed1be4e6ae5e580747a33808348318"
dependencies = [
 "itertools 0.14.0",
 "memchr",
 "oxc_allocator",
 "oxc_ast",
 "oxc_ast_visit",
 "oxc_diagnostics",
 "oxc_ecmascript",
 "oxc_index",
 "oxc_span",
 "oxc_syntax",
 "rustc-hash 2.1.3",
 "self_cell",
]

[[package]]
name = "oxc_span"
version = "0.124.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9b4413a552b443c777dd2782bc49e719a20cb36434c9b196e9021259028ca74c"
dependencies = [
 "compact_str",
 "oxc-miette",
 "oxc_allocator",
 "oxc_ast_macros",
 "oxc_estree",
 "oxc_str",
]

[[package]]
name = "oxc_str"
version = "0.124.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "321abe830f84ab9c13ac43eadb625f7e8ccddab6a150732d1e5bf9dde043ef4f"
dependencies = [
 "compact_str",
 "hashbrown 0.16.1",
 "oxc_allocator",
 "oxc_estree",
]

[[package]]
name = "oxc_syntax"
version = "0.124.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "11919498c468e21e0688d6c99e37c15f2825a64cfa2f9a0c99d6f767076011d8"
dependencies = [
 "bitflags 2.13.2",
 "cow-utils",
 "dragonbox_ecma",
 "nonmax",
 "oxc_allocator",
 "oxc_ast_macros",
 "oxc_estree",
 "oxc_index",
 "oxc_span",
 "phf 0.13.1",
 "unicode-id-start",
]

[[package]]
name = "oxipng"
version = "9.1.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "26c613f0f566526a647c7473f6a8556dbce22c91b13485ee4b4ec7ab648e4973"
dependencies = [
 "bitvec",
 "indexmap",
 "libdeflater",
 "log",
 "rgb",
 "rustc-hash 2.1.3",
]

[[package]]
name = "paint_list_api"
version = "0.1.2"
source = "git+https://github.com/merely-made/netrender.git?branch=main#d92050146768357ead4e4e42967866be7061262e"
dependencies = [
 "euclid",
 "serde",
]

[[package]]
name = "paint_list_render"
version = "0.1.2"
source = "git+https://github.com/merely-made/netrender.git?branch=main#d92050146768357ead4e4e42967866be7061262e"
dependencies = [
 "log",
 "netrender",
 "paint_list_api",
]

[[package]]
name = "parking"
version = "2.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f38d5652c16fde515bb1ecef450ab0f6a219d619a7274976324d5e377f7dceba"

[[package]]
name = "parking_lot"
version = "0.12.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "93857453250e3077bd71ff98b6a65ea6621a19bb0f559a85248955ac12c45a1a"
dependencies = [
 "lock_api",
 "parking_lot_core",
]

[[package]]
name = "parking_lot_core"
version = "0.9.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2621685985a2ebf1c516881c026032ac7deafcda1a2c9b7850dc81e3dfcb64c1"
dependencies = [
 "cfg-if",
 "libc",
 "redox_syscall 0.5.18",
 "smallvec",
 "windows-link",
]

[[package]]
name = "parlance"
version = "0.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4b6937eda350acc1a5d05872c3cbf99fe78619c269096e2be3d4a350058639d5"
dependencies = [
 "bytemuck",
]

[[package]]
name = "parley_data"
version = "0.10.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d1d3755e2cc12c0625b0cd2f2773c6a37dd11d1531940c945ade4aeb78f2b145"
dependencies = [
 "icu_properties",
]

[[package]]
name = "paste"
version = "1.0.15"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "57c0d7b74b563b49d38dae00a0c37d4d6de9b432382b2892f0574ddcae73fd0a"

[[package]]
name = "pastey"
version = "0.1.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "35fb2e5f958ec131621fdd531e9fc186ed768cbe395337403ae56c17a74c68ec"

[[package]]
name = "pastey"
version = "0.2.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2ee67f1008b1ba2321834326597b8e186293b049a023cdef258527550b9935b4"

[[package]]
name = "pbkdf2"
version = "0.13.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "112d82ceb8c5bf524d9af484d4e4970c9fd5a0cc15ba14ad93dccd28873b0629"
dependencies = [
 "digest 0.11.3",
 "hmac 0.13.0",
]

[[package]]
name = "pem"
version = "3.0.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1d30c53c26bc5b31a98cd02d20f25a7c8567146caf63ed593a9d87b2775291be"
dependencies = [
 "base64 0.22.1",
 "serde_core",
]

[[package]]
name = "pem"
version = "4.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d354a98a3d1251555de99e8fdd8afda05573c31b82f59063a7b0a29b5527f120"
dependencies = [
 "base64 0.23.1",
 "serde_core",
]

[[package]]
name = "peniko"
version = "0.6.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "839c8299360d2e998bdb106dc0a6cd71dcc5f4df51df1b620361bf50e283cca6"
dependencies = [
 "bytemuck",
 "color",
 "kurbo",
 "linebender_resource_handle",
 "smallvec",
]

[[package]]
name = "percent-encoding"
version = "2.3.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9b4f627cb1b25917193a259e49bdad08f671f8d9708acfd5fe0a8c1455d87220"

[[package]]
name = "pest"
version = "2.9.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "45d3aca230fad2e6f6317ca0a72724338c4960cb97168a85cdee66df4a9a21a8"
dependencies = [
 "memchr",
 "ucd-trie",
]

[[package]]
name = "pest_derive"
version = "2.9.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "284b60557f2c4a2e72ad3f2d34d42685a2fa4a6a61d0d2a10c0ae2a5e916c2cf"
dependencies = [
 "pest",
 "pest_generator",
]

[[package]]
name = "pest_generator"
version = "2.9.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1d9d1f08a115309ee99268cf85e5228e0e56aa9caf8841ec12866b6be07c3109"
dependencies = [
 "pest",
 "pest_meta",
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "pest_meta"
version = "2.9.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ed93ba1a9ffcca32130a5188701c81c0c49cf00d4b7c5007d5148951d743adcb"
dependencies = [
 "pest",
]

[[package]]
name = "petgraph"
version = "0.4.13"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9c3659d1ee90221741f65dd128d9998311b0e40c5d3c23a62445938214abce4f"
dependencies = [
 "fixedbitset 0.1.9",
 "ordermap",
]

[[package]]
name = "petgraph"
version = "0.8.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8701b58ea97060d5e5b155d383a69952a60943f0e6dfe30b04c287beb0b27455"
dependencies = [
 "fixedbitset 0.5.7",
 "hashbrown 0.15.5",
 "indexmap",
]

[[package]]
name = "phf"
version = "0.13.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c1562dc717473dbaa4c1f85a36410e03c047b2e7df7f45ee938fbef64ae7fadf"
dependencies = [
 "phf_macros 0.13.1",
 "phf_shared 0.13.1",
 "serde",
]

[[package]]
name = "phf"
version = "0.14.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "010378780309880b08997fae13be7834dba947d36393bd372f2b1556deb2a2f6"
dependencies = [
 "phf_macros 0.14.0",
 "phf_shared 0.14.0",
 "serde",
]

[[package]]
name = "phf_codegen"
version = "0.14.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "41b585a510fb76fdebead6897982ef2a03a21d8e6cbcca904999742a4afc6ffe"
dependencies = [
 "phf_generator 0.14.0",
 "phf_shared 0.14.0",
]

[[package]]
name = "phf_generator"
version = "0.13.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "135ace3a761e564ec88c03a77317a7c6b80bb7f7135ef2544dbe054243b89737"
dependencies = [
 "fastrand",
 "phf_shared 0.13.1",
]

[[package]]
name = "phf_generator"
version = "0.14.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "aeb62e0959d5a1bebc965f4d15d9e2b7cea002b6b0f5ba8cde6cc26738467100"
dependencies = [
 "fastrand",
 "phf_shared 0.14.0",
]

[[package]]
name = "phf_macros"
version = "0.13.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "812f032b54b1e759ccd5f8b6677695d5268c588701effba24601f6932f8269ef"
dependencies = [
 "phf_generator 0.13.1",
 "phf_shared 0.13.1",
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "phf_macros"
version = "0.14.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5fa8d0ca26d424d27630da600c6624696e7dec8bf7b3b492b383c5dc49e5e085"
dependencies = [
 "phf_generator 0.14.0",
 "phf_shared 0.14.0",
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "phf_shared"
version = "0.13.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e57fef6bc5981e38c2ce2d63bfa546861309f875b8a75f092d1d54ae2d64f266"
dependencies = [
 "siphasher",
]

[[package]]
name = "phf_shared"
version = "0.14.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c6fd9027e2d9319be6349febd1db4e8d02aa544921200c9b777720ac34a3aa89"
dependencies = [
 "siphasher",
]

[[package]]
name = "piccolo"
version = "0.3.3"
source = "git+https://github.com/merely-made/didgeridoo?rev=e77309c648b91d59c1ded9b20e54999ae8b19741#e77309c648b91d59c1ded9b20e54999ae8b19741"
dependencies = [
 "ahash",
 "allocator-api2",
 "anyhow",
 "gc-arena",
 "hashbrown 0.14.5",
 "rand 0.8.8",
 "thiserror 1.0.69",
]

[[package]]
name = "pin-project"
version = "1.1.13"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2466b2336ed02bcdca6b294417127b90ec92038d1d5c4fbeac971a922e0e0924"
dependencies = [
 "pin-project-internal",
]

[[package]]
name = "pin-project-internal"
version = "1.1.13"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c96395f0a926bc13b1c17622aaddda1ecb55d49c8f1bf9777e4d877800a43f8b"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "pin-project-lite"
version = "0.2.17"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a89322df9ebe1c1578d689c92318e070967d1042b512afbe49518723f4e6d5cd"

[[package]]
name = "piper"
version = "0.2.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c835479a4443ded371d6c535cbfd8d31ad92c5d23ae9770a61bc155e4992a3c1"
dependencies = [
 "atomic-waker",
 "fastrand",
 "futures-io",
]

[[package]]
name = "pkg-config"
version = "0.3.34"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f6b464fbc74e149a392436b17d523f769e057cb6877f6a5c4618bc6f11800548"

[[package]]
name = "plain"
version = "0.2.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b4596b6d070b27117e987119b4dac604f3c58cfb0b191112e24771b2faeac1a6"

[[package]]
name = "png"
version = "0.18.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "60769b8b31b2a9f263dae2776c37b1b28ae246943cf719eb6946a1db05128a61"
dependencies = [
 "bitflags 2.13.2",
 "crc32fast",
 "fdeflate",
 "flate2",
 "miniz_oxide 0.8.9",
]

[[package]]
name = "polling"
version = "3.11.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5d0e4f59085d47d8241c88ead0f274e8a0cb551f3625263c05eb8dd897c34218"
dependencies = [
 "cfg-if",
 "concurrent-queue",
 "hermit-abi",
 "pin-project-lite",
 "rustix 1.1.5",
 "windows-sys 0.61.2",
]

[[package]]
name = "pollster"
version = "1.0.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bc6355899e1c9462875b6757c79f3caa011a1fdae12bbb1a2e72dd1f234f8336"

[[package]]
name = "polycool"
version = "0.4.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "50596ddc09eb5ad5f75cacd40209568e66df71baf86e1499a0e99c4cff12a5a6"
dependencies = [
 "arrayvec",
 "libm",
]

[[package]]
name = "portable-atomic"
version = "1.15.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "05c8b63e8d9609db387f0324918f81d68fe27748f084ef092fb35954d0539a85"

[[package]]
name = "portable-atomic-util"
version = "0.2.8"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "10ab3eb7f3becc3a1cbc4f2c6f20267996cfc1a6467a873763411b136a122715"
dependencies = [
 "portable-atomic",
]

[[package]]
name = "postcard"
version = "1.1.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6764c3b5dd454e283a30e6dfe78e9b31096d9e32036b5d1eaac7a6119ccb9a24"
dependencies = [
 "cobs",
 "embedded-io 0.4.0",
 "embedded-io 0.6.1",
 "heapless",
 "serde",
]

[[package]]
name = "potential_utf"
version = "0.1.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d83eb9bc6d8e5cf568e7a1101d60ee05e81ed50ea106026f3d18deeb046d7661"
dependencies = [
 "serde_core",
 "writeable 0.6.4",
 "zerovec 0.11.8",
]

[[package]]
name = "powerfmt"
version = "0.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4a6394b9e965e73d0a289ee54f589087e2c676aedf60885baf52c76b771e4958"

[[package]]
name = "ppv-lite86"
version = "0.2.21"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "85eae3c4ed2f50dcfe72643da4befc30deadb458a9b590d720cde2f2b1e97da9"
dependencies = [
 "zerocopy",
]

[[package]]
name = "precomputed-hash"
version = "0.1.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "925383efa346730478fb4838dbe9137d2a47675ad789c546d150a6e1dd4ab31c"

[[package]]
name = "presser"
version = "0.3.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e8cf8e6a8aa66ce33f63993ffc4ea4271eb5b0530a9002db8455ea6050c77bfa"

[[package]]
name = "pretty-hex"
version = "0.4.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9a65843dfefbafd3c879c683306959a6de478443ffe9c9adf02f5976432402d7"

[[package]]
name = "proc-macro-crate"
version = "3.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e67ba7e9b2b56446f1d419b1d807906278ffa1a658a8a5d8a39dcb1f5a78614f"
dependencies = [
 "toml_edit",
]

[[package]]
name = "proc-macro2"
version = "1.0.107"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "985e7ec9bb745e6ce6535b544d84d6cd6f7ad8bd711c398938ae983b91a766d9"
dependencies = [
 "unicode-ident",
]

[[package]]
name = "profiling"
version = "1.0.18"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3d595e54a326bc53c1c197b32d295e14b169e3cfeaa8dc82b529f947fba6bcf5"
dependencies = [
 "profiling-procmacros",
]

[[package]]
name = "profiling-procmacros"
version = "1.0.18"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4488a4a36b9a4ba6b9334a32a39971f77c1436ec82c38707bce707699cc3bbcb"
dependencies = [
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "psl"
version = "2.1.241"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c7319b480e204d67e759cad7886178335d26324c7c8573c126cc9dfa7350d064"
dependencies = [
 "psl-types",
]

[[package]]
name = "psl-types"
version = "2.0.11"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "33cb294fe86a74cbcf50d4445b37da762029549ebeea341421c7c70370f86cac"

[[package]]
name = "psm"
version = "0.1.32"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4dcd034599e63b970727f70d79e02d62390a4a84f7c6b827c27c46d5ac3fa622"
dependencies = [
 "ar_archive_writer",
 "cc",
]

[[package]]
name = "ptr_meta"
version = "0.3.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "743da816b98c921cdbe8628ef7381b76f25ecf4da599fc80aca90eae7ef70cc0"
dependencies = [
 "ptr_meta_derive",
]

[[package]]
name = "ptr_meta_derive"
version = "0.3.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1c8d9ca532f185d5d4db7a7c9d51420b452168ea1c2b913953281bd6fe1fcbd0"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 3.0.6",
]

[[package]]
name = "pulp"
version = "0.22.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "046aa45b989642ec2e4717c8e72d677b13edd831a4d3b6cf37d9a3e54912496a"
dependencies = [
 "bytemuck",
 "cfg-if",
 "libm",
 "num-complex",
 "paste",
 "pulp-wasm-simd-flag",
 "raw-cpuid",
 "reborrow",
 "version_check",
]

[[package]]
name = "pulp-wasm-simd-flag"
version = "0.1.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1d8f70e07b9c3962945a74e59ca1c511bba65b6419468acc217c457d93f3c740"

[[package]]
name = "pxfm"
version = "0.1.30"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d55d956fa96f5ec02be2e13af0e20391a5aa83d6a074e3ad368959d0fab299ea"

[[package]]
name = "quick-error"
version = "2.0.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a993555f31e5a609f617c12db6250dedcac1b0a85076912c436e6fc9b2c8e6a3"

[[package]]
name = "quick-xml"
version = "0.41.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e660451e55124f798a69a5af3f49ccfbefbd41910eefd25caf2393e1f3473ec1"
dependencies = [
 "memchr",
]

[[package]]
name = "quinn"
version = "0.11.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4051e23e9185c255a7e33ef59cdbca87a22d359052eecd22fc6b901fb37d9d11"
dependencies = [
 "bytes",
 "cfg_aliases",
 "futures-io",
 "pin-project-lite",
 "quinn-proto",
 "quinn-udp",
 "rustc-hash 2.1.3",
 "rustls",
 "socket2",
 "thiserror 2.0.21",
 "tokio",
 "tracing",
 "web-time",
]

[[package]]
name = "quinn-proto"
version = "0.11.19"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0e750cca55fe4f0439a15d0bb529da9651e79993e8e72c61a899a36d462befbe"
dependencies = [
 "bytes",
 "fastbloom",
 "getrandom 0.4.3",
 "lru-slab",
 "rand 0.10.3",
 "rand_pcg",
 "ring",
 "rustc-hash 2.1.3",
 "rustls",
 "rustls-pki-types",
 "rustls-platform-verifier",
 "slab",
 "thiserror 2.0.21",
 "tinyvec",
 "tracing",
 "web-time",
]

[[package]]
name = "quinn-udp"
version = "0.5.16"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "af66907df18639dcf4db56ca65490cabc4b27a97dbadd96f2926cca73298f016"
dependencies = [
 "cfg_aliases",
 "libc",
 "once_cell",
 "socket2",
 "tracing",
 "windows-sys 0.61.2",
]

[[package]]
name = "quote"
version = "1.0.47"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1fbf4db142a473a8d80c26bbf18454ed458bf8d26c8219c331daecfdbd079001"
dependencies = [
 "proc-macro2",
]

[[package]]
name = "r-efi"
version = "5.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "69cdb34c158ceb288df11e18b4bd39de994f6657d83847bdffdbd7f346754b0f"

[[package]]
name = "r-efi"
version = "6.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f8dcc9c7d52a811697d2151c701e0d08956f92b0e24136cf4cf27b57a6a0d9bf"

[[package]]
name = "radium"
version = "0.7.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "dc33ff2d4973d518d823d61aa239014831e521c75da58e3df4840d3f47749d09"

[[package]]
name = "rancor"
version = "0.1.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9b534442d0fcdb55d66f373d9cac6d33b6293a2335bc2136dbd06ce0e87d2572"
dependencies = [
 "ptr_meta",
]

[[package]]
name = "rand"
version = "0.8.8"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e058c7de0b26af77780c769414d6257830bb240f3c38477dbc2c16e5f54d6d4c"
dependencies = [
 "libc",
 "rand_chacha 0.3.1",
 "rand_core 0.6.4",
]

[[package]]
name = "rand"
version = "0.9.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b9ef1d0d795eb7d84685bca4f72f3649f064e6641543d3a8c415898726a57b41"
dependencies = [
 "rand_chacha 0.9.0",
 "rand_core 0.9.5",
]

[[package]]
name = "rand"
version = "0.10.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "65c9fb96cbc91e3478eaae79a69fcd3f1ae4ad052e471fe6732fff548984b4af"
dependencies = [
 "chacha20",
 "getrandom 0.4.3",
 "rand_core 0.10.1",
]

[[package]]
name = "rand_chacha"
version = "0.3.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e6c10a63a0fa32252be49d21e7709d4d4baf8d231c2dbce1eaa8141b9b127d88"
dependencies = [
 "ppv-lite86",
 "rand_core 0.6.4",
]

[[package]]
name = "rand_chacha"
version = "0.9.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d3022b5f1df60f26e1ffddd6c66e8aa15de382ae63b3a0c1bfc0e4d3e3f325cb"
dependencies = [
 "ppv-lite86",
 "rand_core 0.9.5",
]

[[package]]
name = "rand_core"
version = "0.6.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ec0be4795e2f6a28069bec0b5ff3e2ac9bafc99e6a9a7dc3547996c5c816922c"
dependencies = [
 "getrandom 0.2.17",
]

[[package]]
name = "rand_core"
version = "0.9.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "76afc826de14238e6e8c374ddcc1fa19e374fd8dd986b0d2af0d02377261d83c"
dependencies = [
 "getrandom 0.3.4",
]

[[package]]
name = "rand_core"
version = "0.10.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "63b8176103e19a2643978565ca18b50549f6101881c443590420e4dc998a3c69"

[[package]]
name = "rand_pcg"
version = "0.10.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "caa0f4137e1c0a72f4c651489402276c8e8e1cf081f3b0ba156d2cbeef09e86a"
dependencies = [
 "rand_core 0.10.1",
]

[[package]]
name = "range-alloc"
version = "0.1.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ca45419789ae5a7899559e9512e58ca889e41f04f1f2445e9f4b290ceccd1d08"

[[package]]
name = "rav1e"
version = "0.8.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "43b6dd56e85d9483277cde964fd1bdb0428de4fec5ebba7540995639a21cb32b"
dependencies = [
 "aligned-vec",
 "arbitrary",
 "arg_enum_proc_macro",
 "arrayvec",
 "av-scenechange",
 "av1-grain",
 "bitstream-io",
 "built",
 "cfg-if",
 "interpolate_name",
 "itertools 0.14.0",
 "libc",
 "libfuzzer-sys",
 "log",
 "maybe-rayon",
 "new_debug_unreachable",
 "noop_proc_macro",
 "num-derive",
 "num-traits",
 "paste",
 "profiling",
 "rand 0.9.5",
 "rand_chacha 0.9.0",
 "simd_helpers",
 "thiserror 2.0.21",
 "v_frame",
 "wasm-bindgen",
]

[[package]]
name = "ravif"
version = "0.13.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e52310197d971b0f5be7fe6b57530dcd27beb35c1b013f29d66c1ad73fbbcc45"
dependencies = [
 "avif-serialize",
 "imgref",
 "loop9",
 "quick-error",
 "rav1e",
 "rayon",
 "rgb",
]

[[package]]
name = "raw-cpuid"
version = "11.6.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "498cd0dc59d73224351ee52a95fee0f1a617a2eae0e7d9d720cc622c73a54186"
dependencies = [
 "bitflags 2.13.2",
]

[[package]]
name = "raw-window-handle"
version = "0.6.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "20675572f6f24e9e76ef639bc5552774ed45f1c30e2951e1e99c59888861c539"

[[package]]
name = "raw-window-metal"
version = "1.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "40d213455a5f1dc59214213c7330e074ddf8114c9a42411eb890c767357ce135"
dependencies = [
 "objc2 0.6.5",
 "objc2-core-foundation",
 "objc2-foundation 0.3.2",
 "objc2-quartz-core 0.3.2",
]

[[package]]
name = "rayon"
version = "1.12.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "fb39b166781f92d482534ef4b4b1b2568f42613b53e5b6c160e24cfbfa30926d"
dependencies = [
 "either",
 "rayon-core",
]

[[package]]
name = "rayon-core"
version = "1.13.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "22e18b0f0062d30d4230b2e85ff77fdfe4326feb054b9783a3460d8435c8ab91"
dependencies = [
 "crossbeam-deque",
 "crossbeam-utils",
]

[[package]]
name = "rcgen"
version = "0.14.10"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8774e05a7d0de114588e6a28fe7e71694b82614ed569d86d8b389dfbc98b8ad8"
dependencies = [
 "pem 4.0.0",
 "ring",
 "rustls-pki-types",
 "time",
 "x509-parser 0.18.1",
 "yasna",
]

[[package]]
name = "read-fonts"
version = "0.41.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "046a7d674daf459825b32f5062056d6882db0d2f5a479fbd76ccfc870ac18709"
dependencies = [
 "bytemuck",
 "core_maths",
 "font-types",
 "once_cell",
]

[[package]]
name = "reborrow"
version = "0.5.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "03251193000f4bd3b042892be858ee50e8b3719f2b08e5833ac4353724632430"

[[package]]
name = "redox_syscall"
version = "0.4.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4722d768eff46b75989dd134e5c353f0d6296e5aaa3132e776cbdb56be7731aa"
dependencies = [
 "bitflags 1.3.2",
]

[[package]]
name = "redox_syscall"
version = "0.5.18"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ed2bf2547551a7053d6fdfafda3f938979645c44812fbfcda098faae3f1a362d"
dependencies = [
 "bitflags 2.13.2",
]

[[package]]
name = "redox_syscall"
version = "0.9.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "737970939a87c6fa31e7acad13307bccbb017a073b695b6089a2c484f929e20e"
dependencies = [
 "bitflags 2.13.2",
]

[[package]]
name = "ref-cast"
version = "1.0.27"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7e440fb4e4b4147295338efb76001ab9e4efc0e5839df2c47fc5ac2381d365c3"
dependencies = [
 "ref-cast-impl",
]

[[package]]
name = "ref-cast-impl"
version = "1.0.27"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "92ecd8964f8453721699a1ed72037b0db49ce2f5a5138486ee89bed6f67cdf3a"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 3.0.6",
]

[[package]]
name = "regex"
version = "1.13.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f020237b6c8eed93db2e2cb53c00c60a8e1bc73da7d073199a1180401450218d"
dependencies = [
 "aho-corasick",
 "memchr",
 "regex-automata",
 "regex-syntax",
]

[[package]]
name = "regex-automata"
version = "0.4.18"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ad8553b9b26413251cbf30e620595c7a41b3887f03da04579c0e6b0d6a06b4b2"
dependencies = [
 "aho-corasick",
 "memchr",
 "regex-syntax",
]

[[package]]
name = "regex-syntax"
version = "0.8.11"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d6f6ff9a378485b298a5286656da665ba74413d36db0979633275d2e708145d4"

[[package]]
name = "regress"
version = "0.11.1"
dependencies = [
 "hashbrown 0.16.1",
 "memchr",
]

[[package]]
name = "regress"
version = "0.11.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "158a764437582235e3501f683b93a0a6f8d825d04a789dbe5ed30b8799b8908a"
dependencies = [
 "hashbrown 0.16.1",
 "memchr",
]

[[package]]
name = "regress"
version = "0.12.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "32eef8b209c3c1c15dbad02c1f30f9539f00dc7253e0cbcdaae442a50a09d7c1"
dependencies = [
 "memchr",
]

[[package]]
name = "rend"
version = "0.5.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "663ba70707f96e871406fe10d68128412e619b06d1d47cb91c3a4c6501176240"
dependencies = [
 "bytecheck",
]

[[package]]
name = "renderdoc-sys"
version = "1.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "19b30a45b0cd0bcca8037f3d0dc3421eaf95327a17cad11964fb8179b4fc4832"

[[package]]
name = "rgb"
version = "0.8.53"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "47b34b781b31e5d73e9fbc8689c70551fd1ade9a19e3e28cfec8580a79290cc4"
dependencies = [
 "bytemuck",
]

[[package]]
name = "ring"
version = "0.17.14"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a4689e6c2294d81e88dc6261c768b63bc4fcdb852be6d1352498b114f61383b7"
dependencies = [
 "cc",
 "cfg-if",
 "getrandom 0.2.17",
 "libc",
 "untrusted",
 "windows-sys 0.52.0",
]

[[package]]
name = "rkyv"
version = "0.8.18"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d9776093b7ca170454ab1406954f7b7d97a57c51dc6c0642957fb2ef25c2d399"
dependencies = [
 "bytecheck",
 "bytes",
 "hashbrown 0.17.1",
 "indexmap",
 "munge",
 "ptr_meta",
 "rancor",
 "rend",
 "rkyv_derive",
 "tinyvec",
 "uuid",
]

[[package]]
name = "rkyv_derive"
version = "0.8.18"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1c25ef604ac7dd839d44d64648952ea23c97866f124ff671b0ed2cf3ad9bb06e"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 3.0.6",
]

[[package]]
name = "roxmltree"
version = "0.21.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f1964b10c76125c36f8afe190065a4bf9a87bf324842c05701330bba9f1cacbb"
dependencies = [
 "memchr",
]

[[package]]
name = "rspirv"
version = "0.13.0+sdk-1.4.341.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "091dca2e1d6fd3098417b5ec88e77e80d1ba5945750943419dc976858082c296"
dependencies = [
 "rustc-hash 1.1.0",
 "spirv",
]

[[package]]
name = "rtc"
version = "0.21.1"
source = "git+https://github.com/webrtc-rs/rtc?rev=70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38#70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38"
dependencies = [
 "bytes",
 "hex",
 "log",
 "pem 3.0.6",
 "rand 0.10.3",
 "rcgen",
 "rtc-crypto",
 "rtc-datachannel",
 "rtc-dtls",
 "rtc-ice",
 "rtc-interceptor",
 "rtc-mdns",
 "rtc-media",
 "rtc-rtcp",
 "rtc-rtp",
 "rtc-sctp",
 "rtc-sdp",
 "rtc-shared",
 "rtc-srtp",
 "rtc-stun",
 "rtc-turn",
 "rustls",
 "sansio",
 "serde",
 "serde_json",
 "unicase",
 "url",
 "x509-parser 0.16.0",
]

[[package]]
name = "rtc-crypto"
version = "0.21.1"
source = "git+https://github.com/webrtc-rs/rtc?rev=70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38#70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38"
dependencies = [
 "aes",
 "ccm",
 "ctr",
 "hmac 0.12.1",
 "md-5",
 "rand 0.10.3",
 "ring",
 "sha1 0.10.7",
 "subtle",
 "thiserror 2.0.21",
 "zeroize",
]

[[package]]
name = "rtc-datachannel"
version = "0.21.1"
source = "git+https://github.com/webrtc-rs/rtc?rev=70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38#70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38"
dependencies = [
 "bytes",
 "log",
 "rtc-sctp",
 "rtc-shared",
 "sansio",
]

[[package]]
name = "rtc-dtls"
version = "0.21.1"
source = "git+https://github.com/webrtc-rs/rtc?rev=70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38#70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38"
dependencies = [
 "bytecheck",
 "byteorder",
 "bytes",
 "der-parser 9.0.0",
 "log",
 "pem 3.0.6",
 "rcgen",
 "rkyv",
 "rtc-crypto",
 "rtc-shared",
 "rustls",
 "x509-parser 0.16.0",
]

[[package]]
name = "rtc-ice"
version = "0.21.1"
source = "git+https://github.com/webrtc-rs/rtc?rev=70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38#70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38"
dependencies = [
 "bytes",
 "crc",
 "log",
 "rtc-crypto",
 "rtc-mdns",
 "rtc-shared",
 "rtc-stun",
 "sansio",
 "serde",
 "url",
 "uuid",
]

[[package]]
name = "rtc-interceptor"
version = "0.21.1"
source = "git+https://github.com/webrtc-rs/rtc?rev=70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38#70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38"
dependencies = [
 "log",
 "rand 0.10.3",
 "rtc-rtcp",
 "rtc-rtp",
 "rtc-shared",
 "sansio",
]

[[package]]
name = "rtc-mdns"
version = "0.21.1"
source = "git+https://github.com/webrtc-rs/rtc?rev=70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38#70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38"
dependencies = [
 "bytes",
 "log",
 "rtc-shared",
 "sansio",
 "socket2",
]

[[package]]
name = "rtc-media"
version = "0.21.1"
source = "git+https://github.com/webrtc-rs/rtc?rev=70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38#70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38"
dependencies = [
 "byteorder",
 "bytes",
 "rand 0.10.3",
 "rtc-rtp",
 "rtc-shared",
 "thiserror 2.0.21",
]

[[package]]
name = "rtc-rtcp"
version = "0.21.1"
source = "git+https://github.com/webrtc-rs/rtc?rev=70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38#70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38"
dependencies = [
 "bytes",
 "rtc-shared",
]

[[package]]
name = "rtc-rtp"
version = "0.21.1"
source = "git+https://github.com/webrtc-rs/rtc?rev=70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38#70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38"
dependencies = [
 "bytes",
 "memchr",
 "rand 0.10.3",
 "rtc-shared",
 "serde",
]

[[package]]
name = "rtc-sctp"
version = "0.21.1"
source = "git+https://github.com/webrtc-rs/rtc?rev=70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38#70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38"
dependencies = [
 "bytes",
 "crc32c",
 "log",
 "rand 0.10.3",
 "rtc-shared",
 "rustc-hash 2.1.3",
 "slab",
 "thiserror 2.0.21",
]

[[package]]
name = "rtc-sdp"
version = "0.21.1"
source = "git+https://github.com/webrtc-rs/rtc?rev=70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38#70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38"
dependencies = [
 "rand 0.10.3",
 "rtc-shared",
 "url",
]

[[package]]
name = "rtc-shared"
version = "0.21.1"
source = "git+https://github.com/webrtc-rs/rtc?rev=70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38#70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38"
dependencies = [
 "bitflags 1.3.2",
 "bytes",
 "nix",
 "rand 0.10.3",
 "serde",
 "substring",
 "thiserror 2.0.21",
 "url",
 "winapi",
]

[[package]]
name = "rtc-srtp"
version = "0.21.1"
source = "git+https://github.com/webrtc-rs/rtc?rev=70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38#70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38"
dependencies = [
 "byteorder",
 "bytes",
 "rtc-crypto",
 "rtc-rtcp",
 "rtc-rtp",
 "rtc-shared",
]

[[package]]
name = "rtc-stun"
version = "0.21.1"
source = "git+https://github.com/webrtc-rs/rtc?rev=70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38#70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38"
dependencies = [
 "base64 0.22.1",
 "bytes",
 "crc",
 "lazy_static",
 "rand 0.10.3",
 "rtc-crypto",
 "rtc-shared",
 "sansio",
 "url",
]

[[package]]
name = "rtc-turn"
version = "0.21.1"
source = "git+https://github.com/webrtc-rs/rtc?rev=70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38#70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38"
dependencies = [
 "bytes",
 "log",
 "rtc-crypto",
 "rtc-shared",
 "rtc-stun",
 "sansio",
]

[[package]]
name = "rustc-hash"
version = "1.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "08d43f7aa6b08d49f382cde6a7982047c3426db949b1424bc4b7ec9ae12c6ce2"

[[package]]
name = "rustc-hash"
version = "2.1.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6b1e7f9a428571be2dc5bc0505c13fb6bf936822b894ec87abf8a08a4e51742d"

[[package]]
name = "rustc_version"
version = "0.4.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cfcb3a22ef46e85b45de6ee7e79d063319ebb6594faafcf1c225ea92ab6e9b92"
dependencies = [
 "semver",
]

[[package]]
name = "rusticata-macros"
version = "4.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "faf0c4a6ece9950b9abdb62b1cfcf2a68b3b67a10ba445b3bb85be2a293d0632"
dependencies = [
 "nom 7.1.3",
]

[[package]]
name = "rustix"
version = "0.38.44"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "fdb5bc1ae2baa591800df16c9ca78619bf65c0488b41b96ccec5d11220d8c154"
dependencies = [
 "bitflags 2.13.2",
 "errno",
 "libc",
 "linux-raw-sys 0.4.15",
 "windows-sys 0.59.0",
]

[[package]]
name = "rustix"
version = "1.1.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "891efababe418670775f199f0d233d84843c227a0949a883ce15b37c78d6629d"
dependencies = [
 "bitflags 2.13.2",
 "errno",
 "libc",
 "linux-raw-sys 0.12.1",
 "windows-sys 0.61.2",
]

[[package]]
name = "rustls"
version = "0.23.45"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0d41d731c7d2f962d1ccc364cec258de3c0e93b38c2fb3ba97ac74513048d634"
dependencies = [
 "once_cell",
 "ring",
 "rustls-pki-types",
 "rustls-webpki",
 "subtle",
 "zeroize",
]

[[package]]
name = "rustls-native-certs"
version = "0.8.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "dab5152771c58876a2146916e53e35057e1a4dfa2b9df0f0305b07f611fdea4d"
dependencies = [
 "openssl-probe",
 "rustls-pki-types",
 "schannel",
 "security-framework",
]

[[package]]
name = "rustls-pki-types"
version = "1.15.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2f4925028c7eb5d1fcdaf196971378ed9d2c1c4efc7dc5d011256f76c99c0a96"
dependencies = [
 "web-time",
 "zeroize",
]

[[package]]
name = "rustls-platform-verifier"
version = "0.7.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1167586491e2b18b8bfbb293e8180ec17c201c4f076d7cb3070ca964e7598f98"
dependencies = [
 "core-foundation 0.10.1",
 "core-foundation-sys",
 "jni",
 "log",
 "once_cell",
 "rustls",
 "rustls-native-certs",
 "rustls-platform-verifier-android",
 "rustls-webpki",
 "security-framework",
 "security-framework-sys",
 "webpki-root-certs",
 "windows-sys 0.61.2",
]

[[package]]
name = "rustls-platform-verifier-android"
version = "0.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "eec689c0bc40ff2458a5977b6619cb718087084a18e02a131c599b62d05e1a5f"

[[package]]
name = "rustls-webpki"
version = "0.103.15"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f3c3cf1d8b1e7d4927e2d154c3fcb02979afb9939629c62cd9048d4f07b60ac2"
dependencies = [
 "ring",
 "rustls-pki-types",
 "untrusted",
]

[[package]]
name = "rustversion"
version = "1.0.23"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cf54715a573b99ac80df0bc206da022bcd442c974952c7b9720069370852e21f"

[[package]]
name = "ryu"
version = "1.0.23"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9774ba4a74de5f7b1c1451ed6cd5285a32eddb5cccb8cc655a4e50009e06477f"

[[package]]
name = "ryu-js"
version = "1.0.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "04d056b875a9d2e6cb9a61d127afee9ac5999b9f87bcb32079d1318e505be714"

[[package]]
name = "same-file"
version = "1.0.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "93fc1dc3aaa9bfed95e02e6eadabb4baf7e3078b0bd1b4d7b6b0b68378900502"
dependencies = [
 "winapi-util",
]

[[package]]
name = "sansio"
version = "1.0.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c62751faa8bc286982334a082fe125184a29fc89d17775766e4f891b7d726980"

[[package]]
name = "schannel"
version = "0.1.29"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "91c1b7e4904c873ef0710c1f407dde2e6287de2bebc1bbbf7d430bb7cbffd939"
dependencies = [
 "windows-sys 0.61.2",
]

[[package]]
name = "scoped-tls"
version = "1.0.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e1cf6437eb19a8f4a6cc0f7dca544973b0b78843adbfeb3683d1a94a0024a294"

[[package]]
name = "scopeguard"
version = "1.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "94143f37725109f92c262ed2cf5e59bce7498c01bcc1502d7b9afe439a4e9f49"

[[package]]
name = "script-engine-api"
version = "0.1.1"

[[package]]
name = "scroll"
version = "0.13.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c1257cd4248b4132760d6524d6dda4e053bc648c9070b960929bf50cfb1e7add"
dependencies = [
 "scroll_derive",
]

[[package]]
name = "scroll_derive"
version = "0.13.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e1a36a382ed65dbcc0ab47fd5e9a94112417ccd34560a392ef3b7b0f0ec39148"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 3.0.6",
]

[[package]]
name = "sctk-adwaita"
version = "0.10.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b6277f0217056f77f1d8f49f2950ac6c278c0d607c45f5ee99328d792ede24ec"
dependencies = [
 "ab_glyph",
 "log",
 "memmap2",
 "smithay-client-toolkit",
 "tiny-skia",
]

[[package]]
name = "security-framework"
version = "3.7.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b7f4bc775c73d9a02cde8bf7b2ec4c9d12743edf609006c7facc23998404cd1d"
dependencies = [
 "bitflags 2.13.2",
 "core-foundation 0.10.1",
 "core-foundation-sys",
 "libc",
 "security-framework-sys",
]

[[package]]
name = "security-framework-sys"
version = "2.17.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6ce2691df843ecc5d231c0b14ece2acc3efb62c0a398c7e1d875f3983ce020e3"
dependencies = [
 "core-foundation-sys",
 "libc",
]

[[package]]
name = "self_cell"
version = "1.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2ab42ca02749e120097e328d91d415325bdf43b1c72c4c8badf37375fe40a813"

[[package]]
name = "semver"
version = "1.0.28"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8a7852d02fc848982e0c167ef163aaff9cd91dc640ba85e263cb1ce46fae51cd"

[[package]]
name = "seq-macro"
version = "0.3.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1bc711410fbe7399f390ca1c3b60ad0f53f80e95c5eb935e52268a0e2cd49acc"

[[package]]
name = "serde"
version = "1.0.229"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4148590afebada386688f18773da617792bf2ef03ffc1e4cbd2b1d45b023e0ba"
dependencies = [
 "serde_core",
 "serde_derive",
]

[[package]]
name = "serde_core"
version = "1.0.229"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "67dca2c9c51e58a4791a4b1ed58308b39c64224d349a935ab5039aa360942a48"
dependencies = [
 "serde_derive",
]

[[package]]
name = "serde_derive"
version = "1.0.229"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e7a5d71263a5a7d47b41f6b3f06ba276f10cc18b0931f1799f710578e2309348"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 3.0.6",
]

[[package]]
name = "serde_json"
version = "1.0.151"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c841b55ecdae098c80dcae9cf767f6f8a0c2cdb3416bbef72181df4d0fe73f14"
dependencies = [
 "itoa",
 "memchr",
 "serde",
 "serde_core",
 "zmij",
]

[[package]]
name = "serde_repr"
version = "0.1.21"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8d3b1629de253c70a0508c3899572da79ca359fdab27c7920ff00406df418906"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 3.0.6",
]

[[package]]
name = "serde_spanned"
version = "1.1.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4e7523beb55eece201a2356bee0bbca0d1ab466c14c07703b2e0ee6d42cb0c2c"
dependencies = [
 "serde_core",
]

[[package]]
name = "serde_tokenstream"
version = "0.2.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d7c49585c52c01f13c5c2ebb333f14f6885d76daa768d8a037d28017ec538c69"
dependencies = [
 "proc-macro2",
 "quote",
 "serde",
 "syn 2.0.119",
]

[[package]]
name = "serde_urlencoded"
version = "0.7.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d3491c14715ca2294c4d6a88f15e84739788c1d030eed8c110436aafdaa2f3fd"
dependencies = [
 "form_urlencoded",
 "itoa",
 "ryu",
 "serde",
]

[[package]]
name = "servo-media"
version = "0.2.0"
dependencies = [
 "servo-media-audio",
 "servo-media-player",
 "servo-media-streams",
 "servo-media-traits",
 "servo-media-webrtc",
]

[[package]]
name = "servo-media-audio"
version = "0.2.0"
dependencies = [
 "byte-slice-cast",
 "euclid",
 "log",
 "num-complex",
 "num-traits",
 "petgraph 0.4.13",
 "serde",
 "servo-media-derive",
 "servo-media-player",
 "servo-media-streams",
 "servo-media-traits",
 "smallvec",
 "speexdsp-resampler",
]

[[package]]
name = "servo-media-auto"
version = "0.2.0"
dependencies = [
 "servo-media-dummy",
 "servo-media-gstreamer",
]

[[package]]
name = "servo-media-derive"
version = "0.2.0"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "servo-media-dummy"
version = "0.2.0"
dependencies = [
 "ipc-channel",
 "servo-media",
 "servo-media-audio",
 "servo-media-player",
 "servo-media-streams",
 "servo-media-traits",
 "servo-media-webrtc",
]

[[package]]
name = "servo-media-examples"
version = "0.2.0"
dependencies = [
 "euclid",
 "rand 0.9.5",
 "serde",
 "servo-media",
 "servo-media-auto",
 "servo-media-dummy",
]

[[package]]
name = "servo-media-gstreamer"
version = "0.2.0"
dependencies = [
 "byte-slice-cast",
 "glib",
 "glib-sys",
 "gstreamer",
 "gstreamer-app",
 "gstreamer-audio",
 "gstreamer-base",
 "gstreamer-play",
 "gstreamer-sdp",
 "gstreamer-sys",
 "gstreamer-video",
 "gstreamer-webrtc",
 "ipc-channel",
 "log",
 "mime",
 "servo-media",
 "servo-media-audio",
 "servo-media-gstreamer-render",
 "servo-media-gstreamer-render-android",
 "servo-media-gstreamer-render-unix",
 "servo-media-player",
 "servo-media-streams",
 "servo-media-traits",
 "servo-media-webrtc",
 "url",
]

[[package]]
name = "servo-media-gstreamer-render"
version = "0.2.0"
dependencies = [
 "gstreamer",
 "gstreamer-video",
 "servo-media-player",
]

[[package]]
name = "servo-media-gstreamer-render-android"
version = "0.2.0"
dependencies = [
 "glib",
 "gstreamer",
 "gstreamer-gl",
 "gstreamer-gl-egl",
 "gstreamer-video",
 "servo-media-gstreamer-render",
 "servo-media-player",
]

[[package]]
name = "servo-media-gstreamer-render-unix"
version = "0.2.0"
dependencies = [
 "glib",
 "gstreamer",
 "gstreamer-gl",
 "gstreamer-gl-egl",
 "gstreamer-gl-wayland",
 "gstreamer-gl-x11",
 "gstreamer-video",
 "servo-media-gstreamer-render",
 "servo-media-player",
]

[[package]]
name = "servo-media-player"
version = "0.2.0"
dependencies = [
 "ipc-channel",
 "serde",
 "servo-media-streams",
 "servo-media-traits",
]

[[package]]
name = "servo-media-streams"
version = "0.2.0"
dependencies = [
 "uuid",
]

[[package]]
name = "servo-media-traits"
version = "0.2.0"

[[package]]
name = "servo-media-webrtc"
version = "0.2.0"
dependencies = [
 "log",
 "servo-media-streams",
 "uuid",
]

[[package]]
name = "servo-xpath"
version = "0.2.0"
dependencies = [
 "log",
 "markup5ever",
]

[[package]]
name = "sha1"
version = "0.10.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a978451301f4db1d02937a4ab3ccce137717b81826e79b7d49ffe3244a13c3b8"
dependencies = [
 "cfg-if",
 "cpufeatures 0.2.17",
 "digest 0.10.7",
]

[[package]]
name = "sha1"
version = "0.11.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "aacc4cc499359472b4abe1bf11d0b12e688af9a805fa5e3016f9a386dc2d0214"
dependencies = [
 "cfg-if",
 "cpufeatures 0.3.1",
 "digest 0.11.3",
]

[[package]]
name = "sha1_smol"
version = "1.0.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bbfa15b3dddfee50a0fff136974b3e1bde555604ba463834a7eb7deb6417705d"

[[package]]
name = "sha2"
version = "0.11.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "47d7069beb7d6ac7b9acd1039986e73443f24234f41074da099d6f994ac9ad19"
dependencies = [
 "cfg-if",
 "cpufeatures 0.3.1",
 "digest 0.11.3",
]

[[package]]
name = "shlex"
version = "2.0.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f8fadd59c855ef2080decdef8ff161eb6661b86933c9d82e5ba29dc602a55aba"

[[package]]
name = "signal-hook-registry"
version = "1.4.8"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c4db69cba1110affc0e9f7bcd48bbf87b3f4fc7c61fc9155afd4c469eb3d6c1b"
dependencies = [
 "errno",
 "libc",
]

[[package]]
name = "simd-adler32"
version = "0.3.10"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3a219298ac11a56ea9a6d2120044824d6f01aeb034955e7af7bc16858527deea"

[[package]]
name = "simd_cesu8"
version = "1.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "11031e251abf8611c80f460e19dbdeb54a66db918e49c65a7065b46ac7aec520"
dependencies = [
 "rustc_version",
 "simdutf8",
]

[[package]]
name = "simd_helpers"
version = "0.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "95890f873bec569a0362c235787f3aca6e1e887302ba4840839bcc6459c42da6"
dependencies = [
 "quote",
]

[[package]]
name = "simdutf8"
version = "0.1.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e3a9fe34e3e7a50316060351f37187a3f546bce95496156754b601a5fa71b76e"

[[package]]
name = "similar"
version = "2.7.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bbbb5d9659141646ae647b42fe094daf6c6192d1620870b449d9557f748b2daa"

[[package]]
name = "siphasher"
version = "1.0.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "33f4fe9184a62d842c9ef383018f3306d8ba224fd9d836f56d7288308847c256"

[[package]]
name = "skrifa"
version = "0.44.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "819ab7d62b1d3e72d9d9dea5650bac30424f9111364bb94928dbf5ecad1baa68"
dependencies = [
 "bytemuck",
 "core_maths",
 "read-fonts",
]

[[package]]
name = "slab"
version = "0.4.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0c790de23124f9ab44544d7ac05d60440adc586479ce501c1d6d7da3cd8c9cf5"

[[package]]
name = "slotmap"
version = "1.1.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bdd58c3c93c3d278ca835519292445cb4b0d4dc59ccfdf7ceadaab3f8aeb4038"
dependencies = [
 "version_check",
]

[[package]]
name = "small_btree"
version = "0.1.0"
source = "git+https://github.com/merely-made/bole.git?rev=ddf4e31b0d6cf86c11ea8907b0d07e4799d62368#ddf4e31b0d6cf86c11ea8907b0d07e4799d62368"
dependencies = [
 "arrayvec",
]

[[package]]
name = "small_string"
version = "1.0.0"
source = "git+https://github.com/merely-made/vano.git?rev=b1f3d25d4b7c62ebfc4977346f55dc152ca2cc88#b1f3d25d4b7c62ebfc4977346f55dc152ca2cc88"
dependencies = [
 "wtf8",
]

[[package]]
name = "smallvec"
version = "1.16.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f9395f0f0eee849a9b707b2f06bb92a6a422090e2123bb2ef8e87a0e61892a8e"
dependencies = [
 "serde",
]

[[package]]
name = "smawk"
version = "0.3.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e8e2fb0f499abb4d162f2bedad68f5ef91a1682b5a03596ddb67efd37768d100"

[[package]]
name = "smithay-client-toolkit"
version = "0.19.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3457dea1f0eb631b4034d61d4d8c32074caa6cd1ab2d59f2327bd8461e2c0016"
dependencies = [
 "bitflags 2.13.2",
 "calloop",
 "calloop-wayland-source",
 "cursor-icon",
 "libc",
 "log",
 "memmap2",
 "rustix 0.38.44",
 "thiserror 1.0.69",
 "wayland-backend",
 "wayland-client",
 "wayland-csd-frame",
 "wayland-cursor",
 "wayland-protocols",
 "wayland-protocols-wlr",
 "wayland-scanner",
 "xkeysym",
]

[[package]]
name = "smol_str"
version = "0.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "dd538fb6910ac1099850255cf94a94df6551fbdd602454387d0adb2d1ca6dead"
dependencies = [
 "serde",
]

[[package]]
name = "soavec"
version = "0.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ad00ede36e1df152ce49c19d6a7e0b16c9e0cf936d4c1fd114ec0f86eb2aee4e"
dependencies = [
 "soavec_derive",
]

[[package]]
name = "soavec_derive"
version = "0.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bce974f36fa02f85e686e5d729db2eb4e5a2b7290b9c5608b237ee4351a60cd2"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "socket2"
version = "0.6.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c3d1e2c7f27f8d4cb10542a02c49005dbd6e93095799d6f3be745fae9f8fedd4"
dependencies = [
 "libc",
 "windows-sys 0.61.2",
]

[[package]]
name = "sonic-number"
version = "0.1.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d7c350a05f295f922fc153a3af8b443c10db70904a9291a624ddb6058c6110d0"
dependencies = [
 "cfg-if",
]

[[package]]
name = "sonic-rs"
version = "0.5.8"
dependencies = [
 "ahash",
 "bumpalo",
 "bytes",
 "cfg-if",
 "faststr",
 "itoa",
 "ref-cast",
 "serde",
 "simdutf8",
 "sonic-number",
 "sonic-simd",
 "thiserror 2.0.21",
 "zmij",
]

[[package]]
name = "sonic-simd"
version = "0.1.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f99e664ecd2d85a68c87e3c7a3cfe691f647ea9e835de984aba4d54a41f817d4"
dependencies = [
 "cfg-if",
]

[[package]]
name = "speexdsp-resampler"
version = "0.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b72d540d5c565dbe1f891d7e21ceb21d2649508306782f1066989fccb0b363d3"

[[package]]
name = "spin"
version = "0.9.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3763264f6b73151db08c50ff20d7d8a0b8796e021cdea7ceedad07b80155fa0e"
dependencies = [
 "lock_api",
]

[[package]]
name = "spirv"
version = "0.4.0+sdk-1.4.341.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d9571ea910ebd84c86af4b3ed27f9dbdc6ad06f17c5f96146b2b671e2976744f"
dependencies = [
 "bitflags 2.13.2",
]

[[package]]
name = "sptr"
version = "0.3.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3b9b39299b249ad65f3b7e96443bad61c02ca5cd3589f46cb6d610a0fd6c0d6a"

[[package]]
name = "stable_deref_trait"
version = "1.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6ce2be8dc25455e1f91df71bfa12ad37d7af1092ae736f3a6cd0e37bc7810596"

[[package]]
name = "stacker"
version = "0.1.25"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "707f49d46706bacf8a2b00d51dace3f9de527c13eec3778f570c411f89e69967"
dependencies = [
 "cc",
 "cfg-if",
 "libc",
 "psm",
 "windows-sys 0.61.2",
]

[[package]]
name = "static_assertions"
version = "1.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a2eb9349b6444b326872e140eb1cf5e7c522154d69e7a0ffb0fb81c06b37543f"

[[package]]
name = "strict-num"
version = "0.1.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6637bab7722d379c8b41ba849228d680cc12d0a45ba1fa2b48f2a30577a06731"

[[package]]
name = "string_cache"
version = "0.11.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ffa8a5dbe8b3f0bbe29d4c3225daafaeead63afdc1b65fc4c01a1384166038e6"
dependencies = [
 "new_debug_unreachable",
 "parking_lot",
 "phf_shared 0.14.0",
 "precomputed-hash",
]

[[package]]
name = "string_cache_codegen"
version = "0.11.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "928dcdf75e47626b3617a976ec205d9f057584c371c1f23b782129268d0e6edc"
dependencies = [
 "phf_generator 0.14.0",
 "phf_shared 0.14.0",
 "proc-macro2",
 "quote",
]

[[package]]
name = "strum"
version = "0.28.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9628de9b8791db39ceda2b119bbe13134770b56c138ec1d3af810d045c04f9bd"
dependencies = [
 "strum_macros",
]

[[package]]
name = "strum_macros"
version = "0.28.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ab85eea0270ee17587ed4156089e10b9e6880ee688791d45a905f5b1ca36f664"
dependencies = [
 "heck",
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "substring"
version = "1.4.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "42ee6433ecef213b2e72f587ef64a2f5943e7cd16fbd82dbe8bc07486c534c86"
dependencies = [
 "autocfg",
]

[[package]]
name = "subtle"
version = "2.6.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "13c2bddecc57b384dee18652358fb23172facb8a2c51ccc10d74c157bdea3292"

[[package]]
name = "svg_fmt"
version = "0.4.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0193cc4331cfd2f3d2011ef287590868599a2f33c3e69bc22c1a3d3acf9e02fb"

[[package]]
name = "syn"
version = "2.0.119"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "872831b642d1a07999a962a351ed35b955ea2cfc8f3862091e2a240a84f17297"
dependencies = [
 "proc-macro2",
 "quote",
 "unicode-ident",
]

[[package]]
name = "syn"
version = "3.0.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8593e8e72159ed2257d083c7a454a85cbf854f37a0966d8d483aff8c8a3ebcee"
dependencies = [
 "proc-macro2",
 "quote",
 "unicode-ident",
]

[[package]]
name = "synstructure"
version = "0.13.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "728a70f3dbaf5bab7f0c4b1ac8d7ae5ea60a4b5549c8a5914361c99147a709d2"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "synstructure"
version = "0.14.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "901704edd0dfe137f1987838ee4f259e4e063c31371bdb423f7ae38ec6f77f02"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 3.0.6",
]

[[package]]
name = "system-deps"
version = "9.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8a0dae2cbca1f0ff93795713cfe0a4c02f760e3c0b8ae2ab4ec4045808ff429f"
dependencies = [
 "cfg-expr",
 "heck",
 "pkg-config",
 "toml",
 "version-compare",
]

[[package]]
name = "tag_ptr"
version = "0.1.0"
source = "git+https://github.com/merely-made/bole.git?rev=ddf4e31b0d6cf86c11ea8907b0d07e4799d62368#ddf4e31b0d6cf86c11ea8907b0d07e4799d62368"

[[package]]
name = "tap"
version = "1.0.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "55937e1799185b12863d447f42597ed69d9928686b8d88a1df17376a097d8369"

[[package]]
name = "taproot"
version = "0.1.0"
dependencies = [
 "accesskit",
 "document-session-api",
 "genet-livery",
 "genet-render",
 "genet-scripted-dom",
 "layout-dom-api",
]

[[package]]
name = "target-lexicon"
version = "0.13.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "adb6935a6f5c20170eeceb1a3835a49e12e19d792f6dd344ccc76a985ca5a6ca"

[[package]]
name = "tempfile"
version = "3.27.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "32497e9a4c7b38532efcdebeef879707aa9f794296a4f0244f6f69e9bc8574bd"
dependencies = [
 "fastrand",
 "getrandom 0.4.3",
 "once_cell",
 "rustix 1.1.5",
 "windows-sys 0.61.2",
]

[[package]]
name = "temporal_rs"
version = "0.2.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1a978fef907a27736827d336c0bfaab14fb8a7509dd6dc2b43c44e51d59aca5b"
dependencies = [
 "calendrical_calculations",
 "core_maths",
 "iana-time-zone",
 "icu_calendar",
 "icu_locale_core",
 "ixdtf",
 "num-traits",
 "timezone_provider",
 "tinystr 0.8.4",
 "web-time",
 "writeable 0.6.4",
]

[[package]]
name = "tendril"
version = "0.5.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5fed54709c5b3a53d09bb1c113ea4f5ceafd1e772ddcb0030a82e1d56c087b08"
dependencies = [
 "new_debug_unreachable",
]

[[package]]
name = "termcolor"
version = "1.4.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "06794f8f6c5c898b3275aebefa6b8a1cb24cd2c6c79397ab15774837a0bc5755"
dependencies = [
 "winapi-util",
]

[[package]]
name = "textwrap"
version = "0.16.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6ecfad6c3abc80a577f2b91c1e412ee57e7a060d430b553c1b0c940974ebcd49"
dependencies = [
 "icu_segmenter 2.3.0",
 "smawk",
 "unicode-width",
]

[[package]]
name = "thin-vec"
version = "0.2.21"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d6a4b9ba8738cb4a4f399d37e266becfd475e75eb73425b87a05a2f2039ba63e"

[[package]]
name = "thiserror"
version = "1.0.69"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b6aaf5339b578ea85b50e080feb250a3e8ae8cfcdff9a461c9ec2904bc923f52"
dependencies = [
 "thiserror-impl 1.0.69",
]

[[package]]
name = "thiserror"
version = "2.0.21"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "09e52cb86a36cede5cb101bf8908837b3e4c6e5e59fe7fd85c23fb56200d189e"
dependencies = [
 "thiserror-impl 2.0.21",
]

[[package]]
name = "thiserror-impl"
version = "1.0.69"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4fee6c4efc90059e10f81e6d42c60a18f76588c3d74cb83a0b242a2b6c7504c1"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "thiserror-impl"
version = "2.0.21"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "fe5197923287db20a58125f0bc85c062f7f2c892de97b18c356f9efb14b28524"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 3.0.6",
]

[[package]]
name = "thread-id"
version = "5.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2010d27add3f3240c1fef7959f46c814487b216baee662af53be645ba7831c07"
dependencies = [
 "libc",
 "windows-sys 0.61.2",
]

[[package]]
name = "tiff"
version = "0.11.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b63feaf3343d35b6ca4d50483f94843803b0f51634937cc2ec519fc32232bc52"
dependencies = [
 "fax",
 "flate2",
 "half",
 "quick-error",
 "weezl",
 "zune-jpeg",
]

[[package]]
name = "time"
version = "0.3.55"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cdb87b95ec50ddfa440816d227a17b2ccbdda963a316a727fda0fc4334f7d134"
dependencies = [
 "deranged",
 "js-sys",
 "libc",
 "num-conv",
 "num_threads",
 "powerfmt",
 "serde_core",
 "time-core",
 "time-macros",
]

[[package]]
name = "time-core"
version = "0.1.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9e1c906769ad99c88eaa54e728060edef082f8e358ff32030cb7c7d315e81109"

[[package]]
name = "time-macros"
version = "0.2.32"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7e689342a48d2ea927c87ea50cabf8594854bf940e9310208848d680d668ed85"
dependencies = [
 "num-conv",
 "time-core",
]

[[package]]
name = "timezone_provider"
version = "0.2.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "89ec4f4eebb4817fde02a411aedc7ac5f66c89c68c49e5d4b17a8139f077af52"
dependencies = [
 "combine",
 "jiff-tzdb",
 "tinystr 0.8.4",
 "tzif",
 "zerofrom",
 "zerotrie",
 "zerovec 0.11.8",
]

[[package]]
name = "tiny-keccak"
version = "2.0.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2c9d3793400a45f954c52e73d068316d76b6f4e36977e3fcebb13a2721e80237"
dependencies = [
 "crunchy",
]

[[package]]
name = "tiny-skia"
version = "0.11.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "83d13394d44dae3207b52a326c0c85a8bf87f1541f23b0d143811088497b09ab"
dependencies = [
 "arrayref",
 "arrayvec",
 "bytemuck",
 "cfg-if",
 "log",
 "tiny-skia-path",
]

[[package]]
name = "tiny-skia-path"
version = "0.11.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9c9e7fc0c2e86a30b117d0462aa261b72b7a99b7ebd7deb3a14ceda95c5bdc93"
dependencies = [
 "arrayref",
 "bytemuck",
 "strict-num",
]

[[package]]
name = "tinystr"
version = "0.7.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9117f5d4db391c1cf6927e7bea3db74b9a1c1add8f7eda9ffd5364f40f57b82f"
dependencies = [
 "displaydoc",
]

[[package]]
name = "tinystr"
version = "0.8.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b1e27c91459209c2986af3dcf603a5a74a4368754ce37414f59acc971167f643"
dependencies = [
 "displaydoc",
 "serde_core",
 "zerovec 0.11.8",
]

[[package]]
name = "tinyvec"
version = "1.13.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "fd3ca314f692efd6c868f8408f53fe444634a845f96c028b97d35f6a1f79f0ee"

[[package]]
name = "tokio"
version = "1.53.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e95f91fcc7a621e8b030f6aa23c71fe9838ae2fb4d8118b75602a328f5144044"
dependencies = [
 "bytes",
 "libc",
 "mio",
 "parking_lot",
 "pin-project-lite",
 "socket2",
 "tokio-macros",
 "windows-sys 0.61.2",
]

[[package]]
name = "tokio-macros"
version = "2.7.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "78773a2a397f451582ce068015985c33193cf6dea8b74d2a639fe457b2f07b0e"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 3.0.6",
]

[[package]]
name = "tokio-rustls"
version = "0.26.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c9cc2678c2cdd569ef8215e2afd7954ada2ae20b4fdd2c5fe6139a3b02d105db"
dependencies = [
 "rustls",
 "tokio",
]

[[package]]
name = "tokio-tungstenite"
version = "0.30.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "17a073bfed563fa236697a068031408a93cd9522e08abf9933ead3e73411bd71"
dependencies = [
 "futures-util",
 "log",
 "rustls",
 "rustls-native-certs",
 "rustls-pki-types",
 "tokio",
 "tokio-rustls",
 "tungstenite",
]

[[package]]
name = "tokio-util"
version = "0.7.19"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "494815d09bf52b5548659851081238f0ca39ff638363907596da739561c62c52"
dependencies = [
 "bytes",
 "futures-core",
 "futures-sink",
 "libc",
 "pin-project-lite",
 "tokio",
]

[[package]]
name = "toml"
version = "1.1.7+spec-1.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7874cd34dd99040fc24e30923aa64a60a81ff3ddfe2b8129ccc84ba24949deed"
dependencies = [
 "indexmap",
 "serde_core",
 "serde_spanned",
 "toml_datetime",
 "toml_parser",
 "toml_writer",
 "winnow",
]

[[package]]
name = "toml_datetime"
version = "1.1.2+spec-1.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2b86d767906c6c42421dcba507eb9d203e779497710a47782a224bb871653053"
dependencies = [
 "serde_core",
]

[[package]]
name = "toml_edit"
version = "0.25.16+spec-1.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "88f38f3af8bf71841c67a990f4e2b52143b2d0a1a994c72513ff4c14977fc722"
dependencies = [
 "indexmap",
 "toml_datetime",
 "toml_parser",
 "winnow",
]

[[package]]
name = "toml_parser"
version = "1.1.4+spec-1.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bed810ee007d8e7f9cf20b4774e7198142fa27186e0f805871da7f3c39eaf333"
dependencies = [
 "winnow",
]

[[package]]
name = "toml_writer"
version = "1.1.3+spec-1.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "06bdbd8cfc056b8d2e2e85f29b56a3bdbecb527cef81eb39e3e7b98af4652770"

[[package]]
name = "tower-service"
version = "0.3.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8df9b6e13f2d32c91b9bd719c00d1958837bc7dec474d94952798cc8e69eeec3"

[[package]]
name = "tracing"
version = "0.1.44"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "63e71662fa4b2a2c3a26f570f037eb95bb1f85397f3cd8076caed2f026a6d100"
dependencies = [
 "log",
 "pin-project-lite",
 "tracing-attributes",
 "tracing-core",
]

[[package]]
name = "tracing-attributes"
version = "0.1.31"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7490cfa5ec963746568740651ac6781f701c9c5ea257c58e057f3ba8cf69e8da"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "tracing-core"
version = "0.1.36"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "db97caf9d906fbde555dd62fa95ddba9eecfd14cb388e4f491a66d74cd5fb79a"
dependencies = [
 "once_cell",
]

[[package]]
name = "try-lock"
version = "0.2.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e421abadd41a4225275504ea4d6566923418b7f05506fbc9c0fe86ba7396114b"

[[package]]
name = "ttf-parser"
version = "0.25.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d2df906b07856748fa3f6e0ad0cbaa047052d4a7dd609e231c4f72cee8c36f31"

[[package]]
name = "tungstenite"
version = "0.30.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e48ac77174b19c110a50ab2128b24215ac9cb40e0e12e093fb602d175c569d22"
dependencies = [
 "bytes",
 "data-encoding",
 "http 1.5.0",
 "httparse",
 "log",
 "rand 0.10.3",
 "rustls",
 "rustls-pki-types",
 "sha1 0.11.0",
 "thiserror 2.0.21",
]

[[package]]
name = "typenum"
version = "1.20.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b6f5e870be6c3b371b77fe0ee0bafb859fa4964b4404c27de1d380043c4dda20"

[[package]]
name = "tzif"
version = "0.4.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a0376dfa52cce372f3b095010fd064fb850a5d8fbfd5be8b0ffa3d64eeab5a5d"
dependencies = [
 "combine",
]

[[package]]
name = "ucd-trie"
version = "0.1.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2896d95c02a80c6d6a5d6e953d479f5ddf2dfdb6a244441010e373ac0fb88971"

[[package]]
name = "uds_windows"
version = "1.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f2f6fb2847f6742cd76af783a2a2c49e9375d0a111c7bef6f71cd9e738c72d6e"
dependencies = [
 "memoffset",
 "tempfile",
 "windows-sys 0.61.2",
]

[[package]]
name = "unicase"
version = "2.10.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "357cc3acc6a036009fd6c973ed009037c732d60d0b4f6c673e9041497482a28f"

[[package]]
name = "unicode-bidi"
version = "0.3.18"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5c1cb5db39152898a79168971543b1cb5020dff7fe43c8dc468b0885f5e29df5"

[[package]]
name = "unicode-id-start"
version = "1.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0825d2e09dcae814a4273c1e1435979d414ede25b2ee235b1545da6682483765"

[[package]]
name = "unicode-ident"
version = "1.0.26"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d245f478577f809a851594d02313b640fb437e0bb33866753cff937863096954"

[[package]]
name = "unicode-normalization"
version = "0.1.25"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5fd4f6878c9cb28d874b009da9e8d183b5abc80117c40bbd187a1fde336be6e8"
dependencies = [
 "tinyvec",
]

[[package]]
name = "unicode-segmentation"
version = "1.13.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c6f5d3c3b1bf09027a88a6bc961fc00497d651009560b5463668dc81b0fa87a8"

[[package]]
name = "unicode-width"
version = "0.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b4ac048d71ede7ee76d585517add45da530660ef4390e49b098733c6e897f254"

[[package]]
name = "untrusted"
version = "0.9.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8ecb6da28b8a351d773b68d5825ac39017e680750f980f3a1a85cd8dd28a47c1"

[[package]]
name = "url"
version = "2.5.8"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ff67a8a4397373c3ef660812acab3268222035010ab8680ec4215f38ba3d0eed"
dependencies = [
 "form_urlencoded",
 "idna",
 "percent-encoding",
 "serde",
]

[[package]]
name = "usdt"
version = "0.6.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1953f8d8a98ac7883c230963291acb65c7ed6ae3e2e4c99d5c65f4e65cc9db38"
dependencies = [
 "dof",
 "goblin",
 "memmap2",
 "serde",
 "usdt-attr-macro",
 "usdt-impl",
 "usdt-macro",
]

[[package]]
name = "usdt-attr-macro"
version = "0.6.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "55d0d673848744c556fcfe8479f87b6974459106e4c41f38375f6d559bb0ee28"
dependencies = [
 "dtrace-parser",
 "proc-macro2",
 "quote",
 "serde_tokenstream",
 "syn 2.0.119",
 "usdt-impl",
]

[[package]]
name = "usdt-impl"
version = "0.6.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cf0085a93af1ca095d8b1dc8672cc4620fcd1db5dff8d165486067badce05555"
dependencies = [
 "byteorder",
 "dof",
 "dtrace-parser",
 "libc",
 "proc-macro2",
 "quote",
 "serde",
 "serde_json",
 "syn 2.0.119",
 "thiserror 2.0.21",
 "thread-id",
]

[[package]]
name = "usdt-macro"
version = "0.6.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c9bf594f86b676f7e2fd3d523d50f9d0cffecff6c19729ff5dbebe86c4cb8cb2"
dependencies = [
 "dtrace-parser",
 "proc-macro2",
 "quote",
 "serde_tokenstream",
 "syn 2.0.119",
 "usdt-impl",
]

[[package]]
name = "utf16_iter"
version = "1.0.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c8232dd3cdaed5356e0f716d285e4b40b932ac434100fe9b7e0e8e935b9e6246"

[[package]]
name = "utf8_iter"
version = "1.0.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b6c140620e7ffbb22c2dee59cafe6084a59b5ffc27a8859a5f0d494b5d52b6be"

[[package]]
name = "uuid"
version = "1.26.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2ef6dac1e96601b4fb3acccccff2139741fcb757cb9a36089bf5be91cfb285ce"
dependencies = [
 "getrandom 0.4.3",
 "js-sys",
 "serde_core",
 "sha1_smol",
 "wasm-bindgen",
]

[[package]]
name = "v_frame"
version = "0.3.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "666b7727c8875d6ab5db9533418d7c764233ac9c0cff1d469aec8fa127597be2"
dependencies = [
 "aligned-vec",
 "num-traits",
 "wasm-bindgen",
]

[[package]]
name = "vcpkg"
version = "0.2.15"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "accd4ea62f7bb7a82fe23066fb0957d48ef677f6eeb8215f372f52e48bb32426"

[[package]]
name = "vello_common"
version = "0.3.0"
source = "git+https://github.com/mark-ik/vello?rev=276b46648c08f3d2062e70559afa999842e1873c#276b46648c08f3d2062e70559afa999842e1873c"
dependencies = [
 "bytemuck",
 "fearless_simd",
 "guillotiere",
 "log",
 "peniko",
 "smallvec",
 "thiserror 2.0.21",
]

[[package]]
name = "vello_cpu"
version = "0.3.0"
source = "git+https://github.com/mark-ik/vello?rev=276b46648c08f3d2062e70559afa999842e1873c#276b46648c08f3d2062e70559afa999842e1873c"
dependencies = [
 "bytemuck",
 "glifo",
 "hashbrown 0.17.1",
 "log",
 "vello_common",
]

[[package]]
name = "vello_encoding"
version = "0.11.0"
source = "git+https://github.com/mark-ik/vello?rev=276b46648c08f3d2062e70559afa999842e1873c#276b46648c08f3d2062e70559afa999842e1873c"
dependencies = [
 "bytemuck",
 "guillotiere",
 "peniko",
 "skrifa",
 "smallvec",
]

[[package]]
name = "vello_shaders"
version = "0.11.0"
source = "git+https://github.com/mark-ik/vello?rev=276b46648c08f3d2062e70559afa999842e1873c#276b46648c08f3d2062e70559afa999842e1873c"
dependencies = [
 "bytemuck",
 "log",
 "naga",
 "thiserror 2.0.21",
 "vello_encoding",
]

[[package]]
name = "version-compare"
version = "0.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "03c2856837ef78f57382f06b2b8563a2f512f7185d732608fd9176cb3b8edf0e"

[[package]]
name = "version_check"
version = "0.9.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0b928f33d975fc6ad9f86c8f283853ad26bdd5b10b7f1542aa2fa15e2289105a"

[[package]]
name = "walkdir"
version = "2.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "29790946404f91d9c5d06f9874efddea1dc06c5efe94541a7d6863108e3a5e4b"
dependencies = [
 "same-file",
 "winapi-util",
]

[[package]]
name = "want"
version = "0.3.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ec4cdd0dd910afe868b7ef477227d8d538b46b3075031afee8a9f2acb0a2ed0b"
dependencies = [
 "try-lock",
]

[[package]]
name = "wasi"
version = "0.11.1+wasi-snapshot-preview1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ccf3ec651a847eb01de73ccad15eb7d99f80485de043efb2f370cd654f4ea44b"

[[package]]
name = "wasip2"
version = "1.0.4+wasi-0.2.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b67efb37e106e55ce722a510d6b5f9c17f083e5fc79afc2badeb12cc313d9487"
dependencies = [
 "wit-bindgen",
]

[[package]]
name = "wasm-bindgen"
version = "0.2.129"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9bb54f33acc68fd454578d9820b0bde1a1a3d17aa17bb7b6595806d02886d409"
dependencies = [
 "cfg-if",
 "once_cell",
 "rustversion",
 "wasm-bindgen-macro",
 "wasm-bindgen-shared",
]

[[package]]
name = "wasm-bindgen-futures"
version = "0.4.79"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3cbab34de2d982e9b48e18d216d04c4a6f641066ff19ffb699980f591ee3610e"
dependencies = [
 "js-sys",
 "tokio",
 "wasm-bindgen",
]

[[package]]
name = "wasm-bindgen-macro"
version = "0.2.129"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2e29d0c35b16e224a7eeb5cd2d25e3e1968fbd65604117b44d3b789d00ee8535"
dependencies = [
 "quote",
 "wasm-bindgen-macro-support",
]

[[package]]
name = "wasm-bindgen-macro-support"
version = "0.2.129"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6f501a8bc3719dba86ef8ae4728879c08001bea749eb1333ac5b91e040e2a6b7"
dependencies = [
 "bumpalo",
 "proc-macro2",
 "quote",
 "syn 3.0.6",
 "wasm-bindgen-shared",
]

[[package]]
name = "wasm-bindgen-shared"
version = "0.2.129"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "23f0c9c52aa7cd7d77769a4cfe2a9adb1b331f489a41d912ce14513d5ab995c6"
dependencies = [
 "unicode-ident",
]

[[package]]
name = "wayland-backend"
version = "0.3.17"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "38a91b4eaddff87b1cd1074985e3713da4af2c49742d1b356b2c01670a67a078"
dependencies = [
 "cc",
 "downcast-rs",
 "rustix 1.1.5",
 "scoped-tls",
 "smallvec",
 "wayland-sys",
]

[[package]]
name = "wayland-client"
version = "0.31.15"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e3c36a0f861ad76d0901f2800b46321410d9f73f2ea88aac0650d86c32688073"
dependencies = [
 "bitflags 2.13.2",
 "rustix 1.1.5",
 "wayland-backend",
 "wayland-scanner",
]

[[package]]
name = "wayland-csd-frame"
version = "0.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "625c5029dbd43d25e6aa9615e88b829a5cad13b2819c4ae129fdbb7c31ab4c7e"
dependencies = [
 "bitflags 2.13.2",
 "cursor-icon",
 "wayland-backend",
]

[[package]]
name = "wayland-cursor"
version = "0.31.14"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4a52d18780be9b1314328a3de5f930b73d2200112e3849ca6cb11822793fb34d"
dependencies = [
 "rustix 1.1.5",
 "wayland-client",
 "xcursor",
]

[[package]]
name = "wayland-protocols"
version = "0.32.13"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "23d0c813de3daa2ed6520af85a3bd49b0e722a3078506899aa9686fea58dc4b6"
dependencies = [
 "bitflags 2.13.2",
 "wayland-backend",
 "wayland-client",
 "wayland-scanner",
]

[[package]]
name = "wayland-protocols-plasma"
version = "0.3.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2b6d8cf1eb2c1c31ed1f5643c88a6e53538129d4af80030c8cabd1f9fa884d91"
dependencies = [
 "bitflags 2.13.2",
 "wayland-backend",
 "wayland-client",
 "wayland-protocols",
 "wayland-scanner",
]

[[package]]
name = "wayland-protocols-wlr"
version = "0.3.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "eb04e52f7836d7c7976c78ca0250d61e33873c34156a2a1fc9474828ec268234"
dependencies = [
 "bitflags 2.13.2",
 "wayland-backend",
 "wayland-client",
 "wayland-protocols",
 "wayland-scanner",
]

[[package]]
name = "wayland-scanner"
version = "0.31.11"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "338e30461b3a2b67d70eb30a6d89f8e0c93a833e07d2ae89085cd070c4a00ac0"
dependencies = [
 "proc-macro2",
 "quick-xml",
 "quote",
]

[[package]]
name = "wayland-sys"
version = "0.31.11"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d8eab23fefc9e41f8e841df4a9c707e8a8c4ed26e944ef69297184de2785e3be"
dependencies = [
 "dlib",
 "log",
 "once_cell",
 "pkg-config",
]

[[package]]
name = "web-sys"
version = "0.3.106"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "88261b9deccee56594c11a3460c462c41f58d148598fe70ad77070126a68aba4"
dependencies = [
 "js-sys",
 "wasm-bindgen",
]

[[package]]
name = "web-time"
version = "1.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5a6580f308b1fad9207618087a65c04e7a10bc77e02c8e84e9b00dd4b12fa0bb"
dependencies = [
 "js-sys",
 "wasm-bindgen",
]

[[package]]
name = "web_atoms"
version = "0.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7572660c8890448ba236b7376f27e389c6a7e1c70195622faced601f855c0ada"
dependencies = [
 "phf 0.14.0",
 "phf_codegen",
 "string_cache",
 "string_cache_codegen",
]

[[package]]
name = "webdriver"
version = "0.53.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "91d53921e1bef27512fa358179c9a22428d55778d2c2ae3c5c37a52b82ce6e92"
dependencies = [
 "base64 0.22.1",
 "bytes",
 "cookie 0.16.2",
 "http 0.2.12",
 "icu_segmenter 1.5.0",
 "log",
 "serde",
 "serde_derive",
 "serde_json",
 "thiserror 1.0.69",
 "time",
 "url",
]

[[package]]
name = "webpki-root-certs"
version = "1.0.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b96554aa2acc8ccdb7e1c9a58a7a68dd5d13bccc69cd124cb09406db612a1c9b"
dependencies = [
 "rustls-pki-types",
]

[[package]]
name = "webpki-roots"
version = "1.0.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7dcd9d09a39985f5344844e66b0c530a33843579125f23e21e9f0f220850f22a"
dependencies = [
 "rustls-pki-types",
]

[[package]]
name = "weezl"
version = "0.1.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a28ac98ddc8b9274cb41bb4d9d4d5c425b6020c50c46f25559911905610b4a88"

[[package]]
name = "wgpu"
version = "30.0.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "527ccdf43dd5b2e8676eed9984ce00e2bbb0a1b85b70c1969dcb6cd2eb55ab9e"
dependencies = [
 "arrayvec",
 "bitflags 2.13.2",
 "bytemuck",
 "cfg-if",
 "cfg_aliases",
 "document-features",
 "hashbrown 0.17.1",
 "js-sys",
 "log",
 "naga",
 "parking_lot",
 "portable-atomic",
 "profiling",
 "raw-window-handle",
 "smallvec",
 "static_assertions",
 "wasm-bindgen",
 "wasm-bindgen-futures",
 "web-sys",
 "wgpu-core",
 "wgpu-hal",
 "wgpu-types",
]

[[package]]
name = "wgpu-core"
version = "30.0.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "14c018fce9b6270aa203c2fdd56f3cce996713534bd757e4ea58c8560b121f14"
dependencies = [
 "arrayvec",
 "bit-set",
 "bit-vec",
 "bitflags 2.13.2",
 "bytemuck",
 "cfg_aliases",
 "document-features",
 "hashbrown 0.17.1",
 "indexmap",
 "log",
 "naga",
 "naga-types",
 "once_cell",
 "parking_lot",
 "portable-atomic",
 "profiling",
 "raw-window-handle",
 "rustc-hash 1.1.0",
 "smallvec",
 "thiserror 2.0.21",
 "wgpu-core-deps-apple",
 "wgpu-core-deps-emscripten",
 "wgpu-core-deps-windows-linux-android",
 "wgpu-hal",
 "wgpu-naga-bridge",
 "wgpu-types",
]

[[package]]
name = "wgpu-core-deps-apple"
version = "30.0.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "061f3d319a40d39d00b1ecc2c33b89fe21d4e6fe01859df3500a3a8ecccd6b68"
dependencies = [
 "wgpu-hal",
]

[[package]]
name = "wgpu-core-deps-emscripten"
version = "30.0.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d98b86cf4abf524a902dd35f18ca6a3f08fc2ae9847c8f10b48e30491b1f0b86"
dependencies = [
 "wgpu-hal",
]

[[package]]
name = "wgpu-core-deps-windows-linux-android"
version = "30.0.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7586165fd5f6d881cb9ce4bb71f40d6caab2c0f1837e3fc1d9788a197fb6004f"
dependencies = [
 "wgpu-hal",
]

[[package]]
name = "wgpu-hal"
version = "30.0.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b6b7fb58561a792bc237628ba0792e332de418fefe145f13b5ed8201e6d52f58"
dependencies = [
 "android_system_properties",
 "arrayvec",
 "ash",
 "bit-set",
 "bitflags 2.13.2",
 "block2 0.6.2",
 "bytemuck",
 "cfg-if",
 "cfg_aliases",
 "glow",
 "glutin_wgl_sys",
 "gpu-allocator",
 "hashbrown 0.17.1",
 "js-sys",
 "khronos-egl",
 "libc",
 "libloading",
 "log",
 "naga",
 "naga-types",
 "ndk-sys",
 "objc2 0.6.5",
 "objc2-core-foundation",
 "objc2-core-graphics",
 "objc2-foundation 0.3.2",
 "objc2-metal 0.3.2",
 "objc2-quartz-core 0.3.2",
 "once_cell",
 "ordered-float",
 "parking_lot",
 "portable-atomic",
 "portable-atomic-util",
 "profiling",
 "range-alloc",
 "raw-window-handle",
 "raw-window-metal",
 "renderdoc-sys",
 "smallvec",
 "static_assertions",
 "thiserror 2.0.21",
 "wasm-bindgen",
 "wayland-sys",
 "web-sys",
 "wgpu-naga-bridge",
 "wgpu-types",
 "windows",
 "windows-core",
 "windows-result",
]

[[package]]
name = "wgpu-naga-bridge"
version = "30.0.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d2f62e73117bb7a62bfd9c5a5841438a823f6566c6442a808ee269d2d055c081"
dependencies = [
 "naga",
 "wgpu-types",
]

[[package]]
name = "wgpu-types"
version = "30.0.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "99dad6f1fbdbbdb4c278a6508b059d44688f5cebddf78d005a46a31340269286"
dependencies = [
 "bitflags 2.13.2",
 "bytemuck",
 "js-sys",
 "log",
 "naga-types",
 "raw-window-handle",
 "static_assertions",
 "web-sys",
]

[[package]]
name = "winapi"
version = "0.3.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5c839a674fcd7a98952e593242ea400abe93992746761e38641405d28b00f419"
dependencies = [
 "winapi-i686-pc-windows-gnu",
 "winapi-x86_64-pc-windows-gnu",
]

[[package]]
name = "winapi-i686-pc-windows-gnu"
version = "0.4.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ac3b87c63620426dd9b991e5ce0329eff545bccbbb34f3be09ff6fb6ab51b7b6"

[[package]]
name = "winapi-util"
version = "0.1.11"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c2a7b1c03c876122aa43f3020e6c3c3ee5c05081c9a00739faf7503aeba10d22"
dependencies = [
 "windows-sys 0.61.2",
]

[[package]]
name = "winapi-x86_64-pc-windows-gnu"
version = "0.4.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "712e227841d057c1ee1cd2fb22fa7e5a5461ae8e48fa2ca79ec42cfc1931183f"

[[package]]
name = "windows"
version = "0.62.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "527fadee13e0c05939a6a05d5bd6eec6cd2e3dbd648b9f8e447c6518133d8580"
dependencies = [
 "windows-collections",
 "windows-core",
 "windows-future",
 "windows-numerics",
]

[[package]]
name = "windows-collections"
version = "0.3.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "23b2d95af1a8a14a3c7367e1ed4fc9c20e0a26e79551b1454d72583c97cc6610"
dependencies = [
 "windows-core",
]

[[package]]
name = "windows-core"
version = "0.62.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b8e83a14d34d0623b51dce9581199302a221863196a1dde71a7663a4c2be9deb"
dependencies = [
 "windows-implement",
 "windows-interface",
 "windows-link",
 "windows-result",
 "windows-strings",
]

[[package]]
name = "windows-future"
version = "0.3.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e1d6f90251fe18a279739e78025bd6ddc52a7e22f921070ccdc67dde84c605cb"
dependencies = [
 "windows-core",
 "windows-link",
 "windows-threading",
]

[[package]]
name = "windows-implement"
version = "0.60.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "053e2e040ab57b9dc951b72c264860db7eb3b0200ba345b4e4c3b14f67855ddf"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "windows-interface"
version = "0.59.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3f316c4a2570ba26bbec722032c4099d8c8bc095efccdc15688708623367e358"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "windows-link"
version = "0.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f0805222e57f7521d6a62e36fa9163bc891acd422f971defe97d64e70d0a4fe5"

[[package]]
name = "windows-numerics"
version = "0.3.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6e2e40844ac143cdb44aead537bbf727de9b044e107a0f1220392177d15b0f26"
dependencies = [
 "windows-core",
 "windows-link",
]

[[package]]
name = "windows-result"
version = "0.4.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7781fa89eaf60850ac3d2da7af8e5242a5ea78d1a11c49bf2910bb5a73853eb5"
dependencies = [
 "windows-link",
]

[[package]]
name = "windows-strings"
version = "0.5.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7837d08f69c77cf6b07689544538e017c1bfcf57e34b4c0ff58e6c2cd3b37091"
dependencies = [
 "windows-link",
]

[[package]]
name = "windows-sys"
version = "0.52.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "282be5f36a8ce781fad8c8ae18fa3f9beff57ec1b52cb3de0789201425d9a33d"
dependencies = [
 "windows-targets",
]

[[package]]
name = "windows-sys"
version = "0.59.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1e38bc4d79ed67fd075bcc251a1c39b32a1776bbe92e5bef1f0bf1f8c531853b"
dependencies = [
 "windows-targets",
]

[[package]]
name = "windows-sys"
version = "0.61.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ae137229bcbd6cdf0f7b80a31df61766145077ddf49416a728b02cb3921ff3fc"
dependencies = [
 "windows-link",
]

[[package]]
name = "windows-targets"
version = "0.52.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9b724f72796e036ab90c1021d4780d4d3d648aca59e491e6b98e725b84e99973"
dependencies = [
 "windows_aarch64_gnullvm",
 "windows_aarch64_msvc",
 "windows_i686_gnu",
 "windows_i686_gnullvm",
 "windows_i686_msvc",
 "windows_x86_64_gnu",
 "windows_x86_64_gnullvm",
 "windows_x86_64_msvc",
]

[[package]]
name = "windows-threading"
version = "0.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3949bd5b99cafdf1c7ca86b43ca564028dfe27d66958f2470940f73d86d75b37"
dependencies = [
 "windows-link",
]

[[package]]
name = "windows_aarch64_gnullvm"
version = "0.52.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "32a4622180e7a0ec044bb555404c800bc9fd9ec262ec147edd5989ccd0c02cd3"

[[package]]
name = "windows_aarch64_msvc"
version = "0.52.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "09ec2a7bb152e2252b53fa7803150007879548bc709c039df7627cabbd05d469"

[[package]]
name = "windows_i686_gnu"
version = "0.52.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8e9b5ad5ab802e97eb8e295ac6720e509ee4c243f69d781394014ebfe8bbfa0b"

[[package]]
name = "windows_i686_gnullvm"
version = "0.52.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0eee52d38c090b3caa76c563b86c3a4bd71ef1a819287c19d586d7334ae8ed66"

[[package]]
name = "windows_i686_msvc"
version = "0.52.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "240948bc05c5e7c6dabba28bf89d89ffce3e303022809e73deaefe4f6ec56c66"

[[package]]
name = "windows_x86_64_gnu"
version = "0.52.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "147a5c80aabfbf0c7d901cb5895d1de30ef2907eb21fbbab29ca94c5b08b1a78"

[[package]]
name = "windows_x86_64_gnullvm"
version = "0.52.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "24d5b23dc417412679681396f2b49f3de8c1473deb516bd34410872eff51ed0d"

[[package]]
name = "windows_x86_64_msvc"
version = "0.52.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "589f6da84c646204747d1270a2a5661ea66ed1cced2631d546fdfb155959f9ec"

[[package]]
name = "winit"
version = "0.30.13"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a6755fa58a9f8350bd1e472d4c3fcc25f824ec358933bba33306d0b63df5978d"
dependencies = [
 "ahash",
 "android-activity",
 "atomic-waker",
 "bitflags 2.13.2",
 "block2 0.5.1",
 "bytemuck",
 "calloop",
 "cfg_aliases",
 "concurrent-queue",
 "core-foundation 0.9.4",
 "core-graphics",
 "cursor-icon",
 "dpi",
 "js-sys",
 "libc",
 "memmap2",
 "ndk",
 "objc2 0.5.3",
 "objc2-app-kit 0.2.2",
 "objc2-foundation 0.2.2",
 "objc2-ui-kit",
 "orbclient",
 "percent-encoding",
 "pin-project",
 "raw-window-handle",
 "redox_syscall 0.4.1",
 "rustix 0.38.44",
 "sctk-adwaita",
 "smithay-client-toolkit",
 "smol_str",
 "tracing",
 "unicode-segmentation",
 "wasm-bindgen",
 "wasm-bindgen-futures",
 "wayland-backend",
 "wayland-client",
 "wayland-protocols",
 "wayland-protocols-plasma",
 "web-sys",
 "web-time",
 "windows-sys 0.52.0",
 "x11-dl",
 "x11rb",
 "xkbcommon-dl",
]

[[package]]
name = "winnow"
version = "1.0.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "23b97319f7b8343df12cc98938e5c3eb436064524c8d2b4e30a1d3a36eecdf81"
dependencies = [
 "memchr",
]

[[package]]
name = "wit-bindgen"
version = "0.57.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1ebf944e87a7c253233ad6766e082e3cd714b5d03812acc24c318f549614536e"

[[package]]
name = "write16"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d1890f4022759daae28ed4fe62859b1236caebfc61ede2f63ed4e695f3f6d936"

[[package]]
name = "writeable"
version = "0.5.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1e9df38ee2d2c3c5948ea468a8406ff0db0b29ae1ffde1bcf20ef305bcc95c51"

[[package]]
name = "writeable"
version = "0.6.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3ad82d2a33cdc9674dc7465672f271e096168fcdbe0f799d9e6db8c5892679dc"

[[package]]
name = "wtf8"
version = "0.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c01ae8492c38f52376efd3a17d0994b6bcf3df1e39c0226d458b7d81670b2a06"

[[package]]
name = "wuff"
version = "0.2.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "200a7b806377b65f1ff9a985874332f15357c87e5dd7855f35f06dade95b8b3d"
dependencies = [
 "brotli-decompressor 5.0.3",
 "bytes",
]

[[package]]
name = "wuff-capi"
version = "0.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d4dbb1eebaa6782421e6aeac4ab5f9f3b9c665eb2f8650073f5235408b3898e2"
dependencies = [
 "wuff",
]

[[package]]
name = "wyz"
version = "0.5.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "05f360fc0b24296329c78fda852a1e9ae82de9cf7b27dae4b7f62f118f77b9ed"
dependencies = [
 "tap",
]

[[package]]
name = "x11-dl"
version = "2.21.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "38735924fedd5314a6e548792904ed8c6de6636285cb9fec04d5b1db85c1516f"
dependencies = [
 "libc",
 "once_cell",
 "pkg-config",
]

[[package]]
name = "x11rb"
version = "0.13.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9993aa5be5a26815fe2c3eacfc1fde061fc1a1f094bf1ad2a18bf9c495dd7414"
dependencies = [
 "as-raw-xcb-connection",
 "gethostname",
 "libc",
 "libloading",
 "once_cell",
 "rustix 1.1.5",
 "x11rb-protocol",
]

[[package]]
name = "x11rb-protocol"
version = "0.13.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ea6fc2961e4ef194dcbfe56bb845534d0dc8098940c7e5c012a258bfec6701bd"

[[package]]
name = "x509-parser"
version = "0.16.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "fcbc162f30700d6f3f82a24bf7cc62ffe7caea42c0b2cba8bf7f3ae50cf51f69"
dependencies = [
 "asn1-rs 0.6.2",
 "data-encoding",
 "der-parser 9.0.0",
 "lazy_static",
 "nom 7.1.3",
 "oid-registry 0.7.1",
 "rusticata-macros",
 "thiserror 1.0.69",
 "time",
]

[[package]]
name = "x509-parser"
version = "0.18.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d43b0f71ce057da06bc0851b23ee24f3f86190b07203dd8f567d0b706a185202"
dependencies = [
 "asn1-rs 0.7.2",
 "data-encoding",
 "der-parser 10.0.0",
 "lazy_static",
 "nom 7.1.3",
 "oid-registry 0.8.1",
 "ring",
 "rusticata-macros",
 "thiserror 2.0.21",
 "time",
]

[[package]]
name = "xcursor"
version = "0.3.11"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "163b33ed8786455e2fa5d72f554057ce3f3182425434f756cd39c99839d88e23"

[[package]]
name = "xkbcommon-dl"
version = "0.4.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d039de8032a9a8856a6be89cea3e5d12fdd82306ab7c94d74e6deab2460651c5"
dependencies = [
 "bitflags 2.13.2",
 "dlib",
 "log",
 "once_cell",
 "xkeysym",
]

[[package]]
name = "xkeysym"
version = "0.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b9cc00251562a284751c9973bace760d86c0276c471b4be569fe6b068ee97a56"

[[package]]
name = "xml-rs"
version = "0.8.29"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e450f9b2ed1dff33c94c12589a87338689467b9c4f5d8a5710bd09a847d2c8a7"

[[package]]
name = "xml5ever"
version = "0.40.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "73290529dc9bc6c6155bf77fdd70ce0061f98ad387586aa3104cddab29206961"
dependencies = [
 "log",
 "markup5ever",
]

[[package]]
name = "xsum"
version = "0.1.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0637d3a5566a82fa5214bae89087bc8c9fb94cd8e8a3c07feb691bb8d9c632db"

[[package]]
name = "y4m"
version = "0.8.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7a5a4b21e1a62b67a2970e6831bc091d7b87e119e7f9791aef9702e3bef04448"

[[package]]
name = "yasna"
version = "0.6.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b5f6765e852b9b4dc8e2a76843e4d64d1cea8e79bcde0b6901aea8e7c7f08282"
dependencies = [
 "bit-vec",
 "time",
]

[[package]]
name = "yeslogic-fontconfig-sys"
version = "6.0.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1d8b8abf912b9a29ff112e1671c97c33636903d13a69712037190e6805af4f76"
dependencies = [
 "dlib",
 "once_cell",
 "pkg-config",
]

[[package]]
name = "yoke"
version = "0.7.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "120e6aef9aa629e3d4f52dc8cc43a015c7724194c97dfaf45180d2daf2b77f40"
dependencies = [
 "serde",
 "stable_deref_trait",
 "yoke-derive 0.7.5",
 "zerofrom",
]

[[package]]
name = "yoke"
version = "0.8.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "709fe23a0424b6a435d82152b1bd3fdfb0833487d5fa90d05d42762a9891fef5"
dependencies = [
 "stable_deref_trait",
 "yoke-derive 0.8.4",
 "zerofrom",
]

[[package]]
name = "yoke-derive"
version = "0.7.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2380878cad4ac9aac1e2435f3eb4020e8374b5f13c296cb75b4620ff8e229154"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 2.0.119",
 "synstructure 0.13.2",
]

[[package]]
name = "yoke-derive"
version = "0.8.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ec8ebde2db3681e8c9980cc27822030e68752690ddfa9473e739aeb4dbde6d71"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 3.0.6",
 "synstructure 0.14.0",
]

[[package]]
name = "zbus"
version = "5.19.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5db4be7c075cb421e4b7ee645541604239bd243ba7c357511f4ff3a74b555907"
dependencies = [
 "async-broadcast",
 "async-executor",
 "async-io",
 "async-lock",
 "async-process",
 "async-recursion",
 "async-task",
 "async-trait",
 "blocking",
 "enumflags2",
 "event-listener",
 "futures-core",
 "futures-lite",
 "hex",
 "libc",
 "ordered-stream",
 "rustix 1.1.5",
 "serde",
 "serde_repr",
 "tracing",
 "uds_windows",
 "uuid",
 "windows-sys 0.61.2",
 "winnow",
 "zbus_macros",
 "zbus_names",
 "zvariant",
]

[[package]]
name = "zbus-lockstep"
version = "0.5.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6998de05217a084b7578728a9443d04ea4cd80f2a0839b8d78770b76ccd45863"
dependencies = [
 "zbus_xml",
 "zvariant",
]

[[package]]
name = "zbus-lockstep-macros"
version = "0.5.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "10da05367f3a7b7553c8cdf8fa91aee6b64afebe32b51c95177957efc47ca3a0"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 2.0.119",
 "zbus-lockstep",
 "zbus_xml",
 "zvariant",
]

[[package]]
name = "zbus_macros"
version = "5.19.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2990635d09ade6df1868f72f8cac69a876a90981e8bd3c40b1be413f8dc88f40"
dependencies = [
 "proc-macro-crate",
 "proc-macro2",
 "quote",
 "syn 3.0.6",
 "zbus_names",
 "zvariant",
 "zvariant_utils",
]

[[package]]
name = "zbus_names"
version = "4.3.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d8bf88b4a3ff53e883001e0e0115b297a9d53c31b9c1edd2bfdd853e3428624e"
dependencies = [
 "serde",
 "winnow",
 "zvariant",
]

[[package]]
name = "zbus_xml"
version = "5.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d1586c021a01ca0a9216dcd874e546382e156a5cbab5fab6cb5f10087e22682a"
dependencies = [
 "serde",
 "winnow",
 "zbus_names",
 "zvariant",
]

[[package]]
name = "zcheapstr"
version = "1.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d1afec51604565183aeb5c54c20aeab286120d4e4460f7f76e3e8bb8c0d99473"
dependencies = [
 "serde",
]

[[package]]
name = "zerocopy"
version = "0.8.62"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "86502bf56ac7c77571a32e2647bb2a15894565e981fb2a48d7bde2d91c965a9d"
dependencies = [
 "zerocopy-derive",
]

[[package]]
name = "zerocopy-derive"
version = "0.8.62"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5457206954b06561e2608c7e19cf58b1926586d999c246eebe4502f7e2039d1a"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "zerofrom"
version = "0.1.8"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0ec05a11813ea801ff6d75110ad09cd0824ddba17dfe17128ea0d5f68e6c5272"
dependencies = [
 "zerofrom-derive",
]

[[package]]
name = "zerofrom-derive"
version = "0.1.8"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f75b4683f6c7f45248d4d64056a24298c6281e0993356d7d1b4a1a962ef10d4a"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 3.0.6",
 "synstructure 0.14.0",
]

[[package]]
name = "zeroize"
version = "1.9.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e13084392c5e4bc371903e2935a5eaeed24905a7511356b883835e18a78f6879"

[[package]]
name = "zerotrie"
version = "0.2.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4ea269c3bd32f0a32c321907a2ae912ba6f4649bb0fc764a15627e99a7095a3f"
dependencies = [
 "displaydoc",
 "yoke 0.8.3",
 "zerofrom",
 "zerovec 0.11.8",
]

[[package]]
name = "zerovec"
version = "0.10.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "aa2b893d79df23bfb12d5461018d408ea19dfafe76c2c7ef6d4eba614f8ff079"
dependencies = [
 "yoke 0.7.5",
 "zerofrom",
 "zerovec-derive 0.10.4",
]

[[package]]
name = "zerovec"
version = "0.11.8"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bb0464e17806c1d976d5cba29399c7f08e516e279e2ba493f63123b5fca67dd8"
dependencies = [
 "serde",
 "yoke 0.8.3",
 "zerofrom",
 "zerovec-derive 0.11.6",
]

[[package]]
name = "zerovec-derive"
version = "0.10.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3e3c6377872d72510393f688a555d7097b0f741995c7a00f0407f786dd486b2d"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 2.0.119",
]

[[package]]
name = "zerovec-derive"
version = "0.11.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "34df6fc39dbd26ddc9c10e6a2984476e13acce22e64e4487636ef494369225da"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 3.0.6",
]

[[package]]
name = "zlib-rs"
version = "0.6.8"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b268e58e7c693d7c271f93ffc4ba3b380412554231c85bf61ca7af91042a4112"

[[package]]
name = "zmij"
version = "1.0.23"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "29666d0abbfad1e3dc4dcf6144730dd3a3ab225bbbdac83319345b1b44ccfc1b"

[[package]]
name = "zstd"
version = "0.14.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bf06bd8162af0734b344780deb55b42a2429ae430870d13fcc12f238e880fe6e"
dependencies = [
 "zstd-safe",
]

[[package]]
name = "zstd-safe"
version = "8.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ae42c0555055784c70058d19ba8e275528e8a99a706684868ace5da4e716a4ab"
dependencies = [
 "zstd-sys",
]

[[package]]
name = "zstd-sys"
version = "2.1.0+zstd.1.5.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0ef0a8027ec3ee71300ab3bcbcd0393f434aa72b91ca6d635a39941deae8eea0"
dependencies = [
 "cc",
 "pkg-config",
]

[[package]]
name = "zune-core"
version = "0.5.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d56377fd46368984a170bc5aac5567e52ca5da874caa60bea39fcbca78fb658b"

[[package]]
name = "zune-inflate"
version = "0.2.54"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "73ab332fe2f6680068f3582b16a24f90ad7096d5d39b974d1c0aff0125116f02"
dependencies = [
 "simd-adler32",
]

[[package]]
name = "zune-jpeg"
version = "0.5.15"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "27bc9d5b815bc103f142aa054f561d9187d191692ec7c2d1e2b4737f8dbd7296"
dependencies = [
 "zune-core",
]

[[package]]
name = "zvariant"
version = "5.15.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c1d34c27cc6cdd1f458427519dd6b8612f7b7e3f7b9a0b2355d041dda9869147"
dependencies = [
 "endi",
 "enumflags2",
 "serde",
 "winnow",
 "zcheapstr",
 "zvariant_derive",
 "zvariant_utils",
]

[[package]]
name = "zvariant_derive"
version = "5.15.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "864155e69b4352db0c7f374917bf45d1e0c8d17659c8b3dbf9795f3673f8c497"
dependencies = [
 "proc-macro-crate",
 "proc-macro2",
 "quote",
 "syn 3.0.6",
 "zvariant_utils",
]

[[package]]
name = "zvariant_utils"
version = "4.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bad0294361a320b694a328460dc73add56c306150f5cb6bfafc44446120008a3"
dependencies = [
 "proc-macro2",
 "quote",
 "serde",
 "syn 3.0.6",
 "winnow",
]
```


## 2026-11-01: E1c receiving native ordinary

```json
{
  "source_revision": "0f38cba4fbe9d5a8158170f18469743b8b7c4433",
  "utc": "2026-11-02T01:07:43.109387+00:00",
  "command": [
    "cargo",
    "test",
    "-p",
    "genet-webrtc",
    "--test",
    "browser",
    "--locked",
    "--offline",
    "-j",
    "1"
  ],
  "exit": 0,
  "seconds": 16.17,
  "lock_sha256": "e3efb15480e914bec5fb39bf2b3294aa43d1bee4f5ee40eef4f0b01ca71c9884",
  "stdout_tail": [
    "",
    "running 18 tests",
    "test boa::coturn_relay ... ignored, requires controlled coturn; MURM_RTC_TEST_SERVER numeric endpoint",
    "test vano::coturn_relay ... ignored, requires controlled coturn; MURM_RTC_TEST_SERVER numeric endpoint",
    "test vano::rollback ... ok",
    "test vano::gathering_errors ... ok",
    "test vano::policy_and_budgets ... ok",
    "test vano::strings_and_utf8_limits ... ok",
    "test vano::transfer ... ok",
    "test vano::document_teardown ... ok",
    "test vano::navigation_revokes_old_handles ... ok",
    "test vano::constructors ... ok",
    "test boa::rollback ... ok",
    "test boa::gathering_errors ... ok",
    "test boa::policy_and_budgets ... ok",
    "test boa::strings_and_utf8_limits ... ok",
    "test boa::transfer ... ok",
    "test boa::document_teardown ... ok",
    "test boa::navigation_revokes_old_handles ... ok",
    "test boa::constructors ... ok",
    "",
    "test result: ok. 16 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 13.55s",
    ""
  ],
  "stderr_tail": [
    "9 | use crate::OwnerResolvedCx as _;",
    "  |     ^^^^^^^^^^^^^^^^^^^^^^",
    "  |",
    "  = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default",
    "",
    "warning: unused import: `crate::OwnerResolvedCx`",
    "  --> components\\script-runtime-api\\dom\\shadow.rs:14:5",
    "   |",
    "14 | use crate::OwnerResolvedCx as _;",
    "   |     ^^^^^^^^^^^^^^^^^^^^^^",
    "",
    "warning: unused import: `crate::OwnerResolvedCx`",
    "  --> components\\script-runtime-api\\dom\\markup_insertion.rs:37:5",
    "   |",
    "37 | use crate::OwnerResolvedCx as _;",
    "   |     ^^^^^^^^^^^^^^^^^^^^^^",
    "",
    "warning: `genet-runtime` (lib) generated 4 warnings (run `cargo fix --lib -p genet-runtime` to apply 1 suggestion)",
    "    Finished `test` profile [unoptimized] target(s) in 2.44s",
    "     Running tests\\browser.rs (C:\\t\\cargo-targets\\genet\\build\\debug\\deps\\browser-44f1dcb4b50f684f.exe)"
  ]
}
```


## 2026-11-01: E1c receiving WPT build

```json
{
  "source_revision": "0f38cba4fbe9d5a8158170f18469743b8b7c4433",
  "utc": "2026-11-02T01:08:46.102642+00:00",
  "command": [
    "cargo",
    "build",
    "-p",
    "genet-wpt",
    "--no-default-features",
    "--features",
    "webrtc",
    "--locked",
    "--offline",
    "-j",
    "1"
  ],
  "exit": 0,
  "seconds": 62.99,
  "lock_sha256": "e3efb15480e914bec5fb39bf2b3294aa43d1bee4f5ee40eef4f0b01ca71c9884",
  "stdout_tail": [],
  "stderr_tail": [
    "    |",
    "162 | impl Harness {",
    "    | ------------ method in this implementation",
    "...",
    "178 |     pub fn assemble(&self, test_src: &str, meta: &Test262Meta, strict: bool) -> io::Result<String> {",
    "    |            ^^^^^^^^",
    "",
    "warning: variant `Unsupported` is never constructed",
    "   --> ports\\genet-wpt\\src\\test262.rs:270:5",
    "    |",
    "263 | pub enum Observation {",
    "    |          ----------- variant in this enum",
    "...",
    "270 |     Unsupported,",
    "    |     ^^^^^^^^^^^",
    "    |",
    "    = note: `Observation` has derived impls for the traits `Debug` and `Clone`, but these are intentionally ignored during dead code analysis",
    "",
    "warning: `genet-wpt` (bin \"genet-wpt\") generated 3 warnings",
    "    Finished `dev` profile [unoptimized] target(s) in 1m 02s"
  ]
}
```


## 2026-11-01: E1c matched selected WPT and ICE error constructor

```json
{
  "schema": 2,
  "phase": "after-udp-discovery",
  "source_revision": "0f38cba4fbe9d5a8158170f18469743b8b7c4433",
  "utc": "2026-11-02T01:11:31.004614+00:00",
  "binary_sha256": "fe0956ed14ac1aaae8496450da87bcc647e68da2c3f294733b73ae554ad15f05",
  "baseline_source": "835dca0f37cf7efbd6d791f94752a2e40733c2fa",
  "outcomes": [
    {
      "detail": null,
      "failures": [],
      "reason": null,
      "status": "pass",
      "subtests": [
        {
          "name": "RTCPeerConnection.length",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection()",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection(null)",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection(undefined)",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({})",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: null })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: undefined })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: [] })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: [null] })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: [undefined] })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ iceCandidatePoolSize: toNumberThrows })",
          "status": "pass"
        },
        {
          "name": "localDescription initial value",
          "status": "pass"
        },
        {
          "name": "currentLocalDescription initial value",
          "status": "pass"
        },
        {
          "name": "pendingLocalDescription initial value",
          "status": "pass"
        },
        {
          "name": "remoteDescription initial value",
          "status": "pass"
        },
        {
          "name": "currentRemoteDescription initial value",
          "status": "pass"
        },
        {
          "name": "pendingRemoteDescription initial value",
          "status": "pass"
        },
        {
          "name": "signalingState initial value",
          "status": "pass"
        },
        {
          "name": "iceGatheringState initial value",
          "status": "pass"
        },
        {
          "name": "iceConnectionState initial value",
          "status": "pass"
        },
        {
          "name": "connectionState initial value",
          "status": "pass"
        },
        {
          "name": "canTrickleIceCandidates initial value",
          "status": "pass"
        }
      ],
      "subtests_passed": 22,
      "subtests_total": 22,
      "engine": "boa",
      "test": "webrtc/RTCPeerConnection-constructor.html",
      "exit_code": 0,
      "command": [
        "C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe",
        "testharness-one",
        "webrtc/RTCPeerConnection-constructor.html",
        "--test-path",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCPeerConnection-constructor.html",
        "--tests-root",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests",
        "--engine",
        "boa",
        "--drive-deadline",
        "5"
      ]
    },
    {
      "detail": null,
      "failures": [],
      "reason": null,
      "status": "pass",
      "subtests": [
        {
          "name": "new RTCIceCandidate()",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({})",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with manually filled default values",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ sdpMid: null, sdpMLineIndex: null })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ candidate: '' })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ candidate: null })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with valid candidate string only",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ sdpMid: 'audio' })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ sdpMLineIndex: 0 })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ sdpMid: 'audio', sdpMLineIndex: 0 })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ candidate: '', sdpMid: 'audio' }",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ candidate: null, sdpMLineIndex: 0 }",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with valid candidate string and sdpMid",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with invalid candidate string and sdpMid",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for most fields",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for most fields, tcp candidate",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for all fields",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for all fields, tcp candidate",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ relayProtocol, url }) cloned vs signaled",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with invalid sdpMid",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with invalid sdpMLineIndex",
          "status": "pass"
        }
      ],
      "subtests_passed": 21,
      "subtests_total": 21,
      "engine": "boa",
      "test": "webrtc/RTCIceCandidate-constructor.html",
      "exit_code": 0,
      "command": [
        "C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe",
        "testharness-one",
        "webrtc/RTCIceCandidate-constructor.html",
        "--test-path",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCIceCandidate-constructor.html",
        "--tests-root",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests",
        "--engine",
        "boa",
        "--drive-deadline",
        "5"
      ]
    },
    {
      "detail": null,
      "failures": [
        "        [1] addTrack, then creating datachannel, should negotiate properly promise_test: Unhandled rejection with value: object \"TypeError: cannot convert 'null' or 'undefined' to object\"",
        "        [1] addTrack, then creating datachannel, should negotiate properly when max-bundle is used promise_test: Unhandled rejection with value: object \"TypeError: cannot convert 'null' or 'undefined' to object\"",
        "        [1] Renegotiation with audio/video and datachannel should work properly promise_test: Unhandled rejection with value: object \"TypeError: cannot convert 'null' or 'undefined' to object\"",
        "        [1] addTrack, then creating negotiated datachannel, should negotiate properly promise_test: Unhandled rejection with value: object \"TypeError: cannot convert 'null' or 'undefined' to object\"",
        "        [1] addTrack, then creating negotiated datachannel, should negotiate properly when max-bundle is used promise_test: Unhandled rejection with value: object \"TypeError: cannot convert 'null' or 'undefined' to object\"",
        "        [1] Renegotiation with audio/video and negotiated datachannel should work properly promise_test: Unhandled rejection with value: object \"TypeError: cannot convert 'null' or 'undefined' to object\""
      ],
      "reason": null,
      "status": "fail",
      "subtests": [
        {
          "name": "createDataChannel with no argument should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with closed connection should throw InvalidStateError",
          "status": "pass"
        },
        {
          "name": "createDataChannel attribute default values",
          "status": "pass"
        },
        {
          "name": "createDataChannel with provided parameters should initialize attributes to provided values",
          "status": "pass"
        },
        {
          "name": "createDataChannel with label \"foo\" should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with label null should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with label undefined should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with label lone surrogate should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with ordered false should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with ordered null/undefined should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with maxPacketLifeTime 0 should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with maxRetransmits 0 should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with both maxPacketLifeTime and maxRetransmits undefined should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with both maxPacketLifeTime and maxRetransmits should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with protocol \"foo\" should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with protocol null should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with protocol undefined should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with protocol lone surrogate should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 0 and negotiated not set should succeed, but not set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 1 and negotiated not set should succeed, but not set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65534 and negotiated not set should succeed, but not set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65535 and negotiated not set should succeed, but not set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 0 and negotiated true should succeed, and set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 1 and negotiated true should succeed, and set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65534 and negotiated true should succeed, and set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id -1 and negotiated not set should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65536 and negotiated not set should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id -1 should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65535 should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65536 should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with too long label should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with too long label (2 byte unicode) should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with same label used twice should not throw",
          "status": "pass"
        },
        {
          "name": "createDataChannel with negotiated true and id should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with too long protocol should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with too long protocol (2 byte unicode) should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with maximum length label and protocol should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with negotiated false should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with negotiated false and id 42 should ignore the id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with negotiated true and id not defined should throw TypeError",
          "status": "pass"
        },
        {
          "name": "Channels created (after SCTP connected) should have id assigned",
          "status": "pass"
        },
        {
          "name": "Reusing a data channel id that is in use should throw OperationError",
          "status": "pass"
        },
        {
          "name": "Reusing a data channel id that is in use (after setRemoteDescription) should throw OperationError",
          "status": "pass"
        },
        {
          "name": "Reusing a data channel id that is in use (after setRemoteDescription, negotiated via DCEP) should throw OperationError",
          "status": "pass"
        },
        {
          "name": "New datachannel should be in the connecting state after creation (after connection establishment)",
          "status": "pass"
        },
        {
          "name": "addTrack, then creating datachannel, should negotiate properly",
          "status": "fail"
        },
        {
          "name": "addTrack, then creating datachannel, should negotiate properly when max-bundle is used",
          "status": "fail"
        },
        {
          "name": "Renegotiation with audio/video and datachannel should work properly",
          "status": "fail"
        },
        {
          "name": "New negotiated datachannel should be in the connecting state after creation (after connection establishment)",
          "status": "pass"
        },
        {
          "name": "addTrack, then creating negotiated datachannel, should negotiate properly",
          "status": "fail"
        },
        {
          "name": "addTrack, then creating negotiated datachannel, should negotiate properly when max-bundle is used",
          "status": "fail"
        },
        {
          "name": "Renegotiation with audio/video and negotiated datachannel should work properly",
          "status": "fail"
        }
      ],
      "subtests_passed": 46,
      "subtests_total": 52,
      "engine": "boa",
      "test": "webrtc/RTCPeerConnection-createDataChannel.html",
      "exit_code": 0,
      "command": [
        "C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe",
        "testharness-one",
        "webrtc/RTCPeerConnection-createDataChannel.html",
        "--test-path",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCPeerConnection-createDataChannel.html",
        "--tests-root",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests",
        "--engine",
        "boa",
        "--drive-deadline",
        "5"
      ]
    },
    {
      "detail": null,
      "failures": [],
      "reason": null,
      "status": "pass",
      "subtests": [
        {
          "name": "RTCPeerConnection.length",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection()",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection(null)",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection(undefined)",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({})",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: null })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: undefined })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: [] })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: [null] })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ certificates: [undefined] })",
          "status": "pass"
        },
        {
          "name": "new RTCPeerConnection({ iceCandidatePoolSize: toNumberThrows })",
          "status": "pass"
        },
        {
          "name": "localDescription initial value",
          "status": "pass"
        },
        {
          "name": "currentLocalDescription initial value",
          "status": "pass"
        },
        {
          "name": "pendingLocalDescription initial value",
          "status": "pass"
        },
        {
          "name": "remoteDescription initial value",
          "status": "pass"
        },
        {
          "name": "currentRemoteDescription initial value",
          "status": "pass"
        },
        {
          "name": "pendingRemoteDescription initial value",
          "status": "pass"
        },
        {
          "name": "signalingState initial value",
          "status": "pass"
        },
        {
          "name": "iceGatheringState initial value",
          "status": "pass"
        },
        {
          "name": "iceConnectionState initial value",
          "status": "pass"
        },
        {
          "name": "connectionState initial value",
          "status": "pass"
        },
        {
          "name": "canTrickleIceCandidates initial value",
          "status": "pass"
        }
      ],
      "subtests_passed": 22,
      "subtests_total": 22,
      "engine": "nova",
      "test": "webrtc/RTCPeerConnection-constructor.html",
      "exit_code": 0,
      "command": [
        "C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe",
        "testharness-one",
        "webrtc/RTCPeerConnection-constructor.html",
        "--test-path",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCPeerConnection-constructor.html",
        "--tests-root",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests",
        "--engine",
        "nova",
        "--drive-deadline",
        "5"
      ]
    },
    {
      "detail": null,
      "failures": [],
      "reason": null,
      "status": "pass",
      "subtests": [
        {
          "name": "new RTCIceCandidate()",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({})",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with manually filled default values",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ sdpMid: null, sdpMLineIndex: null })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ candidate: '' })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ candidate: null })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with valid candidate string only",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ sdpMid: 'audio' })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ sdpMLineIndex: 0 })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ sdpMid: 'audio', sdpMLineIndex: 0 })",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ candidate: '', sdpMid: 'audio' }",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ candidate: null, sdpMLineIndex: 0 }",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with valid candidate string and sdpMid",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with invalid candidate string and sdpMid",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for most fields",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for most fields, tcp candidate",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for all fields",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with nondefault values for all fields, tcp candidate",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ relayProtocol, url }) cloned vs signaled",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with invalid sdpMid",
          "status": "pass"
        },
        {
          "name": "new RTCIceCandidate({ ... }) with invalid sdpMLineIndex",
          "status": "pass"
        }
      ],
      "subtests_passed": 21,
      "subtests_total": 21,
      "engine": "nova",
      "test": "webrtc/RTCIceCandidate-constructor.html",
      "exit_code": 0,
      "command": [
        "C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe",
        "testharness-one",
        "webrtc/RTCIceCandidate-constructor.html",
        "--test-path",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCIceCandidate-constructor.html",
        "--tests-root",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests",
        "--engine",
        "nova",
        "--drive-deadline",
        "5"
      ]
    },
    {
      "detail": null,
      "failures": [
        "        [1] addTrack, then creating datachannel, should negotiate properly promise_test: Unhandled rejection with value: object \"TypeError: Cannot read property 'getUserMedia' of undefined.\"",
        "        [1] addTrack, then creating datachannel, should negotiate properly when max-bundle is used promise_test: Unhandled rejection with value: object \"TypeError: Cannot read property 'getUserMedia' of undefined.\"",
        "        [1] Renegotiation with audio/video and datachannel should work properly promise_test: Unhandled rejection with value: object \"TypeError: Cannot read property 'getUserMedia' of undefined.\"",
        "        [1] addTrack, then creating negotiated datachannel, should negotiate properly promise_test: Unhandled rejection with value: object \"TypeError: Cannot read property 'getUserMedia' of undefined.\"",
        "        [1] addTrack, then creating negotiated datachannel, should negotiate properly when max-bundle is used promise_test: Unhandled rejection with value: object \"TypeError: Cannot read property 'getUserMedia' of undefined.\"",
        "        [1] Renegotiation with audio/video and negotiated datachannel should work properly promise_test: Unhandled rejection with value: object \"TypeError: Cannot read property 'getUserMedia' of undefined.\""
      ],
      "reason": null,
      "status": "fail",
      "subtests": [
        {
          "name": "createDataChannel with no argument should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with closed connection should throw InvalidStateError",
          "status": "pass"
        },
        {
          "name": "createDataChannel attribute default values",
          "status": "pass"
        },
        {
          "name": "createDataChannel with provided parameters should initialize attributes to provided values",
          "status": "pass"
        },
        {
          "name": "createDataChannel with label \"foo\" should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with label null should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with label undefined should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with label lone surrogate should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with ordered false should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with ordered null/undefined should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with maxPacketLifeTime 0 should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with maxRetransmits 0 should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with both maxPacketLifeTime and maxRetransmits undefined should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with both maxPacketLifeTime and maxRetransmits should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with protocol \"foo\" should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with protocol null should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with protocol undefined should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with protocol lone surrogate should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 0 and negotiated not set should succeed, but not set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 1 and negotiated not set should succeed, but not set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65534 and negotiated not set should succeed, but not set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65535 and negotiated not set should succeed, but not set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 0 and negotiated true should succeed, and set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 1 and negotiated true should succeed, and set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65534 and negotiated true should succeed, and set the channel's id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id -1 and negotiated not set should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65536 and negotiated not set should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id -1 should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65535 should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with id 65536 should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with too long label should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with too long label (2 byte unicode) should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with same label used twice should not throw",
          "status": "pass"
        },
        {
          "name": "createDataChannel with negotiated true and id should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with too long protocol should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with too long protocol (2 byte unicode) should throw TypeError",
          "status": "pass"
        },
        {
          "name": "createDataChannel with maximum length label and protocol should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with negotiated false should succeed",
          "status": "pass"
        },
        {
          "name": "createDataChannel with negotiated false and id 42 should ignore the id",
          "status": "pass"
        },
        {
          "name": "createDataChannel with negotiated true and id not defined should throw TypeError",
          "status": "pass"
        },
        {
          "name": "Channels created (after SCTP connected) should have id assigned",
          "status": "pass"
        },
        {
          "name": "Reusing a data channel id that is in use should throw OperationError",
          "status": "pass"
        },
        {
          "name": "Reusing a data channel id that is in use (after setRemoteDescription) should throw OperationError",
          "status": "pass"
        },
        {
          "name": "Reusing a data channel id that is in use (after setRemoteDescription, negotiated via DCEP) should throw OperationError",
          "status": "pass"
        },
        {
          "name": "New datachannel should be in the connecting state after creation (after connection establishment)",
          "status": "pass"
        },
        {
          "name": "addTrack, then creating datachannel, should negotiate properly",
          "status": "fail"
        },
        {
          "name": "addTrack, then creating datachannel, should negotiate properly when max-bundle is used",
          "status": "fail"
        },
        {
          "name": "Renegotiation with audio/video and datachannel should work properly",
          "status": "fail"
        },
        {
          "name": "New negotiated datachannel should be in the connecting state after creation (after connection establishment)",
          "status": "pass"
        },
        {
          "name": "addTrack, then creating negotiated datachannel, should negotiate properly",
          "status": "fail"
        },
        {
          "name": "addTrack, then creating negotiated datachannel, should negotiate properly when max-bundle is used",
          "status": "fail"
        },
        {
          "name": "Renegotiation with audio/video and negotiated datachannel should work properly",
          "status": "fail"
        }
      ],
      "subtests_passed": 46,
      "subtests_total": 52,
      "engine": "nova",
      "test": "webrtc/RTCPeerConnection-createDataChannel.html",
      "exit_code": 0,
      "command": [
        "C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe",
        "testharness-one",
        "webrtc/RTCPeerConnection-createDataChannel.html",
        "--test-path",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCPeerConnection-createDataChannel.html",
        "--tests-root",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests",
        "--engine",
        "nova",
        "--drive-deadline",
        "5"
      ]
    },
    {
      "detail": null,
      "failures": [],
      "reason": null,
      "status": "pass",
      "subtests": [
        {
          "name": "RTCPeerConnectionIceErrorEvent constructed from init parameters",
          "status": "pass"
        }
      ],
      "subtests_passed": 1,
      "subtests_total": 1,
      "engine": "boa",
      "test": "webrtc/RTCPeerConnectionIceErrorEvent.html",
      "exit_code": 0,
      "command": [
        "C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe",
        "testharness-one",
        "webrtc/RTCPeerConnectionIceErrorEvent.html",
        "--test-path",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCPeerConnectionIceErrorEvent.html",
        "--tests-root",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests",
        "--engine",
        "boa",
        "--drive-deadline",
        "5"
      ]
    },
    {
      "detail": null,
      "failures": [],
      "reason": null,
      "status": "pass",
      "subtests": [
        {
          "name": "RTCPeerConnectionIceErrorEvent constructed from init parameters",
          "status": "pass"
        }
      ],
      "subtests_passed": 1,
      "subtests_total": 1,
      "engine": "nova",
      "test": "webrtc/RTCPeerConnectionIceErrorEvent.html",
      "exit_code": 0,
      "command": [
        "C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe",
        "testharness-one",
        "webrtc/RTCPeerConnectionIceErrorEvent.html",
        "--test-path",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCPeerConnectionIceErrorEvent.html",
        "--tests-root",
        "C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests",
        "--engine",
        "nova",
        "--drive-deadline",
        "5"
      ]
    }
  ],
  "frozen_case_comparison": [
    {
      "engine": "boa",
      "test": "webrtc/RTCPeerConnection-constructor.html",
      "missing": [],
      "added": [],
      "regressions": []
    },
    {
      "engine": "boa",
      "test": "webrtc/RTCIceCandidate-constructor.html",
      "missing": [],
      "added": [],
      "regressions": []
    },
    {
      "engine": "boa",
      "test": "webrtc/RTCPeerConnection-createDataChannel.html",
      "missing": [],
      "added": [],
      "regressions": []
    },
    {
      "engine": "nova",
      "test": "webrtc/RTCPeerConnection-constructor.html",
      "missing": [],
      "added": [],
      "regressions": []
    },
    {
      "engine": "nova",
      "test": "webrtc/RTCIceCandidate-constructor.html",
      "missing": [],
      "added": [],
      "regressions": []
    },
    {
      "engine": "nova",
      "test": "webrtc/RTCPeerConnection-createDataChannel.html",
      "missing": [],
      "added": [],
      "regressions": []
    }
  ]
}
```


## 2026-11-01: E1c receiving controlled coturn

```json
{
  "source_revision": "0f38cba4fbe9d5a8158170f18469743b8b7c4433",
  "utc": "2026-11-02T01:12:33.047545+00:00",
  "command": [
    "cargo",
    "test",
    "-p",
    "genet-webrtc",
    "--test",
    "browser",
    "--locked",
    "--offline",
    "-j",
    "1",
    "--",
    "--ignored",
    "--test-threads=1"
  ],
  "exit": 0,
  "seconds": 31.05,
  "stdout": [
    "",
    "running 2 tests",
    "test boa::coturn_relay ... ok",
    "test vano::coturn_relay ... ok",
    "",
    "test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 16 filtered out; finished in 20.91s",
    ""
  ],
  "stderr_tail": [
    "",
    "warning: `genet-runtime` (lib) generated 4 warnings (run `cargo fix --lib -p genet-runtime` to apply 1 suggestion)",
    "    Finished `test` profile [unoptimized] target(s) in 9.93s",
    "     Running tests\\browser.rs (C:\\t\\cargo-targets\\genet\\build\\debug\\deps\\browser-44f1dcb4b50f684f.exe)"
  ],
  "binary_sha256": "4253487a709d23de1d53b3c51da76d564d946ce7f2218164dd8bd22ca8fa0026"
}
```


## 2026-11-01: E1c Ortet receiving check

```json
{
  "source_revision": "0f38cba4fbe9d5a8158170f18469743b8b7c4433",
  "utc": "2026-11-02T01:13:05.550381+00:00",
  "command": [
    "cargo",
    "check",
    "-p",
    "ortet",
    "--features",
    "scripted-nova",
    "--all-targets",
    "--locked",
    "--offline",
    "-j",
    "1"
  ],
  "exit": 0,
  "seconds": 20.83,
  "stderr_tail": [
    "warning: `genet-documents` (lib) generated 3 warnings (run `cargo fix --lib -p genet-documents` to apply 2 suggestions)",
    "    Checking genet-webrtc v0.2.0 (C:\\Users\\mark_\\Code\\repos\\genet\\components\\genet-webrtc)",
    "    Checking ortet v0.2.0 (C:\\Users\\mark_\\Code\\repos\\genet\\ports\\ortet)",
    "warning: unused import: `SessionClick`",
    "  --> ports\\ortet\\src\\shell.rs:19:42",
    "   |",
    "19 |     DocumentSession, SessionButtonState, SessionClick, SessionCursor, SessionEffect, SessionEngine,",
    "   |                                          ^^^^^^^^^^^^",
    "   |",
    "   = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default",
    "",
    "warning: method `navigate` is never used",
    "   --> ports\\ortet\\src\\shell.rs:754:8",
    "    |",
    "462 | impl Ortet {",
    "    | ---------- method in this implementation",
    "...",
    "754 |     fn navigate(&mut self, target: &str) {",
    "    |        ^^^^^^^^",
    "    |",
    "    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default",
    "",
    "warning: `ortet` (lib) generated 2 warnings (run `cargo fix --lib -p ortet` to apply 1 suggestion)",
    "warning: `ortet` (lib test) generated 1 warning (1 duplicate)",
    "    Finished `dev` profile [unoptimized] target(s) in 20.58s"
  ]
}
```


## 2026-11-01: coturn confirms allocation refresh and release

{
  "source_revision": "0f38cba4fbe9d5a8158170f18469743b8b7c4433",
  "fixture": "murm/wire/crates/rtc/tests/coturn.sh (verbose)",
  "server": "coturn 4.6.1, WSL local virtual network",
  "allocations": 4,
  "renewals": 22,
  "releases": 4,
  "observed_lines": [
    "96: : session 000000000000000001: realm <murm-rtc.test> user <rtc-test>: incoming packet ALLOCATE processed, success",
    "96: : session 000000000000000002: realm <murm-rtc.test> user <rtc-test>: incoming packet ALLOCATE processed, success",
    "96: : session 000000000000000001: realm <murm-rtc.test> user <rtc-test>: incoming packet CREATE_PERMISSION processed, success",
    "96: : session 000000000000000002: realm <murm-rtc.test> user <rtc-test>: incoming packet CREATE_PERMISSION processed, success",
    "96: : session 000000000000000001: realm <murm-rtc.test> user <rtc-test>: incoming packet CREATE_PERMISSION processed, success",
    "96: : session 000000000000000002: realm <murm-rtc.test> user <rtc-test>: incoming packet CREATE_PERMISSION processed, success",
    "96: : session 000000000000000001: realm <murm-rtc.test> user <rtc-test>: incoming packet CREATE_PERMISSION processed, success",
    "96: : session 000000000000000002: realm <murm-rtc.test> user <rtc-test>: incoming packet CREATE_PERMISSION processed, success",
    "96: : session 000000000000000001: realm <murm-rtc.test> user <rtc-test>: incoming packet CREATE_PERMISSION processed, success",
    "97: : session 000000000000000001: refreshed, realm=<murm-rtc.test>, username=<rtc-test>, lifetime=600",
    "97: : session 000000000000000001: realm <murm-rtc.test> user <rtc-test>: incoming packet REFRESH processed, success",
    "97: : session 000000000000000002: refreshed, realm=<murm-rtc.test>, username=<rtc-test>, lifetime=600",
    "97: : session 000000000000000002: realm <murm-rtc.test> user <rtc-test>: incoming packet REFRESH processed, success",
    "98: : session 000000000000000002: refreshed, realm=<murm-rtc.test>, username=<rtc-test>, lifetime=600",
    "98: : session 000000000000000002: realm <murm-rtc.test> user <rtc-test>: incoming packet REFRESH processed, success",
    "98: : session 000000000000000001: refreshed, realm=<murm-rtc.test>, username=<rtc-test>, lifetime=600",
    "98: : session 000000000000000001: realm <murm-rtc.test> user <rtc-test>: incoming packet REFRESH processed, success",
    "99: : session 000000000000000002: refreshed, realm=<murm-rtc.test>, username=<rtc-test>, lifetime=600",
    "99: : session 000000000000000002: realm <murm-rtc.test> user <rtc-test>: incoming packet REFRESH processed, success",
    "99: : session 000000000000000001: refreshed, realm=<murm-rtc.test>, username=<rtc-test>, lifetime=600",
    "99: : session 000000000000000001: realm <murm-rtc.test> user <rtc-test>: incoming packet REFRESH processed, success",
    "100: : session 000000000000000001: refreshed, realm=<murm-rtc.test>, username=<rtc-test>, lifetime=600",
    "100: : session 000000000000000001: realm <murm-rtc.test> user <rtc-test>: incoming packet REFRESH processed, success",
    "100: : session 000000000000000002: refreshed, realm=<murm-rtc.test>, username=<rtc-test>, lifetime=600",
    "100: : session 000000000000000002: realm <murm-rtc.test> user <rtc-test>: incoming packet REFRESH processed, success",
    "101: : session 000000000000000001: refreshed, realm=<murm-rtc.test>, username=<rtc-test>, lifetime=600",
    "101: : session 000000000000000001: realm <murm-rtc.test> user <rtc-test>: incoming packet REFRESH processed, success",
    "101: : session 000000000000000002: refreshed, realm=<murm-rtc.test>, username=<rtc-test>, lifetime=600",
    "101: : session 000000000000000002: realm <murm-rtc.test> user <rtc-test>: incoming packet REFRESH processed, success",
    "102: : session 000000000000000001: refreshed, realm=<murm-rtc.test>, username=<rtc-test>, lifetime=600",
    "102: : session 000000000000000001: realm <murm-rtc.test> user <rtc-test>: incoming packet REFRESH processed, success",
    "102: : session 000000000000000002: refreshed, realm=<murm-rtc.test>, username=<rtc-test>, lifetime=600",
    "102: : session 000000000000000002: realm <murm-rtc.test> user <rtc-test>: incoming packet REFRESH processed, success",
    "102: : session 000000000000000001: refreshed, realm=<murm-rtc.test>, username=<rtc-test>, lifetime=0",
    "102: : session 000000000000000001: realm <murm-rtc.test> user <rtc-test>: incoming packet REFRESH processed, success",
    "102: : session 000000000000000002: refreshed, realm=<murm-rtc.test>, username=<rtc-test>, lifetime=0",
    "102: : session 000000000000000002: realm <murm-rtc.test> user <rtc-test>: incoming packet REFRESH processed, success",
    "104: : session 000000000000000003: realm <murm-rtc.test> user <rtc-test>: incoming packet ALLOCATE processed, success",
    "104: : session 000000000000000004: realm <murm-rtc.test> user <rtc-test>: incoming packet ALLOCATE processed, success",
    "104: : session 000000000000000003: realm <murm-rtc.test> user <rtc-test>: incoming packet CREATE_PERMISSION processed, success",
    "104: : session 000000000000000004: realm <murm-rtc.test> user <rtc-test>: incoming packet CREATE_PERMISSION processed, success",
    "104: : session 000000000000000003: realm <murm-rtc.test> user <rtc-test>: incoming packet CREATE_PERMISSION processed, success",
    "104: : session 000000000000000004: realm <murm-rtc.test> user <rtc-test>: incoming packet CREATE_PERMISSION processed, success",
    "104: : session 000000000000000003: realm <murm-rtc.test> user <rtc-test>: incoming packet CREATE_PERMISSION processed, success",
    "104: : session 000000000000000004: realm <murm-rtc.test> user <rtc-test>: incoming packet CREATE_PERMISSION processed, success",
    "105: : session 000000000000000003: refreshed, realm=<murm-rtc.test>, username=<rtc-test>, lifetime=600",
    "105: : session 000000000000000003: realm <murm-rtc.test> user <rtc-test>: incoming packet REFRESH processed, success",
    "105: : session 000000000000000004: refreshed, realm=<murm-rtc.test>, username=<rtc-test>, lifetime=600",
    "105: : session 000000000000000004: realm <murm-rtc.test> user <rtc-test>: incoming packet REFRESH processed, success",
    "106: : session 000000000000000003: refreshed, realm=<murm-rtc.test>, username=<rtc-test>, lifetime=600",
    "106: : session 000000000000000003: realm <murm-rtc.test> user <rtc-test>: incoming packet REFRESH processed, success",
    "106: : session 000000000000000004: refreshed, realm=<murm-rtc.test>, username=<rtc-test>, lifetime=600",
    "106: : session 000000000000000004: realm <murm-rtc.test> user <rtc-test>: incoming packet REFRESH processed, success",
    "107: : session 000000000000000003: refreshed, realm=<murm-rtc.test>, username=<rtc-test>, lifetime=600",
    "107: : session 000000000000000003: realm <murm-rtc.test> user <rtc-test>: incoming packet REFRESH processed, success",
    "107: : session 000000000000000004: refreshed, realm=<murm-rtc.test>, username=<rtc-test>, lifetime=600",
    "107: : session 000000000000000004: realm <murm-rtc.test> user <rtc-test>: incoming packet REFRESH processed, success",
    "108: : session 000000000000000003: refreshed, realm=<murm-rtc.test>, username=<rtc-test>, lifetime=600",
    "108: : session 000000000000000003: realm <murm-rtc.test> user <rtc-test>: incoming packet REFRESH processed, success",
    "108: : session 000000000000000004: refreshed, realm=<murm-rtc.test>, username=<rtc-test>, lifetime=600",
    "108: : session 000000000000000004: realm <murm-rtc.test> user <rtc-test>: incoming packet REFRESH processed, success",
    "110: : session 000000000000000003: refreshed, realm=<murm-rtc.test>, username=<rtc-test>, lifetime=600",
    "110: : session 000000000000000003: realm <murm-rtc.test> user <rtc-test>: incoming packet REFRESH processed, success",
    "110: : session 000000000000000004: refreshed, realm=<murm-rtc.test>, username=<rtc-test>, lifetime=600",
    "110: : session 000000000000000004: realm <murm-rtc.test> user <rtc-test>: incoming packet REFRESH processed, success",
    "111: : session 000000000000000003: refreshed, realm=<murm-rtc.test>, username=<rtc-test>, lifetime=0",
    "111: : session 000000000000000003: realm <murm-rtc.test> user <rtc-test>: incoming packet REFRESH processed, success",
    "111: : session 000000000000000004: refreshed, realm=<murm-rtc.test>, username=<rtc-test>, lifetime=0",
    "111: : session 000000000000000004: realm <murm-rtc.test> user <rtc-test>: incoming packet REFRESH processed, success"
  ]
}


## 2026-11-01: E1c final source and receiving audit

Source committed before the final ordinary/WPT/controlled/Ortet gates. The
strict native library Clippy gate (`cargo clippy -p genet-webrtc --lib --no-deps
--locked --offline -j 1 -- -D warnings`) passed on these source bytes before the
source commit; inherited runtime/Ortet warnings remain outside this slice.
Final native ordinary gate: 16 passed; the two explicitly selected coturn cases
also passed, for 18 native receiving checks across Boa/Vano. Coturn confirms
four allocations, 22 renewals and four lifetime-zero releases in the separate
server observations above. Original selected WPT remains 89/95 per engine,
with zero missing, added or regressed identities. The separate unmodified
ICE-error constructor WPT passes 1/1 per engine. Six media assertions remain
failed; the selection is not a whole manifest denominator.

The first native relay attempts caught missing local server URLs. The supplier
now registers that metadata explicitly, and Genet retains it from the core's
separate event field for candidate events and transport getters while omitting
it from JS signaling JSON. Earlier supplier/test conversion, completion and
close-flush corrections are recorded in Murm's receipt. One exploratory WPT
capture parser expected bare JSON; the final gate parses the actual __thresult
marker and preserves all original case rows.

The root lock is ignored by repository policy. Its only change from the full
prior receiving lock is the exact Murm source revision below. No new package
versions were selected. Reused C:\t\cargo-targets\genet and its build directory,
debug=0, jobs=1. No isolated target, Cargo home or worktree was created. The
foreground coturn fixture has been stopped; its installed system service is
inactive and disabled. No fixture process is retained.

Qualified scope: native Windows/WSL virtual network, IP-literal UDP STUN/TURN,
caller-driven I/O, typed gathering events and the selected tests. DNS, TCP/TLS,
mDNS, restart/reconfiguration, allocation/permission expiry and recovery,
induced loss/saturation, public deployment, media, headed interoperability,
browser guest and application-carrier migration remain open.

```json
{
  "source_revision": "0f38cba4fbe9d5a8158170f18469743b8b7c4433",
  "supplier_revision": "ebafed19bf68edcfa0c276f0693a2bf45c858174",
  "toolchain": "rustc 1.99.1 (58cf8f9c3 2026-10-13)",
  "lock_sha256": "e3efb15480e914bec5fb39bf2b3294aa43d1bee4f5ee40eef4f0b01ca71c9884",
  "source_blobs": {
    "components/genet-webrtc/Cargo.toml": "82f9fdb1f3266a722da3433bd1ce28472bb2add4",
    "components/genet-webrtc/src/lib.rs": "c7092bde7f5b66126927f14b54506672878828a2",
    "components/genet-webrtc/tests/browser.rs": "1c417f60778e793061e1f6c62013251224bb53f1",
    "components/script-runtime-api/webrtc/bootstrap.js": "cfe4526a1736651c8c3357eef3725de3fa74974c"
  }
}
```

```diff
--- prior-receiving-lock

+++ E1c-receiving-lock

@@ -4567,7 +4567,7 @@

 [[package]]
 name = "murm-rtc"
 version = "0.1.0"
-source = "git+https://github.com/merely-made/murm.git?rev=c58b70421d31ed9f2288dc44152dcb6d719552a9#c58b70421d31ed9f2288dc44152dcb6d719552a9"
+source = "git+https://github.com/merely-made/murm.git?rev=ebafed19bf68edcfa0c276f0693a2bf45c858174#ebafed19bf68edcfa0c276f0693a2bf45c858174"
 dependencies = [
  "bytes",
  "rtc",
```


## E2h automated browser interoperability - 2026-11-02T18:53:41.393Z

```json
{
  "schema": 1,
  "gate": "E2h",
  "utc": "2026-11-02T18:53:41.393Z",
  "source": "42f9bc7df50e38669479fd37adc450e526959383",
  "node": "v24.11.0",
  "nativeBinarySHA256": "6c759ef1525ff2301c313e683fbe394b6eb956e86c50b0631b78968e749856a2",
  "sourceSHA256": {
    "components/genet-webrtc/examples/interop_host.rs": "f2f765ce031dced2da615daca5b68c1d525544fc880c2d99c75c99591dd1a375",
    "support/ci/webrtc_interop.mjs": "f76e0c4368c96c5e07ace5c8b62bce799dc9c4dcd2502e12f0792b2607ef2471",
    "support/ci/webrtc_interop_peer.js": "94df238585b1f45d27419157edd569e6d023ba6daa75913f364c07d20d6b1813",
    "components/genet-webrtc/Cargo.toml": "5884b70af4d2794b2ff85bbfddd2523632b8a8b4ba66c093247eb98aa1882ec2",
    "components/genet-webrtc/src/lib.rs": "753e27024d2048812924c56158fa274af793aa9b17c6dbd4d43df4dceddb3a05",
    "components/script-runtime-api/webrtc/bootstrap.js": "b16a9c2cc771628ab6800492789278dec6fccc524c8308e2d03272f3b77c1bdd"
  },
  "versions": [
    {
      "browser": "chrome",
      "version": "Chrome/156.0.8078.74",
      "headed": true,
      "visibleWindows": 1,
      "webrtcPreferenceOverrides": false
    }
  ],
  "cases": [
    {
      "browser": "chrome",
      "engine": "boa",
      "route": "direct",
      "offerer": "native",
      "cleanup": "close",
      "status": "fail",
      "phase": "peer startup",
      "error": "native fixture exited 3221225725"
    }
  ],
  "failure": {
    "phase": "peer startup",
    "case": {
      "browser": "chrome",
      "engine": "boa",
      "route": "direct",
      "offerer": "native",
      "cleanup": "close"
    },
    "error": "native fixture exited 3221225725"
  },
  "profilesRemoved": true,
  "scope": "native Windows to installed headed browsers, controlled WSL UDP server; no DNS/TCP/TLS/restart/media/carrier acceptance"
}
```


## E2h automated browser interoperability - 2026-11-02T18:54:52.040Z

```json
{
  "schema": 1,
  "gate": "E2h",
  "utc": "2026-11-02T18:54:52.040Z",
  "source": "42f9bc7df50e38669479fd37adc450e526959383",
  "node": "v24.11.0",
  "nativeBinarySHA256": "fdf5b358be3c0ddbd39d8d7822c6945ae806e5c7f9dd4c27d75b9e87ed96af3a",
  "sourceSHA256": {
    "components/genet-webrtc/examples/interop_host.rs": "ef63ed3a12dff896808373c2e41b0c37ef3540a4f199582c9163191f62f850d6",
    "support/ci/webrtc_interop.mjs": "f1346001c555dfdd4c81f57423d8b0d342f6715f35b888cb3ed54170ab571570",
    "support/ci/webrtc_interop_peer.js": "94df238585b1f45d27419157edd569e6d023ba6daa75913f364c07d20d6b1813",
    "components/genet-webrtc/Cargo.toml": "5884b70af4d2794b2ff85bbfddd2523632b8a8b4ba66c093247eb98aa1882ec2",
    "components/genet-webrtc/src/lib.rs": "753e27024d2048812924c56158fa274af793aa9b17c6dbd4d43df4dceddb3a05",
    "components/script-runtime-api/webrtc/bootstrap.js": "b16a9c2cc771628ab6800492789278dec6fccc524c8308e2d03272f3b77c1bdd"
  },
  "versions": [
    {
      "browser": "chrome",
      "version": "Chrome/156.0.8078.74",
      "headed": true,
      "visibleWindows": 1,
      "webrtcPreferenceOverrides": false
    }
  ],
  "cases": [
    {
      "browser": "chrome",
      "engine": "boa",
      "route": "direct",
      "offerer": "native",
      "cleanup": "close",
      "status": "fail",
      "phase": "negotiation and transfer",
      "error": "application exchange deadline"
    }
  ],
  "failure": {
    "phase": "negotiation and transfer",
    "case": {
      "browser": "chrome",
      "engine": "boa",
      "route": "direct",
      "offerer": "native",
      "cleanup": "close"
    },
    "error": "application exchange deadline"
  },
  "profilesRemoved": true,
  "scope": "native Windows to installed headed browsers, controlled WSL UDP server; no DNS/TCP/TLS/restart/media/carrier acceptance"
}
```


## E2h automated browser interoperability - 2026-11-02T18:56:19.299Z

```json
{
  "schema": 1,
  "gate": "E2h",
  "utc": "2026-11-02T18:56:19.299Z",
  "source": "42f9bc7df50e38669479fd37adc450e526959383",
  "node": "v24.11.0",
  "nativeBinarySHA256": "fdf5b358be3c0ddbd39d8d7822c6945ae806e5c7f9dd4c27d75b9e87ed96af3a",
  "sourceSHA256": {
    "components/genet-webrtc/examples/interop_host.rs": "ef63ed3a12dff896808373c2e41b0c37ef3540a4f199582c9163191f62f850d6",
    "support/ci/webrtc_interop.mjs": "996427ab02d9c17a2b31da2c876e50e4dc03a9b77384e22dfcd0274949d158e7",
    "support/ci/webrtc_interop_peer.js": "d89c77423b939c3126a8868816d1613970b446399e9ccc27a1c4546289b7b7da",
    "components/genet-webrtc/Cargo.toml": "5884b70af4d2794b2ff85bbfddd2523632b8a8b4ba66c093247eb98aa1882ec2",
    "components/genet-webrtc/src/lib.rs": "753e27024d2048812924c56158fa274af793aa9b17c6dbd4d43df4dceddb3a05",
    "components/script-runtime-api/webrtc/bootstrap.js": "b16a9c2cc771628ab6800492789278dec6fccc524c8308e2d03272f3b77c1bdd"
  },
  "versions": [
    {
      "browser": "chrome",
      "version": "Chrome/156.0.8078.74",
      "headed": true,
      "visibleWindows": 1,
      "webrtcPreferenceOverrides": false
    }
  ],
  "cases": [
    {
      "browser": "chrome",
      "engine": "boa",
      "route": "direct",
      "offerer": "native",
      "cleanup": "close",
      "status": "fail",
      "phase": "negotiation and transfer",
      "error": "application exchange deadline",
      "observed": {
        "native": {
          "connected": false,
          "connectionState": "failed",
          "iceState": "checking",
          "signaling": "stable",
          "remoteSet": true,
          "received": [],
          "sent": [],
          "modes": [],
          "errors": [],
          "refused": [],
          "channelStates": [
            "connecting",
            "connecting"
          ],
          "gathering": "complete"
        },
        "browser": {
          "connected": false,
          "connectionState": "failed",
          "iceState": "disconnected",
          "signaling": "stable",
          "remoteSet": true,
          "received": [],
          "sent": [],
          "modes": [],
          "errors": [],
          "refused": [],
          "channelStates": [],
          "gathering": "complete"
        }
      }
    }
  ],
  "failure": {
    "phase": "negotiation and transfer",
    "case": {
      "browser": "chrome",
      "engine": "boa",
      "route": "direct",
      "offerer": "native",
      "cleanup": "close"
    },
    "error": "application exchange deadline"
  },
  "profilesRemoved": true,
  "scope": "native Windows to installed headed browsers, controlled WSL UDP server; no DNS/TCP/TLS/restart/media/carrier acceptance"
}
```


## E2h automated browser interoperability - 2026-11-02T18:58:48.575Z

```json
{
  "schema": 1,
  "gate": "E2h",
  "utc": "2026-11-02T18:58:48.575Z",
  "source": "42f9bc7df50e38669479fd37adc450e526959383",
  "node": "v24.11.0",
  "nativeBinarySHA256": "74fea8ed043acbb00891b1844dcf4f7ffe3a1c4279b94154dad530e61a92829a",
  "sourceSHA256": {
    "components/genet-webrtc/examples/interop_host.rs": "ef63ed3a12dff896808373c2e41b0c37ef3540a4f199582c9163191f62f850d6",
    "support/ci/webrtc_interop.mjs": "5e6a2f36ae4a7c5dacc233575fb90b3cdef775e41b7b2502a5ce2157ac5dc0cf",
    "support/ci/webrtc_interop_peer.js": "d89c77423b939c3126a8868816d1613970b446399e9ccc27a1c4546289b7b7da",
    "components/genet-webrtc/Cargo.toml": "5884b70af4d2794b2ff85bbfddd2523632b8a8b4ba66c093247eb98aa1882ec2",
    "components/genet-webrtc/src/lib.rs": "6faf4af0e36324999333303d33027d68095d266dc087f03fe55ca42d2a8df971",
    "components/script-runtime-api/webrtc/bootstrap.js": "b16a9c2cc771628ab6800492789278dec6fccc524c8308e2d03272f3b77c1bdd"
  },
  "versions": [
    {
      "browser": "chrome",
      "version": "Chrome/156.0.8078.74",
      "headed": true,
      "visibleWindows": 1,
      "webrtcPreferenceOverrides": false
    }
  ],
  "cases": [
    {
      "browser": "chrome",
      "engine": "boa",
      "route": "direct",
      "offerer": "native",
      "cleanup": "close",
      "status": "fail",
      "phase": "negotiation and transfer",
      "error": "application exchange deadline",
      "observed": {
        "native": {
          "connected": false,
          "connectionState": "failed",
          "iceState": "checking",
          "signaling": "stable",
          "remoteSet": true,
          "received": [],
          "sent": [],
          "modes": [],
          "errors": [],
          "refused": [],
          "channelStates": [
            "connecting",
            "connecting"
          ],
          "gathering": "complete"
        },
        "browser": {
          "connected": false,
          "connectionState": "failed",
          "iceState": "disconnected",
          "signaling": "stable",
          "remoteSet": true,
          "received": [],
          "sent": [],
          "modes": [],
          "errors": [],
          "refused": [],
          "channelStates": [],
          "gathering": "complete"
        }
      },
      "observedStats": {
        "native": {
          "pair": null,
          "localFingerprint": "d94a038ec473969d48ca3e02c43c4032d046687f689db1cb7a2300f2e15a996a",
          "remoteFingerprint": "c7588b59db8fa24aae731e1559ba1386a4d239ac7a84b5cf19c46e08b71a5474",
          "remoteCertificates": [],
          "statsRemoteFingerprint": null
        },
        "browser": {
          "pair": null,
          "localFingerprint": "c7588b59db8fa24aae731e1559ba1386a4d239ac7a84b5cf19c46e08b71a5474",
          "remoteFingerprint": "d94a038ec473969d48ca3e02c43c4032d046687f689db1cb7a2300f2e15a996a",
          "remoteCertificates": [],
          "statsRemoteFingerprint": null
        }
      },
      "trace": [
        "RTC poll variant: Route",
        "RTC poll variant: Route",
        "RTC poll variant: Route",
        "RTC poll variant: Route",
        "RTC poll variant: Route",
        "RTC poll variant: Route",
        "RTC poll variant: Route",
        "RTC poll variant: Route"
      ]
    }
  ],
  "failure": {
    "phase": "negotiation and transfer",
    "case": {
      "browser": "chrome",
      "engine": "boa",
      "route": "direct",
      "offerer": "native",
      "cleanup": "close"
    },
    "error": "application exchange deadline"
  },
  "profilesRemoved": true,
  "scope": "native Windows to installed headed browsers, controlled WSL UDP server; no DNS/TCP/TLS/restart/media/carrier acceptance"
}
```


## E2h automated browser interoperability - 2026-11-02T19:00:26.467Z

```json
{
  "schema": 1,
  "gate": "E2h",
  "utc": "2026-11-02T19:00:26.467Z",
  "source": "42f9bc7df50e38669479fd37adc450e526959383",
  "node": "v24.11.0",
  "nativeBinarySHA256": "74fea8ed043acbb00891b1844dcf4f7ffe3a1c4279b94154dad530e61a92829a",
  "sourceSHA256": {
    "components/genet-webrtc/examples/interop_host.rs": "ef63ed3a12dff896808373c2e41b0c37ef3540a4f199582c9163191f62f850d6",
    "support/ci/webrtc_interop.mjs": "5e6a2f36ae4a7c5dacc233575fb90b3cdef775e41b7b2502a5ce2157ac5dc0cf",
    "support/ci/webrtc_interop_peer.js": "a0b6896eb00e7782c5f0942a1249bc64cc2b035771f2097fccbf99c7e09c673b",
    "components/genet-webrtc/Cargo.toml": "5884b70af4d2794b2ff85bbfddd2523632b8a8b4ba66c093247eb98aa1882ec2",
    "components/genet-webrtc/src/lib.rs": "6faf4af0e36324999333303d33027d68095d266dc087f03fe55ca42d2a8df971",
    "components/script-runtime-api/webrtc/bootstrap.js": "b16a9c2cc771628ab6800492789278dec6fccc524c8308e2d03272f3b77c1bdd"
  },
  "versions": [
    {
      "browser": "chrome",
      "version": "Chrome/156.0.8078.74",
      "headed": true,
      "visibleWindows": 1,
      "webrtcPreferenceOverrides": false
    }
  ],
  "cases": [
    {
      "browser": "chrome",
      "engine": "boa",
      "route": "relay",
      "offerer": "native",
      "cleanup": "close",
      "status": "fail",
      "phase": "route and certificate verification",
      "error": "unexpected selected transport",
      "observed": {
        "native": {
          "connected": true,
          "connectionState": "connected",
          "iceState": "connected",
          "signaling": "stable",
          "remoteSet": true,
          "received": [
            "browser:text:λ🙂:reliable",
            "binary:48000",
            "browser:text:λ🙂:unreliable"
          ],
          "sent": [
            "reliable:text",
            "reliable:binary",
            "unreliable:text"
          ],
          "modes": [
            {
              "label": "reliable",
              "ordered": true,
              "maxRetransmits": null
            },
            {
              "label": "unreliable",
              "ordered": false,
              "maxRetransmits": 0
            }
          ],
          "errors": [],
          "refused": [],
          "remoteProtocols": [
            "udp"
          ],
          "channelStates": [
            "open",
            "open"
          ],
          "gathering": "complete"
        },
        "browser": {
          "connected": true,
          "connectionState": "connected",
          "iceState": "connected",
          "signaling": "stable",
          "remoteSet": true,
          "received": [
            "native:text:λ🙂:reliable",
            "binary:48000",
            "native:text:λ🙂:unreliable"
          ],
          "sent": [
            "reliable:text",
            "reliable:binary",
            "unreliable:text"
          ],
          "modes": [
            {
              "label": "reliable",
              "ordered": true,
              "maxRetransmits": null
            },
            {
              "label": "unreliable",
              "ordered": false,
              "maxRetransmits": 0
            }
          ],
          "errors": [],
          "refused": [],
          "remoteProtocols": [
            "udp"
          ],
          "channelStates": [
            "open",
            "open"
          ],
          "gathering": "complete"
        }
      },
      "observedStats": {
        "native": {
          "pair": {
            "state": "succeeded",
            "nominated": true
          },
          "localFingerprint": "3c3836c967533e8facd293d8a7000b888f055d4fecca55cdf066ba99aaceac28",
          "remoteFingerprint": "b8934cd21859c282fa804d0de8bfb1f50152740f77f0f0197065a395fbfde3db",
          "remoteCertificates": [
            [
              48,
              130,
              1,
              22,
              48,
              129,
              189,
              160,
              3,
              2,
              1,
              2,
              2,
              9,
              0,
              240,
              121,
              82,
              27,
              162,
              176,
              230,
              75,
              48,
              10,
              6,
              8,
              42,
              134,
              72,
              206,
              61,
              4,
              3,
              2,
              48,
              17,
              49,
              15,
              48,
              13,
              6,
              3,
              85,
              4,
              3,
              12,
              6,
              87,
              101,
              98,
              82,
              84,
              67,
              48,
              30,
              23,
              13,
              50,
              54,
              49,
              49,
              48,
              49,
              49,
              57,
              48,
              48,
              49,
              57,
              90,
              23,
              13,
              50,
              54,
              49,
              50,
              48,
              50,
              49,
              57,
              48,
              48,
              49,
              57,
              90,
              48,
              17,
              49,
              15,
              48,
              13,
              6,
              3,
              85,
              4,
              3,
              12,
              6,
              87,
              101,
              98,
              82,
              84,
              67,
              48,
              89,
              48,
              19,
              6,
              7,
              42,
              134,
              72,
              206,
              61,
              2,
              1,
              6,
              8,
              42,
              134,
              72,
              206,
              61,
              3,
              1,
              7,
              3,
              66,
              0,
              4,
              204,
              224,
              227,
              69,
              240,
              223,
              196,
              23,
              240,
              178,
              39,
              246,
              81,
              156,
              24,
              212,
              239,
              158,
              116,
              219,
              136,
              181,
              103,
              112,
              32,
              129,
              219,
              16,
              80,
              53,
              95,
              17,
              118,
              81,
              158,
              172,
              144,
              54,
              37,
              184,
              211,
              72,
              109,
              166,
              42,
              36,
              117,
              199,
              240,
              66,
              122,
              79,
              89,
              221,
              252,
              190,
              242,
              147,
              228,
              3,
              54,
              21,
              11,
              61,
              48,
              10,
              6,
              8,
              42,
              134,
              72,
              206,
              61,
              4,
              3,
              2,
              3,
              72,
              0,
              48,
              69,
              2,
              32,
              67,
              251,
              12,
              21,
              112,
              74,
              158,
              192,
              47,
              57,
              14,
              247,
              56,
              142,
              223,
              111,
              196,
              190,
              87,
              208,
              123,
              36,
              226,
              103,
              161,
              169,
              105,
              154,
              137,
              18,
              8,
              10,
              2,
              33,
              0,
              224,
              22,
              10,
              99,
              205,
              53,
              111,
              167,
              66,
              105,
              72,
              197,
              181,
              213,
              237,
              3,
              6,
              115,
              217,
              115,
              221,
              28,
              24,
              175,
              222,
              189,
              192,
              224,
              218,
              117,
              119,
              166
            ]
          ],
          "statsRemoteFingerprint": "b8934cd21859c282fa804d0de8bfb1f50152740f77f0f0197065a395fbfde3db"
        },
        "browser": {
          "pair": {
            "state": "succeeded",
            "nominated": true,
            "local": "relay",
            "remote": "relay",
            "localProtocol": "udp",
            "remoteProtocol": "udp"
          },
          "localFingerprint": "b8934cd21859c282fa804d0de8bfb1f50152740f77f0f0197065a395fbfde3db",
          "remoteFingerprint": "3c3836c967533e8facd293d8a7000b888f055d4fecca55cdf066ba99aaceac28",
          "remoteCertificates": [
            [
              48,
              130,
              1,
              101,
              48,
              130,
              1,
              11,
              160,
              3,
              2,
              1,
              2,
              2,
              20,
              43,
              159,
              142,
              153,
              153,
              14,
              120,
              177,
              255,
              205,
              152,
              156,
              165,
              89,
              157,
              195,
              168,
              133,
              55,
              119,
              48,
              10,
              6,
              8,
              42,
              134,
              72,
              206,
              61,
              4,
              3,
              2,
              48,
              33,
              49,
              31,
              48,
              29,
              6,
              3,
              85,
              4,
              3,
              12,
              22,
              114,
              99,
              103,
              101,
              110,
              32,
              115,
              101,
              108,
              102,
              32,
              115,
              105,
              103,
              110,
              101,
              100,
              32,
              99,
              101,
              114,
              116,
              48,
              32,
              23,
              13,
              55,
              53,
              48,
              49,
              48,
              49,
              48,
              48,
              48,
              48,
              48,
              48,
              90,
              24,
              15,
              52,
              48,
              57,
              54,
              48,
              49,
              48,
              49,
              48,
              48,
              48,
              48,
              48,
              48,
              90,
              48,
              33,
              49,
              31,
              48,
              29,
              6,
              3,
              85,
              4,
              3,
              12,
              22,
              114,
              99,
              103,
              101,
              110,
              32,
              115,
              101,
              108,
              102,
              32,
              115,
              105,
              103,
              110,
              101,
              100,
              32,
              99,
              101,
              114,
              116,
              48,
              89,
              48,
              19,
              6,
              7,
              42,
              134,
              72,
              206,
              61,
              2,
              1,
              6,
              8,
              42,
              134,
              72,
              206,
              61,
              3,
              1,
              7,
              3,
              66,
              0,
              4,
              11,
              138,
              17,
              237,
              25,
              209,
              166,
              12,
              241,
              154,
              222,
              62,
              217,
              153,
              42,
              13,
              73,
              39,
              6,
              209,
              232,
              83,
              138,
              152,
              227,
              115,
              16,
              247,
              124,
              253,
              216,
              5,
              92,
              140,
              143,
              150,
              161,
              72,
              15,
              181,
              182,
              102,
              71,
              210,
              254,
              84,
              100,
              23,
              64,
              220,
              122,
              154,
              77,
              96,
              111,
              160,
              92,
              122,
              255,
              208,
              166,
              199,
              95,
              52,
              163,
              31,
              48,
              29,
              48,
              27,
              6,
              3,
              85,
              29,
              17,
              4,
              20,
              48,
              18,
              130,
              16,
              107,
              109,
              88,
              110,
              86,
              104,
              121,
              76,
              102,
              116,
              72,
              90,
              79,
              102,
              99,
              111,
              48,
              10,
              6,
              8,
              42,
              134,
              72,
              206,
              61,
              4,
              3,
              2,
              3,
              72,
              0,
              48,
              69,
              2,
              32,
              74,
              193,
              231,
              174,
              238,
              97,
              72,
              35,
              166,
              156,
              104,
              72,
              224,
              148,
              243,
              126,
              212,
              120,
              94,
              70,
              134,
              111,
              138,
              122,
              229,
              252,
              74,
              169,
              28,
              178,
              96,
              235,
              2,
              33,
              0,
              202,
              164,
              37,
              67,
              233,
              147,
              83,
              20,
              163,
              71,
              68,
              191,
              133,
              254,
              145,
              78,
              45,
              134,
              81,
              53,
              159,
              0,
              171,
              146,
              159,
              11,
              203,
              193,
              43,
              34,
              68,
              158
            ]
          ],
          "statsRemoteFingerprint": "3c3836c967533e8facd293d8a7000b888f055d4fecca55cdf066ba99aaceac28"
        }
      },
      "trace": []
    }
  ],
  "failure": {
    "phase": "route and certificate verification",
    "case": {
      "browser": "chrome",
      "engine": "boa",
      "route": "relay",
      "offerer": "native",
      "cleanup": "close"
    },
    "error": "unexpected selected transport"
  },
  "profilesRemoved": true,
  "scope": "native Windows to installed headed browsers, controlled WSL UDP server; no DNS/TCP/TLS/restart/media/carrier acceptance"
}
```


## E2h automated browser interoperability - 2026-11-02T19:01:47.590Z

```json
{
  "schema": 1,
  "gate": "E2h",
  "utc": "2026-11-02T19:01:47.590Z",
  "source": "42f9bc7df50e38669479fd37adc450e526959383",
  "node": "v24.11.0",
  "nativeBinarySHA256": "74fea8ed043acbb00891b1844dcf4f7ffe3a1c4279b94154dad530e61a92829a",
  "sourceSHA256": {
    "components/genet-webrtc/examples/interop_host.rs": "ef63ed3a12dff896808373c2e41b0c37ef3540a4f199582c9163191f62f850d6",
    "support/ci/webrtc_interop.mjs": "5e6a2f36ae4a7c5dacc233575fb90b3cdef775e41b7b2502a5ce2157ac5dc0cf",
    "support/ci/webrtc_interop_peer.js": "35ab1f8c7f74c4ff550400617a1b86a4f61a0e2a93d63263b5930fb5c04462f7",
    "components/genet-webrtc/Cargo.toml": "5884b70af4d2794b2ff85bbfddd2523632b8a8b4ba66c093247eb98aa1882ec2",
    "components/genet-webrtc/src/lib.rs": "6faf4af0e36324999333303d33027d68095d266dc087f03fe55ca42d2a8df971",
    "components/script-runtime-api/webrtc/bootstrap.js": "b16a9c2cc771628ab6800492789278dec6fccc524c8308e2d03272f3b77c1bdd"
  },
  "versions": [
    {
      "browser": "chrome",
      "version": "Chrome/156.0.8078.74",
      "headed": true,
      "visibleWindows": 1,
      "webrtcPreferenceOverrides": false
    }
  ],
  "cases": [
    {
      "browser": "chrome",
      "engine": "boa",
      "route": "direct",
      "offerer": "native",
      "cleanup": "close",
      "status": "fail",
      "phase": "peer startup",
      "error": "native control deadline",
      "observed": {},
      "trace": []
    }
  ],
  "failure": {
    "phase": "peer startup",
    "case": {
      "browser": "chrome",
      "engine": "boa",
      "route": "direct",
      "offerer": "native",
      "cleanup": "close"
    },
    "error": "native control deadline"
  },
  "profilesRemoved": true,
  "scope": "native Windows to installed headed browsers, controlled WSL UDP server; no DNS/TCP/TLS/restart/media/carrier acceptance"
}
```


## E2h automated browser interoperability - 2026-11-02T19:05:01.749Z

```json
{
  "schema": 1,
  "gate": "E2h",
  "utc": "2026-11-02T19:05:01.749Z",
  "source": "42f9bc7df50e38669479fd37adc450e526959383",
  "node": "v24.11.0",
  "nativeBinarySHA256": "74fea8ed043acbb00891b1844dcf4f7ffe3a1c4279b94154dad530e61a92829a",
  "sourceSHA256": {
    "components/genet-webrtc/examples/interop_host.rs": "ef63ed3a12dff896808373c2e41b0c37ef3540a4f199582c9163191f62f850d6",
    "support/ci/webrtc_interop.mjs": "cadd2e4c947b4b3fd38d1ab8596c9fef348ec4a6947fbc9c4a7c7ebbbb395380",
    "support/ci/webrtc_interop_peer.js": "35ab1f8c7f74c4ff550400617a1b86a4f61a0e2a93d63263b5930fb5c04462f7",
    "components/genet-webrtc/Cargo.toml": "5884b70af4d2794b2ff85bbfddd2523632b8a8b4ba66c093247eb98aa1882ec2",
    "components/genet-webrtc/src/lib.rs": "6faf4af0e36324999333303d33027d68095d266dc087f03fe55ca42d2a8df971",
    "components/script-runtime-api/webrtc/bootstrap.js": "b16a9c2cc771628ab6800492789278dec6fccc524c8308e2d03272f3b77c1bdd"
  },
  "versions": [
    {
      "browser": "chrome",
      "version": "Chrome/156.0.8078.74",
      "headed": true,
      "visibleWindows": 1,
      "webrtcPreferenceOverrides": false
    }
  ],
  "cases": [
    {
      "browser": "chrome",
      "engine": "boa",
      "route": "relay",
      "offerer": "native",
      "cleanup": "close",
      "status": "fail",
      "phase": "peer startup",
      "error": "native control deadline",
      "observed": {},
      "trace": []
    }
  ],
  "failure": {
    "phase": "peer startup",
    "case": {
      "browser": "chrome",
      "engine": "boa",
      "route": "relay",
      "offerer": "native",
      "cleanup": "close"
    },
    "error": "Command failed: taskkill /PID 30704 /T /F"
  },
  "profilesRemoved": true,
  "scope": "native Windows to installed headed browsers, controlled WSL UDP server; no DNS/TCP/TLS/restart/media/carrier acceptance"
}
```


## E2h automated browser interoperability - 2026-11-02T19:11:06.076Z

```json
{
  "schema": 1,
  "gate": "E2h",
  "utc": "2026-11-02T19:11:06.076Z",
  "source": "42f9bc7df50e38669479fd37adc450e526959383",
  "node": "v24.11.0",
  "nativeBinarySHA256": "1c3fdec953713f5be5ecfdb6cf3ac69ebbee4152e21e080c5191af4be01dce43",
  "sourceSHA256": {
    "components/genet-webrtc/examples/interop_host.rs": "de4f114e285f3d0117f3906d57b46213fa49f597916f5039745673bb9447a79c",
    "support/ci/webrtc_interop.mjs": "589d126a60bfd4b264fbbe76634209f6cd18cc54d2043474fa1be8e0bc1e507e",
    "support/ci/webrtc_interop_peer.js": "35ab1f8c7f74c4ff550400617a1b86a4f61a0e2a93d63263b5930fb5c04462f7",
    "components/genet-webrtc/Cargo.toml": "a0c7b33c3e49b10ba972f67c1169c674046cb6c5e717caa9f6ed2d1c0e3a4265",
    "components/genet-webrtc/src/lib.rs": "d691ee2a7786d2e92f0198f026653506536b1f9caa7b55f31bbca49cddafa993",
    "components/script-runtime-api/webrtc/bootstrap.js": "b16a9c2cc771628ab6800492789278dec6fccc524c8308e2d03272f3b77c1bdd"
  },
  "versions": [
    {
      "browser": "chrome",
      "version": "Chrome/156.0.8078.74",
      "headed": true,
      "visibleWindows": 1,
      "webrtcPreferenceOverrides": false
    }
  ],
  "cases": [
    {
      "browser": "chrome",
      "engine": "boa",
      "route": "direct",
      "offerer": "native",
      "cleanup": "close",
      "status": "fail",
      "phase": "negotiation and transfer",
      "error": "application exchange deadline",
      "observed": {
        "native": {
          "connected": false,
          "connectionState": "failed",
          "iceState": "checking",
          "signaling": "stable",
          "remoteSet": true,
          "received": [],
          "sent": [],
          "modes": [],
          "errors": [],
          "refused": [],
          "remoteProtocols": [
            "udp",
            "udp",
            "udp"
          ],
          "channelStates": [
            "connecting",
            "connecting"
          ],
          "gathering": "complete"
        },
        "browser": {
          "connected": false,
          "connectionState": "failed",
          "iceState": "disconnected",
          "signaling": "stable",
          "remoteSet": true,
          "received": [],
          "sent": [],
          "modes": [],
          "errors": [],
          "refused": [],
          "remoteProtocols": [
            "udp",
            "udp"
          ],
          "channelStates": [],
          "gathering": "complete"
        }
      },
      "observedStats": {
        "native": {
          "pair": null,
          "localFingerprint": "ddffbaea352746049f390dc57894046dce462c668206d5c3abd9a44b6c0268ff",
          "remoteFingerprint": "280a1c9149c4f6558a47db9c30466c736d97e392f89b6e3a99b5060c9c5412dc",
          "statsRemoteFingerprint": null,
          "pairKeys": null,
          "candidateKeys": [
            {
              "id": "RTCLocalIceCandidate_candidate:zgwmN4ppMQv4bXhqje5cjNR0zCDc29EX",
              "type": "local-candidate",
              "candidateType": "host",
              "protocol": "udp"
            },
            {
              "id": "RTCLocalIceCandidate_candidate:Uvr77LEGdwS0U8vYGq7x1CcR9PWLUUXN",
              "type": "local-candidate",
              "candidateType": "srflx",
              "protocol": "udp"
            },
            {
              "id": "RTCRemoteIceCandidate_candidate:wfGv0pwXRqhLyyuXW2WJzw98g0ezUZKS",
              "type": "remote-candidate",
              "candidateType": "srflx",
              "protocol": "udp"
            },
            {
              "id": "RTCRemoteIceCandidate_candidate:Y9syk//qab5u1fepXwjdPIwa8S6pqI+Z",
              "type": "remote-candidate",
              "candidateType": "host",
              "protocol": "udp"
            },
            {
              "id": "RTCRemoteIceCandidate_candidate:8FxXq7DFLH7MzDexTPphpFul9BnGF9ey",
              "type": "remote-candidate",
              "candidateType": "host",
              "protocol": "udp"
            }
          ],
          "remoteDER_SHA256": []
        },
        "browser": {
          "pair": null,
          "localFingerprint": "280a1c9149c4f6558a47db9c30466c736d97e392f89b6e3a99b5060c9c5412dc",
          "remoteFingerprint": "ddffbaea352746049f390dc57894046dce462c668206d5c3abd9a44b6c0268ff",
          "statsRemoteFingerprint": null,
          "pairKeys": null,
          "candidateKeys": [
            {
              "id": "ICCzyxrEx",
              "type": "local-candidate",
              "candidateType": "host",
              "protocol": "udp"
            },
            {
              "id": "IQKQMLWX/",
              "type": "local-candidate",
              "candidateType": "host",
              "protocol": "udp"
            },
            {
              "id": "IfHkwLa9O",
              "type": "local-candidate",
              "candidateType": "srflx",
              "protocol": "udp"
            }
          ],
          "remoteDER_SHA256": []
        }
      },
      "trace": [
        "RTC poll variant: Route { transport: TransportContext { local_addr: 0.0.0.0:5353, peer_addr: 224.0.0.251:5353, transport_protocol: UDP, ecn: None } }"
      ]
    }
  ],
  "failure": {
    "phase": "negotiation and transfer",
    "case": {
      "browser": "chrome",
      "engine": "boa",
      "route": "direct",
      "offerer": "native",
      "cleanup": "close"
    },
    "error": "application exchange deadline"
  },
  "profileCache": "C:\\t\\cargo-targets\\genet\\rtc-interop",
  "profileCleanup": "automatic approval review blocked recursive deletion; marked profiles retained for reuse",
  "scope": "native Windows to installed headed browsers, controlled WSL UDP server; no DNS/TCP/TLS/restart/media/carrier acceptance"
}
```


## E2h automated browser interoperability - 2026-11-02T19:11:54.803Z

```json
{
  "schema": 1,
  "gate": "E2h",
  "utc": "2026-11-02T19:11:54.803Z",
  "source": "42f9bc7df50e38669479fd37adc450e526959383",
  "node": "v24.11.0",
  "nativeBinarySHA256": "1c3fdec953713f5be5ecfdb6cf3ac69ebbee4152e21e080c5191af4be01dce43",
  "sourceSHA256": {
    "components/genet-webrtc/examples/interop_host.rs": "de4f114e285f3d0117f3906d57b46213fa49f597916f5039745673bb9447a79c",
    "support/ci/webrtc_interop.mjs": "589d126a60bfd4b264fbbe76634209f6cd18cc54d2043474fa1be8e0bc1e507e",
    "support/ci/webrtc_interop_peer.js": "35ab1f8c7f74c4ff550400617a1b86a4f61a0e2a93d63263b5930fb5c04462f7",
    "components/genet-webrtc/Cargo.toml": "a0c7b33c3e49b10ba972f67c1169c674046cb6c5e717caa9f6ed2d1c0e3a4265",
    "components/genet-webrtc/src/lib.rs": "d691ee2a7786d2e92f0198f026653506536b1f9caa7b55f31bbca49cddafa993",
    "components/script-runtime-api/webrtc/bootstrap.js": "b16a9c2cc771628ab6800492789278dec6fccc524c8308e2d03272f3b77c1bdd"
  },
  "versions": [
    {
      "browser": "chrome",
      "version": "Chrome/156.0.8078.74",
      "headed": true,
      "visibleWindows": 1,
      "webrtcPreferenceOverrides": false
    }
  ],
  "cases": [
    {
      "browser": "chrome",
      "engine": "boa",
      "route": "relay",
      "offerer": "native",
      "cleanup": "close",
      "status": "pass",
      "seconds": 18.33,
      "native": {
        "connected": true,
        "connectionState": "connected",
        "iceState": "connected",
        "signaling": "stable",
        "remoteSet": true,
        "received": [
          "browser:text:λ🙂:unreliable",
          "browser:text:λ🙂:reliable",
          "binary:48000"
        ],
        "sent": [
          "unreliable:text",
          "reliable:text",
          "reliable:binary"
        ],
        "modes": [
          {
            "label": "unreliable",
            "ordered": false,
            "maxRetransmits": 0
          },
          {
            "label": "reliable",
            "ordered": true,
            "maxRetransmits": null
          }
        ],
        "errors": [],
        "refused": [],
        "remoteProtocols": [
          "udp"
        ],
        "channelStates": [
          "open",
          "open"
        ],
        "gathering": "complete",
        "stats": {
          "pair": {
            "state": "succeeded",
            "nominated": true,
            "local": "relay",
            "remote": "relay",
            "localProtocol": "udp",
            "remoteProtocol": "udp"
          },
          "localFingerprint": "ef5993bfaf27f622a95549180bb7a6c32d2750b7b4c61810e045d597863e607b",
          "remoteFingerprint": "fe5477591a878daeceecac19776cb460126adf98de2b3ad7a7e0b2bfe5802f65",
          "statsRemoteFingerprint": "fe5477591a878daeceecac19776cb460126adf98de2b3ad7a7e0b2bfe5802f65",
          "pairKeys": {
            "local": "RTCLocalIceCandidate_candidate:me0d/rPhm2RaIJlQgN1NW7Y2UrhmtDeb",
            "remote": "RTCRemoteIceCandidate_candidate:UtOd41UH7WDD3WUrYT7DptoGHZ9bIAyd"
          },
          "candidateKeys": [
            {
              "id": "RTCLocalIceCandidate_candidate:me0d/rPhm2RaIJlQgN1NW7Y2UrhmtDeb",
              "type": "local-candidate",
              "candidateType": "relay",
              "protocol": "udp"
            },
            {
              "id": "RTCRemoteIceCandidate_candidate:UtOd41UH7WDD3WUrYT7DptoGHZ9bIAyd",
              "type": "remote-candidate",
              "candidateType": "relay",
              "protocol": "udp"
            }
          ],
          "remoteDER_SHA256": [
            "fe5477591a878daeceecac19776cb460126adf98de2b3ad7a7e0b2bfe5802f65"
          ]
        }
      },
      "browserPeer": {
        "connected": true,
        "connectionState": "connected",
        "iceState": "connected",
        "signaling": "stable",
        "remoteSet": true,
        "received": [
          "native:text:λ🙂:unreliable",
          "native:text:λ🙂:reliable",
          "binary:48000"
        ],
        "sent": [
          "unreliable:text",
          "reliable:text",
          "reliable:binary"
        ],
        "modes": [
          {
            "label": "unreliable",
            "ordered": false,
            "maxRetransmits": 0
          },
          {
            "label": "reliable",
            "ordered": true,
            "maxRetransmits": null
          }
        ],
        "errors": [],
        "refused": [],
        "remoteProtocols": [
          "udp"
        ],
        "channelStates": [
          "open",
          "open"
        ],
        "gathering": "complete",
        "stats": {
          "pair": {
            "state": "succeeded",
            "nominated": true,
            "local": "relay",
            "remote": "relay",
            "localProtocol": "udp",
            "remoteProtocol": "udp"
          },
          "localFingerprint": "fe5477591a878daeceecac19776cb460126adf98de2b3ad7a7e0b2bfe5802f65",
          "remoteFingerprint": "ef5993bfaf27f622a95549180bb7a6c32d2750b7b4c61810e045d597863e607b",
          "statsRemoteFingerprint": "ef5993bfaf27f622a95549180bb7a6c32d2750b7b4c61810e045d597863e607b",
          "pairKeys": {
            "local": "IEUpfiagg",
            "remote": "I7Zoasvzo"
          },
          "candidateKeys": [
            {
              "id": "I7Zoasvzo",
              "type": "remote-candidate",
              "candidateType": "relay",
              "protocol": "udp"
            },
            {
              "id": "IEUpfiagg",
              "type": "local-candidate",
              "candidateType": "relay",
              "protocol": "udp"
            }
          ],
          "remoteDER_SHA256": [
            "ef5993bfaf27f622a95549180bb7a6c32d2750b7b4c61810e045d597863e607b"
          ]
        }
      },
      "cleanupVerified": true,
      "socketRebound": true
    }
  ],
  "failure": null,
  "profileCache": "C:\\t\\cargo-targets\\genet\\rtc-interop",
  "profileCleanup": "automatic approval review blocked recursive deletion; marked profiles retained for reuse",
  "scope": "native Windows to installed headed browsers, controlled WSL UDP server; no DNS/TCP/TLS/restart/media/carrier acceptance"
}
```


## E2h automated browser interoperability - 2026-11-02T19:12:59.614Z

```json
{
  "schema": 1,
  "gate": "E2h",
  "utc": "2026-11-02T19:12:59.614Z",
  "source": "42f9bc7df50e38669479fd37adc450e526959383",
  "node": "v24.11.0",
  "nativeBinarySHA256": "8e893c219a5352aa0395483de04b62e2aab0520463e88e879a7137da564ff0bd",
  "sourceSHA256": {
    "components/genet-webrtc/examples/interop_host.rs": "de4f114e285f3d0117f3906d57b46213fa49f597916f5039745673bb9447a79c",
    "support/ci/webrtc_interop.mjs": "589d126a60bfd4b264fbbe76634209f6cd18cc54d2043474fa1be8e0bc1e507e",
    "support/ci/webrtc_interop_peer.js": "35ab1f8c7f74c4ff550400617a1b86a4f61a0e2a93d63263b5930fb5c04462f7",
    "components/genet-webrtc/Cargo.toml": "a0c7b33c3e49b10ba972f67c1169c674046cb6c5e717caa9f6ed2d1c0e3a4265",
    "components/genet-webrtc/src/lib.rs": "83a99e4f364c7892cd46acb2800675c95baaecb426e2395a06fbe4f154c75864",
    "components/script-runtime-api/webrtc/bootstrap.js": "b16a9c2cc771628ab6800492789278dec6fccc524c8308e2d03272f3b77c1bdd"
  },
  "versions": [
    {
      "browser": "chrome",
      "version": "Chrome/156.0.8078.74",
      "headed": true,
      "visibleWindows": 1,
      "webrtcPreferenceOverrides": false
    }
  ],
  "cases": [
    {
      "browser": "chrome",
      "engine": "boa",
      "route": "direct",
      "offerer": "native",
      "cleanup": "close",
      "status": "fail",
      "phase": "route and certificate verification",
      "error": "unexpected selected transport",
      "observed": {
        "native": {
          "connected": true,
          "connectionState": "connected",
          "iceState": "connected",
          "signaling": "stable",
          "remoteSet": true,
          "received": [
            "browser:text:λ🙂:unreliable",
            "browser:text:λ🙂:reliable",
            "binary:48000"
          ],
          "sent": [
            "unreliable:text",
            "reliable:text",
            "reliable:binary"
          ],
          "modes": [
            {
              "label": "unreliable",
              "ordered": false,
              "maxRetransmits": 0
            },
            {
              "label": "reliable",
              "ordered": true,
              "maxRetransmits": null
            }
          ],
          "errors": [],
          "refused": [
            "mdns:OperationError",
            "mdns:OperationError"
          ],
          "remoteProtocols": [
            "udp",
            "udp",
            "udp"
          ],
          "channelStates": [
            "open",
            "open"
          ],
          "gathering": "complete"
        },
        "browser": {
          "connected": true,
          "connectionState": "connected",
          "iceState": "connected",
          "signaling": "stable",
          "remoteSet": true,
          "received": [
            "native:text:λ🙂:unreliable",
            "native:text:λ🙂:reliable",
            "binary:48000"
          ],
          "sent": [
            "unreliable:text",
            "reliable:text",
            "reliable:binary"
          ],
          "modes": [
            {
              "label": "unreliable",
              "ordered": false,
              "maxRetransmits": 0
            },
            {
              "label": "reliable",
              "ordered": true,
              "maxRetransmits": null
            }
          ],
          "errors": [],
          "refused": [],
          "remoteProtocols": [
            "udp",
            "udp"
          ],
          "channelStates": [
            "open",
            "open"
          ],
          "gathering": "complete"
        }
      },
      "observedStats": {
        "native": {
          "pair": {
            "state": "succeeded",
            "nominated": true,
            "local": "host",
            "localProtocol": "udp"
          },
          "localFingerprint": "26f8b966316eccc90a83b23d997501abd3e7fdf2a80e43a3f8a96c6c08770825",
          "remoteFingerprint": "e19773b8ae67172d4e1633fbc6b98e3565cb0354d1d78eb6fd245a0457313812",
          "statsRemoteFingerprint": "e19773b8ae67172d4e1633fbc6b98e3565cb0354d1d78eb6fd245a0457313812",
          "pairKeys": {
            "local": "RTCLocalIceCandidate_candidate:fkYaVjzNUGIYnxfWlOAPAlAui9Y1FRE9",
            "remote": "candidate:I0BzaaizTRtDxQegSJ3RZGnfPE4uxJJe"
          },
          "candidateKeys": [
            {
              "id": "RTCLocalIceCandidate_candidate:49HPurIghXfslMA3xaTEBNXdu6rfnaWa",
              "type": "local-candidate",
              "candidateType": "srflx",
              "protocol": "udp"
            },
            {
              "id": "RTCLocalIceCandidate_candidate:fkYaVjzNUGIYnxfWlOAPAlAui9Y1FRE9",
              "type": "local-candidate",
              "candidateType": "host",
              "protocol": "udp"
            },
            {
              "id": "RTCRemoteIceCandidate_candidate:yNAVxLHZTrWhWsfLu0sjUow+OPY21wbc",
              "type": "remote-candidate",
              "candidateType": "srflx",
              "protocol": "udp"
            }
          ],
          "remoteDER_SHA256": [
            "e19773b8ae67172d4e1633fbc6b98e3565cb0354d1d78eb6fd245a0457313812"
          ]
        },
        "browser": {
          "pair": {
            "state": "succeeded",
            "nominated": true,
            "local": "srflx",
            "remote": "host",
            "localProtocol": "udp",
            "remoteProtocol": "udp"
          },
          "localFingerprint": "e19773b8ae67172d4e1633fbc6b98e3565cb0354d1d78eb6fd245a0457313812",
          "remoteFingerprint": "26f8b966316eccc90a83b23d997501abd3e7fdf2a80e43a3f8a96c6c08770825",
          "statsRemoteFingerprint": "26f8b966316eccc90a83b23d997501abd3e7fdf2a80e43a3f8a96c6c08770825",
          "pairKeys": {
            "local": "IzfQaCNuq",
            "remote": "IhET9v7sp"
          },
          "candidateKeys": [
            {
              "id": "IQKSr5mAR",
              "type": "local-candidate",
              "candidateType": "host",
              "protocol": "udp"
            },
            {
              "id": "IbouJvO0q",
              "type": "local-candidate",
              "candidateType": "host",
              "protocol": "udp"
            },
            {
              "id": "IhET9v7sp",
              "type": "remote-candidate",
              "candidateType": "host",
              "protocol": "udp"
            },
            {
              "id": "IzfQaCNuq",
              "type": "local-candidate",
              "candidateType": "srflx",
              "protocol": "udp"
            }
          ],
          "remoteDER_SHA256": [
            "26f8b966316eccc90a83b23d997501abd3e7fdf2a80e43a3f8a96c6c08770825"
          ]
        }
      },
      "trace": []
    }
  ],
  "failure": {
    "phase": "route and certificate verification",
    "case": {
      "browser": "chrome",
      "engine": "boa",
      "route": "direct",
      "offerer": "native",
      "cleanup": "close"
    },
    "error": "unexpected selected transport"
  },
  "profileCache": "C:\\t\\cargo-targets\\genet\\rtc-interop",
  "profileCleanup": "automatic approval review blocked recursive deletion; marked profiles retained for reuse",
  "scope": "native Windows to installed headed browsers, controlled WSL UDP server; no DNS/TCP/TLS/restart/media/carrier acceptance"
}
```


## E2h automated browser interoperability - 2026-11-02T19:15:22.361Z

```json
{
  "schema": 1,
  "gate": "E2h",
  "utc": "2026-11-02T19:15:22.361Z",
  "source": "42f9bc7df50e38669479fd37adc450e526959383",
  "node": "v24.11.0",
  "nativeBinarySHA256": "32f4556b304ad51e9e0f72ad5dca3f8d03bbf94ed16a0ddb19a5e0d35c21b389",
  "sourceSHA256": {
    "components/genet-webrtc/examples/interop_host.rs": "de4f114e285f3d0117f3906d57b46213fa49f597916f5039745673bb9447a79c",
    "support/ci/webrtc_interop.mjs": "589d126a60bfd4b264fbbe76634209f6cd18cc54d2043474fa1be8e0bc1e507e",
    "support/ci/webrtc_interop_peer.js": "35ab1f8c7f74c4ff550400617a1b86a4f61a0e2a93d63263b5930fb5c04462f7",
    "components/genet-webrtc/Cargo.toml": "a0c7b33c3e49b10ba972f67c1169c674046cb6c5e717caa9f6ed2d1c0e3a4265",
    "components/genet-webrtc/src/lib.rs": "1edb440b14b9c85d08051068ad4272345ba83a573e2aa9fbe8ba257210532fad",
    "components/script-runtime-api/webrtc/bootstrap.js": "b16a9c2cc771628ab6800492789278dec6fccc524c8308e2d03272f3b77c1bdd"
  },
  "versions": [
    {
      "browser": "chrome",
      "version": "Chrome/156.0.8078.74",
      "headed": true,
      "visibleWindows": 1,
      "webrtcPreferenceOverrides": false
    }
  ],
  "cases": [
    {
      "browser": "chrome",
      "engine": "boa",
      "route": "direct",
      "offerer": "native",
      "cleanup": "close",
      "status": "pass",
      "seconds": 15.85,
      "native": {
        "connected": true,
        "connectionState": "connected",
        "iceState": "connected",
        "signaling": "stable",
        "remoteSet": true,
        "received": [
          "browser:text:λ🙂:reliable",
          "binary:48000",
          "browser:text:λ🙂:unreliable"
        ],
        "sent": [
          "reliable:text",
          "reliable:binary",
          "unreliable:text"
        ],
        "modes": [
          {
            "label": "reliable",
            "ordered": true,
            "maxRetransmits": null
          },
          {
            "label": "unreliable",
            "ordered": false,
            "maxRetransmits": 0
          }
        ],
        "errors": [],
        "refused": [
          "mdns:OperationError",
          "mdns:OperationError"
        ],
        "remoteProtocols": [
          "udp",
          "udp",
          "udp"
        ],
        "channelStates": [
          "open",
          "open"
        ],
        "gathering": "complete",
        "stats": {
          "pair": {
            "state": "succeeded",
            "nominated": true,
            "local": "host",
            "remote": "prflx",
            "localProtocol": "udp",
            "remoteProtocol": "udp"
          },
          "localFingerprint": "3d53f83fee72decd1a4a35f5fe0082fc856225aca967428f428f8b32b137c95c",
          "remoteFingerprint": "e2fd08ea82581bd76544380cadddaf6ae73da0144fe54fd6049bca6425d64e33",
          "statsRemoteFingerprint": "e2fd08ea82581bd76544380cadddaf6ae73da0144fe54fd6049bca6425d64e33",
          "pairKeys": {
            "local": "RTCLocalIceCandidate_candidate:4FllNQUbruWZNRBXxrQvD7ZWCmgVzZ8f",
            "remote": "RTCRemoteIceCandidate_candidate:+SISIcfVhCvjuh90KmJkpsK/N5JKjz0G"
          },
          "candidateKeys": [
            {
              "id": "RTCLocalIceCandidate_candidate:Ep9zB/PkrLEkZMh9Xxj25vlu2dbDLg4K",
              "type": "local-candidate",
              "candidateType": "srflx",
              "protocol": "udp"
            },
            {
              "id": "RTCLocalIceCandidate_candidate:4FllNQUbruWZNRBXxrQvD7ZWCmgVzZ8f",
              "type": "local-candidate",
              "candidateType": "host",
              "protocol": "udp"
            },
            {
              "id": "RTCRemoteIceCandidate_candidate:FBWBp3XJjuFJKsGauQ4Jb/u5qqOlSV6l",
              "type": "remote-candidate",
              "candidateType": "srflx",
              "protocol": "udp"
            },
            {
              "id": "RTCRemoteIceCandidate_candidate:+SISIcfVhCvjuh90KmJkpsK/N5JKjz0G",
              "type": "remote-candidate",
              "candidateType": "prflx",
              "protocol": "udp"
            }
          ],
          "remoteDER_SHA256": [
            "e2fd08ea82581bd76544380cadddaf6ae73da0144fe54fd6049bca6425d64e33"
          ]
        }
      },
      "browserPeer": {
        "connected": true,
        "connectionState": "connected",
        "iceState": "connected",
        "signaling": "stable",
        "remoteSet": true,
        "received": [
          "native:text:λ🙂:reliable",
          "binary:48000",
          "native:text:λ🙂:unreliable"
        ],
        "sent": [
          "reliable:text",
          "reliable:binary",
          "unreliable:text"
        ],
        "modes": [
          {
            "label": "reliable",
            "ordered": true,
            "maxRetransmits": null
          },
          {
            "label": "unreliable",
            "ordered": false,
            "maxRetransmits": 0
          }
        ],
        "errors": [],
        "refused": [],
        "remoteProtocols": [
          "udp",
          "udp"
        ],
        "channelStates": [
          "open",
          "open"
        ],
        "gathering": "complete",
        "stats": {
          "pair": {
            "state": "succeeded",
            "nominated": true,
            "local": "srflx",
            "remote": "host",
            "localProtocol": "udp",
            "remoteProtocol": "udp"
          },
          "localFingerprint": "e2fd08ea82581bd76544380cadddaf6ae73da0144fe54fd6049bca6425d64e33",
          "remoteFingerprint": "3d53f83fee72decd1a4a35f5fe0082fc856225aca967428f428f8b32b137c95c",
          "statsRemoteFingerprint": "3d53f83fee72decd1a4a35f5fe0082fc856225aca967428f428f8b32b137c95c",
          "pairKeys": {
            "local": "Iq6H8bddQ",
            "remote": "Iy1YnZjfh"
          },
          "candidateKeys": [
            {
              "id": "IAagdkYUl",
              "type": "local-candidate",
              "candidateType": "host",
              "protocol": "udp"
            },
            {
              "id": "Iq6H8bddQ",
              "type": "local-candidate",
              "candidateType": "srflx",
              "protocol": "udp"
            },
            {
              "id": "IsMglqGUK",
              "type": "local-candidate",
              "candidateType": "host",
              "protocol": "udp"
            },
            {
              "id": "Iy1YnZjfh",
              "type": "remote-candidate",
              "candidateType": "host",
              "protocol": "udp"
            }
          ],
          "remoteDER_SHA256": [
            "3d53f83fee72decd1a4a35f5fe0082fc856225aca967428f428f8b32b137c95c"
          ]
        }
      },
      "cleanupVerified": true,
      "socketRebound": true
    }
  ],
  "failure": null,
  "profileCache": "C:\\t\\cargo-targets\\genet\\rtc-interop",
  "profileCleanup": "automatic approval review blocked recursive deletion; marked profiles retained for reuse",
  "scope": "native Windows to installed headed browsers, controlled WSL UDP server; no DNS/TCP/TLS/restart/media/carrier acceptance"
}
```


## E2h automated browser interoperability - 2026-11-02T19:18:24.600Z

```json
{
  "schema": 1,
  "gate": "E2h",
  "utc": "2026-11-02T19:18:24.600Z",
  "source": "42f9bc7df50e38669479fd37adc450e526959383",
  "node": "v24.11.0",
  "nativeBinarySHA256": "9f07b0fcda46985c363b05c47dc8c3a377d9f93ef4a01249b2604fb6189cedec",
  "sourceSHA256": {
    "components/genet-webrtc/examples/interop_host.rs": "de4f114e285f3d0117f3906d57b46213fa49f597916f5039745673bb9447a79c",
    "support/ci/webrtc_interop.mjs": "e98d52fae689f1998ea7ccbc66506529223a3fc30f4cb717a114d08658424320",
    "support/ci/webrtc_interop_peer.js": "35ab1f8c7f74c4ff550400617a1b86a4f61a0e2a93d63263b5930fb5c04462f7",
    "components/genet-webrtc/Cargo.toml": "a0c7b33c3e49b10ba972f67c1169c674046cb6c5e717caa9f6ed2d1c0e3a4265",
    "components/genet-webrtc/src/lib.rs": "0bae61d087b330f70fef6e7d6a000b554d1867ab02b968c54dcbca3017a23b19",
    "components/script-runtime-api/webrtc/bootstrap.js": "b16a9c2cc771628ab6800492789278dec6fccc524c8308e2d03272f3b77c1bdd"
  },
  "versions": [],
  "cases": [],
  "failure": {
    "phase": "browser startup",
    "case": null,
    "error": "browser has no OS-visible owned window"
  },
  "profileCache": "C:\\t\\cargo-targets\\genet\\rtc-interop",
  "profileCleanup": "automatic approval review blocked recursive deletion; marked profiles retained for reuse",
  "scope": "native Windows to installed headed browsers, controlled WSL UDP server; no DNS/TCP/TLS/restart/media/carrier acceptance"
}
```


## E2h automated browser interoperability - 2026-11-02T19:20:57.591Z

```json
{
  "schema": 1,
  "gate": "E2h",
  "utc": "2026-11-02T19:20:57.591Z",
  "source": "42f9bc7df50e38669479fd37adc450e526959383",
  "node": "v24.11.0",
  "nativeBinarySHA256": "9f07b0fcda46985c363b05c47dc8c3a377d9f93ef4a01249b2604fb6189cedec",
  "sourceSHA256": {
    "components/genet-webrtc/examples/interop_host.rs": "de4f114e285f3d0117f3906d57b46213fa49f597916f5039745673bb9447a79c",
    "support/ci/webrtc_interop.mjs": "e1899847249566107e259ba14593979d90ecca17a88312f7dc552b819db6e35d",
    "support/ci/webrtc_interop_peer.js": "35ab1f8c7f74c4ff550400617a1b86a4f61a0e2a93d63263b5930fb5c04462f7",
    "components/genet-webrtc/Cargo.toml": "a0c7b33c3e49b10ba972f67c1169c674046cb6c5e717caa9f6ed2d1c0e3a4265",
    "components/genet-webrtc/src/lib.rs": "0bae61d087b330f70fef6e7d6a000b554d1867ab02b968c54dcbca3017a23b19",
    "components/script-runtime-api/webrtc/bootstrap.js": "b16a9c2cc771628ab6800492789278dec6fccc524c8308e2d03272f3b77c1bdd"
  },
  "versions": [
    {
      "browser": "firefox",
      "version": "159.0",
      "headed": true,
      "visibleWindows": 1,
      "webrtcPreferenceOverrides": false
    }
  ],
  "cases": [
    {
      "browser": "firefox",
      "engine": "boa",
      "route": "direct",
      "offerer": "native",
      "cleanup": "navigate",
      "status": "fail",
      "phase": "negotiation and transfer",
      "error": "peer error gathering:701",
      "observed": {
        "native": {
          "connected": false,
          "connectionState": "connecting",
          "iceState": "checking",
          "signaling": "stable",
          "remoteSet": true,
          "received": [],
          "sent": [],
          "modes": [],
          "errors": [],
          "refused": [
            "mdns:OperationError",
            "mdns:OperationError",
            "mdns:OperationError",
            "mdns:OperationError"
          ],
          "remoteProtocols": [
            "UDP",
            "UDP",
            "TCP",
            "TCP",
            null
          ],
          "channelStates": [
            "connecting",
            "connecting"
          ],
          "gathering": "complete"
        },
        "browser": {
          "connected": false,
          "connectionState": "connecting",
          "iceState": "checking",
          "signaling": "stable",
          "remoteSet": true,
          "received": [],
          "sent": [],
          "modes": [],
          "errors": [
            "gathering:701"
          ],
          "refused": [],
          "remoteProtocols": [
            "udp",
            "udp"
          ],
          "channelStates": [],
          "gathering": "complete"
        }
      },
      "observedStats": {
        "native": {
          "pair": null,
          "localFingerprint": "e812ed9561b4ed0ef4c30a21c2cd6e6a5819845c7783172c18ddfabaafd29a6a",
          "remoteFingerprint": "0535e6f72ebc795d93c39b6cc62a38d7eabec29f478ae4bfa93bd677a9be4f89",
          "statsRemoteFingerprint": null,
          "pairKeys": null,
          "candidateKeys": [
            {
              "id": "RTCLocalIceCandidate_candidate:pfXDNOqD53YXkkCySUrofaSBO3ot1oHz",
              "type": "local-candidate",
              "candidateType": "srflx",
              "protocol": "udp"
            },
            {
              "id": "RTCLocalIceCandidate_candidate:IQi1wabCfUqTrLOz6xEuZcmEBSvT/uC7",
              "type": "local-candidate",
              "candidateType": "host",
              "protocol": "udp"
            }
          ],
          "remoteDER_SHA256": []
        },
        "browser": {
          "pair": null,
          "localFingerprint": "0535e6f72ebc795d93c39b6cc62a38d7eabec29f478ae4bfa93bd677a9be4f89",
          "remoteFingerprint": "e812ed9561b4ed0ef4c30a21c2cd6e6a5819845c7783172c18ddfabaafd29a6a",
          "statsRemoteFingerprint": null,
          "pairKeys": null,
          "candidateKeys": [
            {
              "id": "bcb41154",
              "type": "local-candidate",
              "candidateType": "host",
              "protocol": "udp"
            },
            {
              "id": "afa176a1",
              "type": "local-candidate",
              "candidateType": "host",
              "protocol": "udp"
            },
            {
              "id": "1ee9ea8b",
              "type": "local-candidate",
              "candidateType": "host",
              "protocol": "tcp"
            },
            {
              "id": "b5aa69ac",
              "type": "local-candidate",
              "candidateType": "host",
              "protocol": "tcp"
            },
            {
              "id": "d6913d4c",
              "type": "remote-candidate",
              "candidateType": "host",
              "protocol": "udp"
            },
            {
              "id": "24c9dda6",
              "type": "remote-candidate",
              "candidateType": "srflx",
              "protocol": "udp"
            }
          ],
          "remoteDER_SHA256": []
        }
      },
      "trace": []
    }
  ],
  "failure": {
    "phase": "negotiation and transfer",
    "case": {
      "browser": "firefox",
      "engine": "boa",
      "route": "direct",
      "offerer": "native",
      "cleanup": "navigate"
    },
    "error": "peer error gathering:701"
  },
  "profileCache": "C:\\t\\cargo-targets\\genet\\rtc-interop",
  "profileCleanup": "automatic approval review blocked recursive deletion; marked profiles retained for reuse",
  "scope": "native Windows to installed headed browsers, controlled WSL UDP server; no DNS/TCP/TLS/restart/media/carrier acceptance"
}
```


## E2h automated browser interoperability - 2026-11-02T19:23:31.572Z

```json
{
  "schema": 1,
  "gate": "E2h",
  "utc": "2026-11-02T19:23:31.572Z",
  "source": "42f9bc7df50e38669479fd37adc450e526959383",
  "node": "v24.11.0",
  "nativeBinarySHA256": "9f07b0fcda46985c363b05c47dc8c3a377d9f93ef4a01249b2604fb6189cedec",
  "sourceSHA256": {
    "components/genet-webrtc/examples/interop_host.rs": "de4f114e285f3d0117f3906d57b46213fa49f597916f5039745673bb9447a79c",
    "support/ci/webrtc_interop.mjs": "ed91bda3ab568b2628cd1a0e34eb95dff4afab9cd43947d470b289d965fd6dfe",
    "support/ci/webrtc_interop_peer.js": "e4defa0fb7bc53407f64a3151c39aef45c3ab82ef959a39a575b02f093edfa28",
    "components/genet-webrtc/Cargo.toml": "a0c7b33c3e49b10ba972f67c1169c674046cb6c5e717caa9f6ed2d1c0e3a4265",
    "components/genet-webrtc/src/lib.rs": "0bae61d087b330f70fef6e7d6a000b554d1867ab02b968c54dcbca3017a23b19",
    "components/script-runtime-api/webrtc/bootstrap.js": "b16a9c2cc771628ab6800492789278dec6fccc524c8308e2d03272f3b77c1bdd"
  },
  "versions": [
    {
      "browser": "firefox",
      "version": "159.0",
      "headed": true,
      "visibleWindows": 1,
      "webrtcPreferenceOverrides": false
    }
  ],
  "cases": [
    {
      "browser": "firefox",
      "engine": "boa",
      "route": "direct",
      "offerer": "native",
      "cleanup": "navigate",
      "status": "fail",
      "phase": "negotiation and transfer",
      "error": "application exchange deadline",
      "observed": {
        "native": {
          "connected": false,
          "connectionState": "connecting",
          "iceState": "checking",
          "signaling": "stable",
          "remoteSet": true,
          "received": [],
          "sent": [],
          "modes": [],
          "errors": [],
          "refused": [
            "mdns:OperationError",
            "mdns:OperationError",
            "mdns:OperationError",
            "mdns:OperationError"
          ],
          "remoteProtocols": [
            "UDP",
            "UDP",
            "TCP",
            "TCP",
            null
          ],
          "gatheringErrors": [],
          "channelStates": [
            "connecting",
            "connecting"
          ],
          "gathering": "complete"
        },
        "browser": {
          "connected": false,
          "connectionState": "failed",
          "iceState": "failed",
          "signaling": "stable",
          "remoteSet": true,
          "received": [],
          "sent": [],
          "modes": [],
          "errors": [],
          "refused": [],
          "remoteProtocols": [
            "udp",
            "udp"
          ],
          "gatheringErrors": [
            {
              "code": 701,
              "text": ""
            }
          ],
          "channelStates": [],
          "gathering": "complete"
        }
      },
      "observedStats": {
        "native": {
          "pair": null,
          "localFingerprint": "c4d74817909fe08e4a464b63e5b30d578bf1fbcef313eeed00cc6b7ee4ccc874",
          "remoteFingerprint": "032e893d665ca47469e0232940187d274ce24ff83d026dd5275d980433e35172",
          "statsRemoteFingerprint": null,
          "pairKeys": null,
          "candidateKeys": [
            {
              "id": "RTCLocalIceCandidate_candidate:TnAFZVg3ts0AwrTA7xIAXTCcgjCz4Rrs",
              "type": "local-candidate",
              "candidateType": "host",
              "protocol": "udp"
            },
            {
              "id": "RTCLocalIceCandidate_candidate:be6GVZdkJSFpvLp0PeQEVDrOo/xVecdh",
              "type": "local-candidate",
              "candidateType": "srflx",
              "protocol": "udp"
            }
          ],
          "remoteDER_SHA256": []
        },
        "browser": {
          "pair": null,
          "localFingerprint": "032e893d665ca47469e0232940187d274ce24ff83d026dd5275d980433e35172",
          "remoteFingerprint": "c4d74817909fe08e4a464b63e5b30d578bf1fbcef313eeed00cc6b7ee4ccc874",
          "statsRemoteFingerprint": null,
          "pairKeys": null,
          "candidateKeys": [
            {
              "id": "ce01e9aa",
              "type": "local-candidate",
              "candidateType": "host",
              "protocol": "udp"
            },
            {
              "id": "96d44e66",
              "type": "local-candidate",
              "candidateType": "host",
              "protocol": "udp"
            },
            {
              "id": "c8c49301",
              "type": "local-candidate",
              "candidateType": "host",
              "protocol": "tcp"
            },
            {
              "id": "1db8b35b",
              "type": "local-candidate",
              "candidateType": "host",
              "protocol": "tcp"
            },
            {
              "id": "34dcc90f",
              "type": "remote-candidate",
              "candidateType": "host",
              "protocol": "udp"
            },
            {
              "id": "41a6a236",
              "type": "remote-candidate",
              "candidateType": "srflx",
              "protocol": "udp"
            }
          ],
          "remoteDER_SHA256": []
        }
      },
      "trace": []
    }
  ],
  "failure": {
    "phase": "negotiation and transfer",
    "case": {
      "browser": "firefox",
      "engine": "boa",
      "route": "direct",
      "offerer": "native",
      "cleanup": "navigate"
    },
    "error": "application exchange deadline"
  },
  "profileCache": "C:\\t\\cargo-targets\\genet\\rtc-interop",
  "profileCleanup": "automatic approval review blocked recursive deletion; marked profiles retained for reuse",
  "scope": "native Windows to installed headed browsers, controlled WSL UDP server; no DNS/TCP/TLS/restart/media/carrier acceptance"
}
```


## E2h automated browser interoperability - 2026-11-02T19:26:53.240Z

```json
{
  "schema": 1,
  "gate": "E2h",
  "utc": "2026-11-02T19:26:53.240Z",
  "source": "42f9bc7df50e38669479fd37adc450e526959383",
  "node": "v24.11.0",
  "nativeBinarySHA256": "9f07b0fcda46985c363b05c47dc8c3a377d9f93ef4a01249b2604fb6189cedec",
  "sourceSHA256": {
    "components/genet-webrtc/examples/interop_host.rs": "de4f114e285f3d0117f3906d57b46213fa49f597916f5039745673bb9447a79c",
    "support/ci/webrtc_interop.mjs": "ed91bda3ab568b2628cd1a0e34eb95dff4afab9cd43947d470b289d965fd6dfe",
    "support/ci/webrtc_interop_peer.js": "e4defa0fb7bc53407f64a3151c39aef45c3ab82ef959a39a575b02f093edfa28",
    "components/genet-webrtc/Cargo.toml": "a0c7b33c3e49b10ba972f67c1169c674046cb6c5e717caa9f6ed2d1c0e3a4265",
    "components/genet-webrtc/src/lib.rs": "0bae61d087b330f70fef6e7d6a000b554d1867ab02b968c54dcbca3017a23b19",
    "components/script-runtime-api/webrtc/bootstrap.js": "b16a9c2cc771628ab6800492789278dec6fccc524c8308e2d03272f3b77c1bdd"
  },
  "versions": [
    {
      "browser": "firefox",
      "version": "159.0",
      "headed": true,
      "visibleWindows": 1,
      "webrtcPreferenceOverrides": false
    }
  ],
  "cases": [
    {
      "browser": "firefox",
      "engine": "boa",
      "route": "relay",
      "offerer": "browser",
      "cleanup": "close",
      "status": "fail",
      "phase": "negotiation and transfer",
      "error": "incomplete application exchange",
      "observed": {
        "native": {
          "connected": false,
          "connectionState": "connecting",
          "iceState": "checking",
          "signaling": "stable",
          "remoteSet": true,
          "received": [],
          "sent": [],
          "modes": [],
          "errors": [],
          "refused": [],
          "remoteProtocols": [
            null
          ],
          "gatheringErrors": [],
          "channelStates": [],
          "gathering": "complete"
        },
        "browser": {
          "connected": false,
          "connectionState": "failed",
          "iceState": "failed",
          "signaling": "stable",
          "remoteSet": true,
          "received": [],
          "sent": [],
          "modes": [],
          "errors": [],
          "refused": [],
          "remoteProtocols": [
            "udp"
          ],
          "gatheringErrors": [
            {
              "code": 701,
              "text": ""
            }
          ],
          "channelStates": [
            "closed",
            "closed"
          ],
          "gathering": "complete"
        }
      },
      "observedStats": {
        "native": {
          "pair": null,
          "localFingerprint": "1bf07b5b62491e37de820b1311848a7b6b87ac0cca7ecdc9017be31eb5464005",
          "remoteFingerprint": "d291617d4e8f50a2be33e2fc8393b09dfa04df3b62e172a5fbf5d612b8c8ec73",
          "statsRemoteFingerprint": null,
          "pairKeys": null,
          "candidateKeys": [
            {
              "id": "RTCLocalIceCandidate_candidate:8uGuRWlIjjHzjgrybWVIkqQ6CYmXlIwH",
              "type": "local-candidate",
              "candidateType": "relay",
              "protocol": "udp"
            }
          ],
          "remoteDER_SHA256": []
        },
        "browser": {
          "pair": null,
          "localFingerprint": "d291617d4e8f50a2be33e2fc8393b09dfa04df3b62e172a5fbf5d612b8c8ec73",
          "remoteFingerprint": "1bf07b5b62491e37de820b1311848a7b6b87ac0cca7ecdc9017be31eb5464005",
          "statsRemoteFingerprint": null,
          "pairKeys": null,
          "candidateKeys": [
            {
              "id": "45ccc26d",
              "type": "remote-candidate",
              "candidateType": "relay",
              "protocol": "udp"
            }
          ],
          "remoteDER_SHA256": []
        }
      },
      "trace": []
    }
  ],
  "failure": {
    "phase": "negotiation and transfer",
    "case": {
      "browser": "firefox",
      "engine": "boa",
      "route": "relay",
      "offerer": "browser",
      "cleanup": "close"
    },
    "error": "incomplete application exchange"
  },
  "profileCache": "C:\\t\\cargo-targets\\genet\\rtc-interop",
  "profileCleanup": "automatic approval review blocked recursive deletion; marked profiles retained for reuse",
  "scope": "native Windows to installed headed browsers, controlled WSL UDP server; no DNS/TCP/TLS/restart/media/carrier acceptance"
}
```


## E2h automated browser interoperability - 2026-11-02T19:30:14.870Z

```json
{
  "schema": 1,
  "gate": "E2h",
  "utc": "2026-11-02T19:30:14.870Z",
  "source": "42f9bc7df50e38669479fd37adc450e526959383",
  "node": "v24.11.0",
  "nativeBinarySHA256": "9f07b0fcda46985c363b05c47dc8c3a377d9f93ef4a01249b2604fb6189cedec",
  "sourceSHA256": {
    "components/genet-webrtc/examples/interop_host.rs": "de4f114e285f3d0117f3906d57b46213fa49f597916f5039745673bb9447a79c",
    "support/ci/webrtc_interop.mjs": "2db29663c29cda911afb7d6bd6e2d8c62c52d5556523b38ccd47a0f168bc237e",
    "support/ci/webrtc_interop_peer.js": "e4defa0fb7bc53407f64a3151c39aef45c3ab82ef959a39a575b02f093edfa28",
    "components/genet-webrtc/Cargo.toml": "a0c7b33c3e49b10ba972f67c1169c674046cb6c5e717caa9f6ed2d1c0e3a4265",
    "components/genet-webrtc/src/lib.rs": "0bae61d087b330f70fef6e7d6a000b554d1867ab02b968c54dcbca3017a23b19",
    "components/script-runtime-api/webrtc/bootstrap.js": "b16a9c2cc771628ab6800492789278dec6fccc524c8308e2d03272f3b77c1bdd"
  },
  "versions": [
    {
      "browser": "firefox",
      "version": "159.0",
      "headed": true,
      "visibleWindows": 1,
      "webrtcPreferenceOverrides": false,
      "networkPermissions": [
        {
          "permission": "local-network",
          "before": "prompt",
          "after": "granted",
          "error": null
        }
      ]
    }
  ],
  "cases": [
    {
      "browser": "firefox",
      "engine": "boa",
      "route": "relay",
      "offerer": "browser",
      "cleanup": "close",
      "status": "fail",
      "phase": "negotiation and transfer",
      "error": "application exchange deadline",
      "observed": {
        "native": {
          "connected": false,
          "connectionState": "connecting",
          "iceState": "checking",
          "signaling": "stable",
          "remoteSet": true,
          "received": [],
          "sent": [],
          "modes": [],
          "errors": [],
          "refused": [],
          "remoteProtocols": [],
          "gatheringErrors": [],
          "channelStates": [],
          "gathering": "complete"
        },
        "browser": {
          "connected": false,
          "connectionState": "failed",
          "iceState": "failed",
          "signaling": "stable",
          "remoteSet": true,
          "received": [],
          "sent": [],
          "modes": [],
          "errors": [],
          "refused": [],
          "remoteProtocols": [
            "udp"
          ],
          "gatheringErrors": [],
          "channelStates": [
            "closed",
            "closed"
          ],
          "gathering": "gathering"
        }
      },
      "observedStats": {
        "native": {
          "pair": null,
          "localFingerprint": "704ee44aa9f9f3431e0c5dceb4eadd81dc985265d549d69165b9aa1d142791b8",
          "remoteFingerprint": "21511c7afe115eb7f67cf5ef548610b9ead32716543e84ce252121fb3a1f0d76",
          "statsRemoteFingerprint": null,
          "pairKeys": null,
          "candidateKeys": [
            {
              "id": "RTCLocalIceCandidate_candidate:kJACZQHGzsFJb9boS88ie3hRcgg+cqQI",
              "type": "local-candidate",
              "candidateType": "relay",
              "protocol": "udp"
            }
          ],
          "remoteDER_SHA256": []
        },
        "browser": {
          "pair": null,
          "localFingerprint": "21511c7afe115eb7f67cf5ef548610b9ead32716543e84ce252121fb3a1f0d76",
          "remoteFingerprint": "704ee44aa9f9f3431e0c5dceb4eadd81dc985265d549d69165b9aa1d142791b8",
          "statsRemoteFingerprint": null,
          "pairKeys": null,
          "candidateKeys": [
            {
              "id": "e2c5d7ad",
              "type": "remote-candidate",
              "candidateType": "relay",
              "protocol": "udp"
            }
          ],
          "remoteDER_SHA256": []
        }
      },
      "trace": []
    }
  ],
  "failure": {
    "phase": "negotiation and transfer",
    "case": {
      "browser": "firefox",
      "engine": "boa",
      "route": "relay",
      "offerer": "browser",
      "cleanup": "close"
    },
    "error": "application exchange deadline"
  },
  "profileCache": "C:\\t\\cargo-targets\\genet\\rtc-interop",
  "profileCleanup": "automatic approval review blocked recursive deletion; marked profiles retained for reuse",
  "scope": "native Windows to installed headed browsers, controlled WSL UDP server; no DNS/TCP/TLS/restart/media/carrier acceptance"
}
```


## E2h automated browser interoperability - 2026-11-02T19:33:24.271Z

```json
{
  "schema": 1,
  "gate": "E2h",
  "utc": "2026-11-02T19:33:24.271Z",
  "source": "42f9bc7df50e38669479fd37adc450e526959383",
  "node": "v24.11.0",
  "nativeBinarySHA256": "9f07b0fcda46985c363b05c47dc8c3a377d9f93ef4a01249b2604fb6189cedec",
  "sourceSHA256": {
    "components/genet-webrtc/examples/interop_host.rs": "de4f114e285f3d0117f3906d57b46213fa49f597916f5039745673bb9447a79c",
    "support/ci/webrtc_interop.mjs": "e962787a5bff5ecf1933202cdb1f81ed8f9eba66fbae54a45702a7a59148c6db",
    "support/ci/webrtc_interop_peer.js": "e4defa0fb7bc53407f64a3151c39aef45c3ab82ef959a39a575b02f093edfa28",
    "components/genet-webrtc/Cargo.toml": "a0c7b33c3e49b10ba972f67c1169c674046cb6c5e717caa9f6ed2d1c0e3a4265",
    "components/genet-webrtc/src/lib.rs": "0bae61d087b330f70fef6e7d6a000b554d1867ab02b968c54dcbca3017a23b19",
    "components/script-runtime-api/webrtc/bootstrap.js": "b16a9c2cc771628ab6800492789278dec6fccc524c8308e2d03272f3b77c1bdd"
  },
  "versions": [],
  "cases": [],
  "failure": {
    "phase": "browser startup",
    "case": null,
    "error": "browser RPC no such frame"
  },
  "profileCache": "C:\\t\\cargo-targets\\genet\\rtc-interop",
  "profileCleanup": "automatic approval review blocked recursive deletion; marked profiles retained for reuse",
  "scope": "native Windows to installed headed browsers, controlled WSL UDP server; no DNS/TCP/TLS/restart/media/carrier acceptance"
}
```


## E2h automated browser interoperability - 2026-11-02T19:35:40.272Z

```json
{
  "schema": 1,
  "gate": "E2h",
  "utc": "2026-11-02T19:35:40.272Z",
  "source": "42f9bc7df50e38669479fd37adc450e526959383",
  "node": "v24.11.0",
  "nativeBinarySHA256": "9f07b0fcda46985c363b05c47dc8c3a377d9f93ef4a01249b2604fb6189cedec",
  "sourceSHA256": {
    "components/genet-webrtc/examples/interop_host.rs": "de4f114e285f3d0117f3906d57b46213fa49f597916f5039745673bb9447a79c",
    "support/ci/webrtc_interop.mjs": "3ee7c8889822b5bc84f81d3032248000a06ba16d624819bd9b88a62f8a6ad449",
    "support/ci/webrtc_interop_peer.js": "e4defa0fb7bc53407f64a3151c39aef45c3ab82ef959a39a575b02f093edfa28",
    "components/genet-webrtc/Cargo.toml": "a0c7b33c3e49b10ba972f67c1169c674046cb6c5e717caa9f6ed2d1c0e3a4265",
    "components/genet-webrtc/src/lib.rs": "0bae61d087b330f70fef6e7d6a000b554d1867ab02b968c54dcbca3017a23b19",
    "components/script-runtime-api/webrtc/bootstrap.js": "b16a9c2cc771628ab6800492789278dec6fccc524c8308e2d03272f3b77c1bdd"
  },
  "versions": [
    {
      "browser": "firefox",
      "version": "159.0",
      "headed": true,
      "visibleWindows": 1,
      "webrtcPreferenceOverrides": false,
      "networkPermissions": [
        {
          "permission": "local-network",
          "before": "prompt",
          "after": "granted",
          "error": null,
          "reset": "prompt"
        },
        {
          "permission": "loopback-network",
          "before": "prompt",
          "after": "granted",
          "error": null,
          "reset": "prompt"
        }
      ]
    }
  ],
  "cases": [
    {
      "browser": "firefox",
      "engine": "boa",
      "route": "relay",
      "offerer": "browser",
      "cleanup": "close",
      "status": "fail",
      "phase": "negotiation and transfer",
      "error": "application exchange deadline",
      "observed": {
        "native": {
          "connected": false,
          "connectionState": "connecting",
          "iceState": "checking",
          "signaling": "stable",
          "remoteSet": true,
          "received": [],
          "sent": [],
          "modes": [],
          "errors": [],
          "refused": [],
          "remoteProtocols": [
            null
          ],
          "gatheringErrors": [],
          "channelStates": [],
          "gathering": "complete"
        },
        "browser": {
          "connected": false,
          "connectionState": "failed",
          "iceState": "failed",
          "signaling": "stable",
          "remoteSet": true,
          "received": [],
          "sent": [],
          "modes": [],
          "errors": [],
          "refused": [],
          "remoteProtocols": [
            "udp"
          ],
          "gatheringErrors": [
            {
              "code": 701,
              "text": ""
            }
          ],
          "channelStates": [
            "closed",
            "closed"
          ],
          "gathering": "complete"
        }
      },
      "observedStats": {
        "native": {
          "pair": null,
          "localFingerprint": "1216fa38e07cb027d3478a4c0db7f3d9ad76eebec6c43092ccecb26696e24e73",
          "remoteFingerprint": "f54960428e7130b7f8cebdd6ee77f2867d804c92a7f3079eee8c2656f724cd1b",
          "statsRemoteFingerprint": null,
          "pairKeys": null,
          "candidateKeys": [
            {
              "id": "RTCLocalIceCandidate_candidate:N+tPa1SGKIZYyfAXAlhyR1BTkgZcZ1Ej",
              "type": "local-candidate",
              "candidateType": "relay",
              "protocol": "udp"
            }
          ],
          "remoteDER_SHA256": []
        },
        "browser": {
          "pair": null,
          "localFingerprint": "f54960428e7130b7f8cebdd6ee77f2867d804c92a7f3079eee8c2656f724cd1b",
          "remoteFingerprint": "1216fa38e07cb027d3478a4c0db7f3d9ad76eebec6c43092ccecb26696e24e73",
          "statsRemoteFingerprint": null,
          "pairKeys": null,
          "candidateKeys": [
            {
              "id": "3c70c875",
              "type": "remote-candidate",
              "candidateType": "relay",
              "protocol": "udp"
            }
          ],
          "remoteDER_SHA256": []
        }
      },
      "trace": []
    }
  ],
  "failure": {
    "phase": "negotiation and transfer",
    "case": {
      "browser": "firefox",
      "engine": "boa",
      "route": "relay",
      "offerer": "browser",
      "cleanup": "close"
    },
    "error": "application exchange deadline"
  },
  "profileCache": "C:\\t\\cargo-targets\\genet\\rtc-interop",
  "profileCleanup": "automatic approval review blocked recursive deletion; marked profiles retained for reuse",
  "scope": "native Windows to installed headed browsers, controlled WSL UDP server; no DNS/TCP/TLS/restart/media/carrier acceptance"
}
```


## E2h automated browser interoperability - 2026-11-02T19:40:06.875Z

```json
{
  "schema": 1,
  "gate": "E2h",
  "utc": "2026-11-02T19:40:06.875Z",
  "source": "42f9bc7df50e38669479fd37adc450e526959383",
  "node": "v24.11.0",
  "nativeBinarySHA256": "9f07b0fcda46985c363b05c47dc8c3a377d9f93ef4a01249b2604fb6189cedec",
  "sourceSHA256": {
    "components/genet-webrtc/examples/interop_host.rs": "de4f114e285f3d0117f3906d57b46213fa49f597916f5039745673bb9447a79c",
    "support/ci/webrtc_interop.mjs": "c6bb644c5a707c2151a1c2a234af2efff9048bf2ca483d878a252a0fc296dda6",
    "support/ci/webrtc_interop_peer.js": "e4defa0fb7bc53407f64a3151c39aef45c3ab82ef959a39a575b02f093edfa28",
    "components/genet-webrtc/Cargo.toml": "a0c7b33c3e49b10ba972f67c1169c674046cb6c5e717caa9f6ed2d1c0e3a4265",
    "components/genet-webrtc/src/lib.rs": "0bae61d087b330f70fef6e7d6a000b554d1867ab02b968c54dcbca3017a23b19",
    "components/script-runtime-api/webrtc/bootstrap.js": "b16a9c2cc771628ab6800492789278dec6fccc524c8308e2d03272f3b77c1bdd"
  },
  "versions": [
    {
      "browser": "firefox",
      "version": "159.0",
      "headed": true,
      "visibleWindows": 1,
      "webrtcPreferenceOverrides": false,
      "networkPermissions": [
        {
          "permission": "local-network",
          "before": "prompt",
          "after": "granted",
          "error": null,
          "reset": "prompt"
        },
        {
          "permission": "loopback-network",
          "before": "prompt",
          "after": "granted",
          "error": null,
          "reset": "prompt"
        }
      ]
    }
  ],
  "cases": [
    {
      "browser": "firefox",
      "engine": "boa",
      "route": "relay",
      "offerer": "browser",
      "cleanup": "close",
      "status": "pass",
      "seconds": 17.27,
      "native": {
        "connected": true,
        "textMessagesVerified": 2,
        "binaryBytesVerified": 48000,
        "modes": [
          {
            "label": "reliable",
            "ordered": true,
            "maxRetransmits": null
          },
          {
            "label": "unreliable",
            "ordered": false,
            "maxRetransmits": 0
          }
        ],
        "refused": [],
        "gatheringErrors": [],
        "pair": {
          "state": "succeeded",
          "nominated": true,
          "local": "relay",
          "remote": "relay",
          "localProtocol": "udp",
          "remoteProtocol": "udp"
        },
        "localFingerprint": "ee96e2f6b944e8c7571b8831d4411737e3a273b58f8f54bd5060152e9d7fd442",
        "remoteFingerprint": "401dfaba14bf7c2736aca958b00f1ea9f1452e3f82b3cb6d0b5f8bab1b42a910",
        "remoteDER_SHA256": [
          "401dfaba14bf7c2736aca958b00f1ea9f1452e3f82b3cb6d0b5f8bab1b42a910"
        ],
        "statsRemoteFingerprint": "401dfaba14bf7c2736aca958b00f1ea9f1452e3f82b3cb6d0b5f8bab1b42a910"
      },
      "browserPeer": {
        "connected": true,
        "textMessagesVerified": 2,
        "binaryBytesVerified": 48000,
        "modes": [
          {
            "label": "reliable",
            "ordered": true,
            "maxRetransmits": null
          },
          {
            "label": "unreliable",
            "ordered": false,
            "maxRetransmits": 0
          }
        ],
        "refused": [],
        "gatheringErrors": [],
        "pair": {
          "state": "succeeded",
          "nominated": true,
          "local": "relay",
          "remote": "relay",
          "localProtocol": "udp",
          "remoteProtocol": "udp"
        },
        "localFingerprint": "401dfaba14bf7c2736aca958b00f1ea9f1452e3f82b3cb6d0b5f8bab1b42a910",
        "remoteFingerprint": "ee96e2f6b944e8c7571b8831d4411737e3a273b58f8f54bd5060152e9d7fd442",
        "remoteDER_SHA256": [
          "ee96e2f6b944e8c7571b8831d4411737e3a273b58f8f54bd5060152e9d7fd442"
        ],
        "statsRemoteFingerprint": "ee96e2f6b944e8c7571b8831d4411737e3a273b58f8f54bd5060152e9d7fd442"
      },
      "cleanupVerified": true,
      "socketRebound": true
    }
  ],
  "failure": null,
  "relayFixture": "bounded Windows UDP front end forwarding unchanged datagrams to WSL coturn; distinct backend socket per client; max 8 clients/256 pending datagrams/1 MiB",
  "profileCache": "C:\\t\\cargo-targets\\genet\\rtc-interop",
  "profileCleanup": "automatic approval review blocked recursive deletion; marked profiles retained for reuse",
  "scope": "native Windows to installed headed browsers, controlled WSL UDP server; no DNS/TCP/TLS/restart/media/carrier acceptance"
}
```


## E2h automated browser interoperability - 2026-11-02T19:43:46.697Z

```json
{
  "schema": 1,
  "gate": "E2h",
  "utc": "2026-11-02T19:43:46.697Z",
  "source": "42f9bc7df50e38669479fd37adc450e526959383",
  "node": "v24.11.0",
  "nativeBinarySHA256": "9f07b0fcda46985c363b05c47dc8c3a377d9f93ef4a01249b2604fb6189cedec",
  "sourceSHA256": {
    "components/genet-webrtc/examples/interop_host.rs": "de4f114e285f3d0117f3906d57b46213fa49f597916f5039745673bb9447a79c",
    "support/ci/webrtc_interop.mjs": "c6bb644c5a707c2151a1c2a234af2efff9048bf2ca483d878a252a0fc296dda6",
    "support/ci/webrtc_interop_peer.js": "e4defa0fb7bc53407f64a3151c39aef45c3ab82ef959a39a575b02f093edfa28",
    "components/genet-webrtc/Cargo.toml": "a0c7b33c3e49b10ba972f67c1169c674046cb6c5e717caa9f6ed2d1c0e3a4265",
    "components/genet-webrtc/src/lib.rs": "0bae61d087b330f70fef6e7d6a000b554d1867ab02b968c54dcbca3017a23b19",
    "components/script-runtime-api/webrtc/bootstrap.js": "b16a9c2cc771628ab6800492789278dec6fccc524c8308e2d03272f3b77c1bdd"
  },
  "versions": [
    {
      "browser": "firefox",
      "version": "159.0",
      "headed": true,
      "visibleWindows": 1,
      "webrtcPreferenceOverrides": false,
      "networkPermissions": [
        {
          "permission": "local-network",
          "before": "prompt",
          "after": "granted",
          "error": null,
          "reset": "prompt"
        },
        {
          "permission": "loopback-network",
          "before": "prompt",
          "after": "granted",
          "error": null,
          "reset": "prompt"
        }
      ]
    }
  ],
  "cases": [
    {
      "browser": "firefox",
      "engine": "boa",
      "route": "direct",
      "offerer": "native",
      "cleanup": "navigate",
      "status": "pass",
      "seconds": 17.3,
      "native": {
        "connected": true,
        "textMessagesVerified": 2,
        "binaryBytesVerified": 48000,
        "modes": [
          {
            "label": "unreliable",
            "ordered": false,
            "maxRetransmits": 0
          },
          {
            "label": "reliable",
            "ordered": true,
            "maxRetransmits": null
          }
        ],
        "refused": [
          "mdns:OperationError",
          "mdns:OperationError",
          "mdns:OperationError",
          "mdns:OperationError"
        ],
        "gatheringErrors": [],
        "pair": {
          "state": "succeeded",
          "nominated": true,
          "local": "host",
          "remote": "prflx",
          "localProtocol": "udp",
          "remoteProtocol": "udp"
        },
        "localFingerprint": "404a7a713726976a88f45444d237d595051f6aa8cfffef57c3f5d07d7f61f169",
        "remoteFingerprint": "b24118bc32b51229b3c40b7c4df62834ad90c0993df77e28f555c4ef4672073b",
        "remoteDER_SHA256": [
          "b24118bc32b51229b3c40b7c4df62834ad90c0993df77e28f555c4ef4672073b"
        ],
        "statsRemoteFingerprint": "b24118bc32b51229b3c40b7c4df62834ad90c0993df77e28f555c4ef4672073b"
      },
      "browserPeer": {
        "connected": true,
        "textMessagesVerified": 2,
        "binaryBytesVerified": 48000,
        "modes": [
          {
            "label": "unreliable",
            "ordered": false,
            "maxRetransmits": 0
          },
          {
            "label": "reliable",
            "ordered": true,
            "maxRetransmits": null
          }
        ],
        "refused": [],
        "gatheringErrors": [],
        "pair": {
          "state": "succeeded",
          "nominated": true,
          "local": "host",
          "remote": "host",
          "localProtocol": "udp",
          "remoteProtocol": "udp"
        },
        "localFingerprint": "b24118bc32b51229b3c40b7c4df62834ad90c0993df77e28f555c4ef4672073b",
        "remoteFingerprint": "404a7a713726976a88f45444d237d595051f6aa8cfffef57c3f5d07d7f61f169",
        "remoteDER_SHA256": [
          "404a7a713726976a88f45444d237d595051f6aa8cfffef57c3f5d07d7f61f169"
        ],
        "statsRemoteFingerprint": "404a7a713726976a88f45444d237d595051f6aa8cfffef57c3f5d07d7f61f169"
      },
      "cleanupVerified": true,
      "socketRebound": true
    }
  ],
  "failure": null,
  "relayFixture": "bounded Windows UDP front end forwarding unchanged datagrams to WSL coturn; distinct backend socket per client; max 8 clients/256 pending datagrams/1 MiB",
  "profileCache": "C:\\t\\cargo-targets\\genet\\rtc-interop",
  "profileCleanup": "automatic approval review blocked recursive deletion; marked profiles retained for reuse",
  "scope": "native Windows to installed headed browsers, controlled WSL UDP server; no DNS/TCP/TLS/restart/media/carrier acceptance"
}
```


## E2h final native fixture build

```json
{"source":"d644fa0a013364a3d3437b30e98ef3da08dccbb2","utc":"2026-11-02T19:56:32.506032+00:00","command":["cargo","build","-p","genet-webrtc","--example","interop_host","--locked","--offline","-j","1"],"exit":0,"seconds":17.94,"lockSHA256":"0a42b50cb56df741ad34acc18be60070a2eb3244a3a50d64260e7e67e9754f9f","stderr_tail":["   |     ^^^^^^^^^^^^^^^^^^^^^^","","warning: `genet-runtime` (lib) generated 4 warnings (run `cargo fix --lib -p genet-runtime` to apply 1 suggestion)","   Compiling genet-webrtc v0.2.0 (C:\\Users\\mark_\\Code\\repos\\genet\\components\\genet-webrtc)","    Finished `dev` profile [unoptimized] target(s) in 17.70s"]}
```


## E2h automated browser interoperability - 2026-11-02T20:03:54.787Z

```json
{"schema":1,"gate":"E2h","utc":"2026-11-02T20:03:54.787Z","source":"d644fa0a013364a3d3437b30e98ef3da08dccbb2","node":"v24.11.0","command":["node","support/ci/webrtc_interop.mjs","--server","172.28.40.199:34790"],"expectedCases":32,"completeMatrix":true,"nativeBinarySHA256":"eee9ff809eb71d647a9679881b494dfebadb05d8642c9e2c5536a88b8b041536","sourceSHA256":{"components/genet-webrtc/examples/interop_host.rs":"821b677b6bf201647cca780409886e0ef46b2cf6c52eaec4baab36d6febe394b","support/ci/webrtc_interop.mjs":"9406d01b856275312b60a291ec29d9a52f52e4c5b5c8abddd847adf9176b2c54","support/ci/webrtc_interop_peer.js":"e4defa0fb7bc53407f64a3151c39aef45c3ab82ef959a39a575b02f093edfa28","components/genet-webrtc/Cargo.toml":"a0c7b33c3e49b10ba972f67c1169c674046cb6c5e717caa9f6ed2d1c0e3a4265","components/genet-webrtc/src/lib.rs":"0bae61d087b330f70fef6e7d6a000b554d1867ab02b968c54dcbca3017a23b19","components/script-runtime-api/webrtc/bootstrap.js":"b16a9c2cc771628ab6800492789278dec6fccc524c8308e2d03272f3b77c1bdd"},"versions":[{"browser":"chrome","version":"Chrome/156.0.8078.74","headed":true,"visibleWindows":1,"webrtcPreferenceOverrides":false,"networkPermissions":[]},{"browser":"firefox","version":"159.0","headed":true,"visibleWindows":1,"webrtcPreferenceOverrides":false,"networkPermissions":[{"permission":"local-network","before":"prompt","after":"granted","error":null,"reset":"prompt"},{"permission":"loopback-network","before":"prompt","after":"granted","error":null,"reset":"prompt"}]}],"cases":[{"browser":"chrome","engine":"boa","route":"direct","offerer":"native","cleanup":"close","status":"pass","seconds":16.82,"native":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"unreliable","ordered":false,"maxRetransmits":0},{"label":"reliable","ordered":true,"maxRetransmits":null}],"refused":["mdns:OperationError","mdns:OperationError"],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"host","remote":"prflx","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"994ca3e0617e30c6707b2a35686cebe307825f08b762cf686223bf41735ad68e","remoteFingerprint":"564ea216fb1abe2e820ce1e6228a1fb43359e43369be4de8a243e4407b19138b","remoteDER_SHA256":["564ea216fb1abe2e820ce1e6228a1fb43359e43369be4de8a243e4407b19138b"],"statsRemoteFingerprint":"564ea216fb1abe2e820ce1e6228a1fb43359e43369be4de8a243e4407b19138b"},"browserPeer":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"unreliable","ordered":false,"maxRetransmits":0},{"label":"reliable","ordered":true,"maxRetransmits":null}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"host","remote":"host","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"564ea216fb1abe2e820ce1e6228a1fb43359e43369be4de8a243e4407b19138b","remoteFingerprint":"994ca3e0617e30c6707b2a35686cebe307825f08b762cf686223bf41735ad68e","remoteDER_SHA256":["994ca3e0617e30c6707b2a35686cebe307825f08b762cf686223bf41735ad68e"],"statsRemoteFingerprint":"994ca3e0617e30c6707b2a35686cebe307825f08b762cf686223bf41735ad68e"},"cleanupVerified":true,"socketRebound":true},{"browser":"chrome","engine":"boa","route":"direct","offerer":"native","cleanup":"navigate","status":"pass","seconds":16.97,"native":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":["mdns:OperationError","mdns:OperationError"],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"host","remote":"prflx","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"b39fe1ca4671de9b0699aae27a18f6e2aa73e4a92258affd07e93c77c7a499ae","remoteFingerprint":"a6ea5f63e14f58703cec556c57ffb39198b9d4bb61af9dfc1aa36d71e97d7e43","remoteDER_SHA256":["a6ea5f63e14f58703cec556c57ffb39198b9d4bb61af9dfc1aa36d71e97d7e43"],"statsRemoteFingerprint":"a6ea5f63e14f58703cec556c57ffb39198b9d4bb61af9dfc1aa36d71e97d7e43"},"browserPeer":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"host","remote":"host","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"a6ea5f63e14f58703cec556c57ffb39198b9d4bb61af9dfc1aa36d71e97d7e43","remoteFingerprint":"b39fe1ca4671de9b0699aae27a18f6e2aa73e4a92258affd07e93c77c7a499ae","remoteDER_SHA256":["b39fe1ca4671de9b0699aae27a18f6e2aa73e4a92258affd07e93c77c7a499ae"],"statsRemoteFingerprint":"b39fe1ca4671de9b0699aae27a18f6e2aa73e4a92258affd07e93c77c7a499ae"},"cleanupVerified":true,"socketRebound":true},{"browser":"chrome","engine":"boa","route":"direct","offerer":"browser","cleanup":"close","status":"pass","seconds":12.64,"native":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":["mdns:OperationError","mdns:OperationError"],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"host","remote":"prflx","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"61bf72e9276feaaa81bd18d086f01ad97e368accfe6d8017c0a2db3b1967d647","remoteFingerprint":"1be4233a635ac124448e2d34d88df19d4f1488068e549f932dfc2dc7490040c5","remoteDER_SHA256":["1be4233a635ac124448e2d34d88df19d4f1488068e549f932dfc2dc7490040c5"],"statsRemoteFingerprint":"1be4233a635ac124448e2d34d88df19d4f1488068e549f932dfc2dc7490040c5"},"browserPeer":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"host","remote":"host","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"1be4233a635ac124448e2d34d88df19d4f1488068e549f932dfc2dc7490040c5","remoteFingerprint":"61bf72e9276feaaa81bd18d086f01ad97e368accfe6d8017c0a2db3b1967d647","remoteDER_SHA256":["61bf72e9276feaaa81bd18d086f01ad97e368accfe6d8017c0a2db3b1967d647"],"statsRemoteFingerprint":"61bf72e9276feaaa81bd18d086f01ad97e368accfe6d8017c0a2db3b1967d647"},"cleanupVerified":true,"socketRebound":true},{"browser":"chrome","engine":"boa","route":"direct","offerer":"browser","cleanup":"navigate","status":"pass","seconds":15.62,"native":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":["mdns:OperationError","mdns:OperationError"],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"host","remote":"prflx","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"ea5b448af46b596e862ca21f3940f49c0c82df473d3caf702d1a329500a87153","remoteFingerprint":"5ad854a441527a321dd84545c76d0e679074afa0da39e041986781b9a360ca0a","remoteDER_SHA256":["5ad854a441527a321dd84545c76d0e679074afa0da39e041986781b9a360ca0a"],"statsRemoteFingerprint":"5ad854a441527a321dd84545c76d0e679074afa0da39e041986781b9a360ca0a"},"browserPeer":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"host","remote":"host","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"5ad854a441527a321dd84545c76d0e679074afa0da39e041986781b9a360ca0a","remoteFingerprint":"ea5b448af46b596e862ca21f3940f49c0c82df473d3caf702d1a329500a87153","remoteDER_SHA256":["ea5b448af46b596e862ca21f3940f49c0c82df473d3caf702d1a329500a87153"],"statsRemoteFingerprint":"ea5b448af46b596e862ca21f3940f49c0c82df473d3caf702d1a329500a87153"},"cleanupVerified":true,"socketRebound":true},{"browser":"chrome","engine":"boa","route":"relay","offerer":"native","cleanup":"close","status":"pass","seconds":16.28,"native":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"relay","remote":"relay","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"6a8d7c3fcfe4acd64c7f2147dbead6cec7d975d418d587f10e2fe366860de8f7","remoteFingerprint":"f6aac549d878162e12b9f087ff9f62f6db435712df68f98f7e77f17a7a197344","remoteDER_SHA256":["f6aac549d878162e12b9f087ff9f62f6db435712df68f98f7e77f17a7a197344"],"statsRemoteFingerprint":"f6aac549d878162e12b9f087ff9f62f6db435712df68f98f7e77f17a7a197344"},"browserPeer":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"relay","remote":"relay","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"f6aac549d878162e12b9f087ff9f62f6db435712df68f98f7e77f17a7a197344","remoteFingerprint":"6a8d7c3fcfe4acd64c7f2147dbead6cec7d975d418d587f10e2fe366860de8f7","remoteDER_SHA256":["6a8d7c3fcfe4acd64c7f2147dbead6cec7d975d418d587f10e2fe366860de8f7"],"statsRemoteFingerprint":"6a8d7c3fcfe4acd64c7f2147dbead6cec7d975d418d587f10e2fe366860de8f7"},"cleanupVerified":true,"socketRebound":true},{"browser":"chrome","engine":"boa","route":"relay","offerer":"native","cleanup":"navigate","status":"pass","seconds":19.63,"native":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"relay","remote":"relay","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"b3505c92d441c42f37ceda0ece1dd0f808df3376ac135c0c9a2244529e994d11","remoteFingerprint":"30eb394736e24a77afb7d787a2fdc56610e0e9af42a6d02632f5bca20dd0d5b4","remoteDER_SHA256":["30eb394736e24a77afb7d787a2fdc56610e0e9af42a6d02632f5bca20dd0d5b4"],"statsRemoteFingerprint":"30eb394736e24a77afb7d787a2fdc56610e0e9af42a6d02632f5bca20dd0d5b4"},"browserPeer":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"relay","remote":"relay","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"30eb394736e24a77afb7d787a2fdc56610e0e9af42a6d02632f5bca20dd0d5b4","remoteFingerprint":"b3505c92d441c42f37ceda0ece1dd0f808df3376ac135c0c9a2244529e994d11","remoteDER_SHA256":["b3505c92d441c42f37ceda0ece1dd0f808df3376ac135c0c9a2244529e994d11"],"statsRemoteFingerprint":"b3505c92d441c42f37ceda0ece1dd0f808df3376ac135c0c9a2244529e994d11"},"cleanupVerified":true,"socketRebound":true},{"browser":"chrome","engine":"boa","route":"relay","offerer":"browser","cleanup":"close","status":"pass","seconds":12.99,"native":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"relay","remote":"relay","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"ba1d72dc5a2486465bd806341ac694fb38a8cd34d1694abe883c36a68dd31f50","remoteFingerprint":"0467cc25c9123fb0cde1c5727d14d5db7ab2bb2aa16ef4be78c24529ef02e8fa","remoteDER_SHA256":["0467cc25c9123fb0cde1c5727d14d5db7ab2bb2aa16ef4be78c24529ef02e8fa"],"statsRemoteFingerprint":"0467cc25c9123fb0cde1c5727d14d5db7ab2bb2aa16ef4be78c24529ef02e8fa"},"browserPeer":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"relay","remote":"relay","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"0467cc25c9123fb0cde1c5727d14d5db7ab2bb2aa16ef4be78c24529ef02e8fa","remoteFingerprint":"ba1d72dc5a2486465bd806341ac694fb38a8cd34d1694abe883c36a68dd31f50","remoteDER_SHA256":["ba1d72dc5a2486465bd806341ac694fb38a8cd34d1694abe883c36a68dd31f50"],"statsRemoteFingerprint":"ba1d72dc5a2486465bd806341ac694fb38a8cd34d1694abe883c36a68dd31f50"},"cleanupVerified":true,"socketRebound":true},{"browser":"chrome","engine":"boa","route":"relay","offerer":"browser","cleanup":"navigate","status":"pass","seconds":16.4,"native":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"relay","remote":"relay","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"f1f1b63bc40173ed4b84639bb2883b67983a57d059030b4c6177bcf42debb135","remoteFingerprint":"e6ae1d60cbfebff03cf2450b1d8ae12c875e1dffbdb92bac30cb434f429bbdf3","remoteDER_SHA256":["e6ae1d60cbfebff03cf2450b1d8ae12c875e1dffbdb92bac30cb434f429bbdf3"],"statsRemoteFingerprint":"e6ae1d60cbfebff03cf2450b1d8ae12c875e1dffbdb92bac30cb434f429bbdf3"},"browserPeer":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"relay","remote":"relay","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"e6ae1d60cbfebff03cf2450b1d8ae12c875e1dffbdb92bac30cb434f429bbdf3","remoteFingerprint":"f1f1b63bc40173ed4b84639bb2883b67983a57d059030b4c6177bcf42debb135","remoteDER_SHA256":["f1f1b63bc40173ed4b84639bb2883b67983a57d059030b4c6177bcf42debb135"],"statsRemoteFingerprint":"f1f1b63bc40173ed4b84639bb2883b67983a57d059030b4c6177bcf42debb135"},"cleanupVerified":true,"socketRebound":true},{"browser":"chrome","engine":"nova","route":"direct","offerer":"native","cleanup":"close","status":"pass","seconds":13.3,"native":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":["mdns:OperationError","mdns:OperationError"],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"host","remote":"prflx","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"9325ecbc59b0091fb637234362d19079000ab00b18e19291cb3fe880491934a8","remoteFingerprint":"38a7096c81e79cc3230c37c70dcbfe79f1434dbda31a0e98e769b9f0765fe837","remoteDER_SHA256":["38a7096c81e79cc3230c37c70dcbfe79f1434dbda31a0e98e769b9f0765fe837"],"statsRemoteFingerprint":"38a7096c81e79cc3230c37c70dcbfe79f1434dbda31a0e98e769b9f0765fe837"},"browserPeer":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"host","remote":"host","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"38a7096c81e79cc3230c37c70dcbfe79f1434dbda31a0e98e769b9f0765fe837","remoteFingerprint":"9325ecbc59b0091fb637234362d19079000ab00b18e19291cb3fe880491934a8","remoteDER_SHA256":["9325ecbc59b0091fb637234362d19079000ab00b18e19291cb3fe880491934a8"],"statsRemoteFingerprint":"9325ecbc59b0091fb637234362d19079000ab00b18e19291cb3fe880491934a8"},"cleanupVerified":true,"socketRebound":true},{"browser":"chrome","engine":"nova","route":"direct","offerer":"native","cleanup":"navigate","status":"pass","seconds":12.03,"native":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":["mdns:OperationError","mdns:OperationError"],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"host","remote":"prflx","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"2fb907a4d66980b58bba95320fbee4ba18043332558fcd91c498178d49d77c4d","remoteFingerprint":"dc92287c00922e0b808d111be3ce7b2125826d0487085e75568854b0cce79b5e","remoteDER_SHA256":["dc92287c00922e0b808d111be3ce7b2125826d0487085e75568854b0cce79b5e"],"statsRemoteFingerprint":"dc92287c00922e0b808d111be3ce7b2125826d0487085e75568854b0cce79b5e"},"browserPeer":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"host","remote":"host","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"dc92287c00922e0b808d111be3ce7b2125826d0487085e75568854b0cce79b5e","remoteFingerprint":"2fb907a4d66980b58bba95320fbee4ba18043332558fcd91c498178d49d77c4d","remoteDER_SHA256":["2fb907a4d66980b58bba95320fbee4ba18043332558fcd91c498178d49d77c4d"],"statsRemoteFingerprint":"2fb907a4d66980b58bba95320fbee4ba18043332558fcd91c498178d49d77c4d"},"cleanupVerified":true,"socketRebound":true},{"browser":"chrome","engine":"nova","route":"direct","offerer":"browser","cleanup":"close","status":"pass","seconds":9.42,"native":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":["mdns:OperationError","mdns:OperationError"],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"host","remote":"prflx","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"31b4a2f7e89c174b43ebfeb5cee178c67fb235b19ba8235c2597e637ddc33b24","remoteFingerprint":"af84c7bf6ef1bcbb5346a127d2bc178ae43a5df299140bbcfabc86a174a604dd","remoteDER_SHA256":["af84c7bf6ef1bcbb5346a127d2bc178ae43a5df299140bbcfabc86a174a604dd"],"statsRemoteFingerprint":"af84c7bf6ef1bcbb5346a127d2bc178ae43a5df299140bbcfabc86a174a604dd"},"browserPeer":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"host","remote":"host","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"af84c7bf6ef1bcbb5346a127d2bc178ae43a5df299140bbcfabc86a174a604dd","remoteFingerprint":"31b4a2f7e89c174b43ebfeb5cee178c67fb235b19ba8235c2597e637ddc33b24","remoteDER_SHA256":["31b4a2f7e89c174b43ebfeb5cee178c67fb235b19ba8235c2597e637ddc33b24"],"statsRemoteFingerprint":"31b4a2f7e89c174b43ebfeb5cee178c67fb235b19ba8235c2597e637ddc33b24"},"cleanupVerified":true,"socketRebound":true},{"browser":"chrome","engine":"nova","route":"direct","offerer":"browser","cleanup":"navigate","status":"pass","seconds":10.45,"native":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":["mdns:OperationError","mdns:OperationError"],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"host","remote":"prflx","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"ffa7dd280cf3cadbd6787e1c81eff6d3942bffc4cfc15a3fb6866984175d8ea5","remoteFingerprint":"ebb841be19cc13a9b8acb6984e0f53ec9408fb3fc4fe874042fd4264f5ca9ee2","remoteDER_SHA256":["ebb841be19cc13a9b8acb6984e0f53ec9408fb3fc4fe874042fd4264f5ca9ee2"],"statsRemoteFingerprint":"ebb841be19cc13a9b8acb6984e0f53ec9408fb3fc4fe874042fd4264f5ca9ee2"},"browserPeer":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"host","remote":"host","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"ebb841be19cc13a9b8acb6984e0f53ec9408fb3fc4fe874042fd4264f5ca9ee2","remoteFingerprint":"ffa7dd280cf3cadbd6787e1c81eff6d3942bffc4cfc15a3fb6866984175d8ea5","remoteDER_SHA256":["ffa7dd280cf3cadbd6787e1c81eff6d3942bffc4cfc15a3fb6866984175d8ea5"],"statsRemoteFingerprint":"ffa7dd280cf3cadbd6787e1c81eff6d3942bffc4cfc15a3fb6866984175d8ea5"},"cleanupVerified":true,"socketRebound":true},{"browser":"chrome","engine":"nova","route":"relay","offerer":"native","cleanup":"close","status":"pass","seconds":12.47,"native":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"relay","remote":"relay","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"0dcaae672105f68a5164e24b75c925d0124494bc821cd429645a21999237a81a","remoteFingerprint":"35dc42dfc0e861e6332c248f2a481f20ee25f6c0ea6288356d52256f1608caa9","remoteDER_SHA256":["35dc42dfc0e861e6332c248f2a481f20ee25f6c0ea6288356d52256f1608caa9"],"statsRemoteFingerprint":"35dc42dfc0e861e6332c248f2a481f20ee25f6c0ea6288356d52256f1608caa9"},"browserPeer":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"relay","remote":"relay","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"35dc42dfc0e861e6332c248f2a481f20ee25f6c0ea6288356d52256f1608caa9","remoteFingerprint":"0dcaae672105f68a5164e24b75c925d0124494bc821cd429645a21999237a81a","remoteDER_SHA256":["0dcaae672105f68a5164e24b75c925d0124494bc821cd429645a21999237a81a"],"statsRemoteFingerprint":"0dcaae672105f68a5164e24b75c925d0124494bc821cd429645a21999237a81a"},"cleanupVerified":true,"socketRebound":true},{"browser":"chrome","engine":"nova","route":"relay","offerer":"native","cleanup":"navigate","status":"pass","seconds":13.58,"native":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"unreliable","ordered":false,"maxRetransmits":0},{"label":"reliable","ordered":true,"maxRetransmits":null}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"relay","remote":"relay","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"14bda565252388fcdf761c226652714341b5a9715234213f8ddf90a10bc15c4a","remoteFingerprint":"c9f01fbb0afa81106d3bf44384b86500b46e51118666c27e318195c44099445a","remoteDER_SHA256":["c9f01fbb0afa81106d3bf44384b86500b46e51118666c27e318195c44099445a"],"statsRemoteFingerprint":"c9f01fbb0afa81106d3bf44384b86500b46e51118666c27e318195c44099445a"},"browserPeer":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"unreliable","ordered":false,"maxRetransmits":0},{"label":"reliable","ordered":true,"maxRetransmits":null}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"relay","remote":"relay","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"c9f01fbb0afa81106d3bf44384b86500b46e51118666c27e318195c44099445a","remoteFingerprint":"14bda565252388fcdf761c226652714341b5a9715234213f8ddf90a10bc15c4a","remoteDER_SHA256":["14bda565252388fcdf761c226652714341b5a9715234213f8ddf90a10bc15c4a"],"statsRemoteFingerprint":"14bda565252388fcdf761c226652714341b5a9715234213f8ddf90a10bc15c4a"},"cleanupVerified":true,"socketRebound":true},{"browser":"chrome","engine":"nova","route":"relay","offerer":"browser","cleanup":"close","status":"pass","seconds":9.52,"native":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"relay","remote":"relay","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"482787d3b5d5bc749461f2f5f6bc36f9294f19fbdb32978e375bff69533acc47","remoteFingerprint":"edc7fe5346ce14af9a1e4789eb1fa98484a00f800c89a58e77f8e2c54ae988b4","remoteDER_SHA256":["edc7fe5346ce14af9a1e4789eb1fa98484a00f800c89a58e77f8e2c54ae988b4"],"statsRemoteFingerprint":"edc7fe5346ce14af9a1e4789eb1fa98484a00f800c89a58e77f8e2c54ae988b4"},"browserPeer":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"relay","remote":"relay","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"edc7fe5346ce14af9a1e4789eb1fa98484a00f800c89a58e77f8e2c54ae988b4","remoteFingerprint":"482787d3b5d5bc749461f2f5f6bc36f9294f19fbdb32978e375bff69533acc47","remoteDER_SHA256":["482787d3b5d5bc749461f2f5f6bc36f9294f19fbdb32978e375bff69533acc47"],"statsRemoteFingerprint":"482787d3b5d5bc749461f2f5f6bc36f9294f19fbdb32978e375bff69533acc47"},"cleanupVerified":true,"socketRebound":true},{"browser":"chrome","engine":"nova","route":"relay","offerer":"browser","cleanup":"navigate","status":"pass","seconds":10.53,"native":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"relay","remote":"relay","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"0e3844db93730e9a31ca4568c51e514580f5f03b77583dabf3ac53969dc5aab2","remoteFingerprint":"8da118868fbf3ce340f7332eacf8aa297eeb602c9230945b142cd88ee9e72156","remoteDER_SHA256":["8da118868fbf3ce340f7332eacf8aa297eeb602c9230945b142cd88ee9e72156"],"statsRemoteFingerprint":"8da118868fbf3ce340f7332eacf8aa297eeb602c9230945b142cd88ee9e72156"},"browserPeer":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"relay","remote":"relay","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"8da118868fbf3ce340f7332eacf8aa297eeb602c9230945b142cd88ee9e72156","remoteFingerprint":"0e3844db93730e9a31ca4568c51e514580f5f03b77583dabf3ac53969dc5aab2","remoteDER_SHA256":["0e3844db93730e9a31ca4568c51e514580f5f03b77583dabf3ac53969dc5aab2"],"statsRemoteFingerprint":"0e3844db93730e9a31ca4568c51e514580f5f03b77583dabf3ac53969dc5aab2"},"cleanupVerified":true,"socketRebound":true},{"browser":"firefox","engine":"boa","route":"direct","offerer":"native","cleanup":"close","status":"pass","seconds":13.78,"native":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":["mdns:OperationError","mdns:OperationError","mdns:OperationError","mdns:OperationError"],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"host","remote":"prflx","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"6fa796657e192f5684b026bc87db8cc06f332ba22bff06062fd57e3602364f60","remoteFingerprint":"8aa1483e244d52a22cfcc0ee1e1cba948b6856e83f32b38a3d0b188f24d5e415","remoteDER_SHA256":["8aa1483e244d52a22cfcc0ee1e1cba948b6856e83f32b38a3d0b188f24d5e415"],"statsRemoteFingerprint":"8aa1483e244d52a22cfcc0ee1e1cba948b6856e83f32b38a3d0b188f24d5e415"},"browserPeer":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"host","remote":"host","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"8aa1483e244d52a22cfcc0ee1e1cba948b6856e83f32b38a3d0b188f24d5e415","remoteFingerprint":"6fa796657e192f5684b026bc87db8cc06f332ba22bff06062fd57e3602364f60","remoteDER_SHA256":["6fa796657e192f5684b026bc87db8cc06f332ba22bff06062fd57e3602364f60"],"statsRemoteFingerprint":"6fa796657e192f5684b026bc87db8cc06f332ba22bff06062fd57e3602364f60"},"cleanupVerified":true,"socketRebound":true},{"browser":"firefox","engine":"boa","route":"direct","offerer":"native","cleanup":"navigate","status":"pass","seconds":16.66,"native":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":["mdns:OperationError","mdns:OperationError","mdns:OperationError","mdns:OperationError"],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"host","remote":"prflx","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"756c472b3348d55dd7b58fb9266013baf666fff5023f1e41149dd73930ce5914","remoteFingerprint":"032d3b54b1da218cbeb25ed6f7a4a5bf8b016780f0da68a40be0927950a59a1c","remoteDER_SHA256":["032d3b54b1da218cbeb25ed6f7a4a5bf8b016780f0da68a40be0927950a59a1c"],"statsRemoteFingerprint":"032d3b54b1da218cbeb25ed6f7a4a5bf8b016780f0da68a40be0927950a59a1c"},"browserPeer":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"host","remote":"host","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"032d3b54b1da218cbeb25ed6f7a4a5bf8b016780f0da68a40be0927950a59a1c","remoteFingerprint":"756c472b3348d55dd7b58fb9266013baf666fff5023f1e41149dd73930ce5914","remoteDER_SHA256":["756c472b3348d55dd7b58fb9266013baf666fff5023f1e41149dd73930ce5914"],"statsRemoteFingerprint":"756c472b3348d55dd7b58fb9266013baf666fff5023f1e41149dd73930ce5914"},"cleanupVerified":true,"socketRebound":true},{"browser":"firefox","engine":"boa","route":"direct","offerer":"browser","cleanup":"close","status":"pass","seconds":16.9,"native":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":["mdns:OperationError","mdns:OperationError","mdns:OperationError","mdns:OperationError"],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"host","remote":"prflx","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"b27ea6cf6618d25e582f8874da95c7f6ce0c0f5b63d38fd90ff7143565ca3044","remoteFingerprint":"b197c41d56b8bbafbe393ec1e41c342b67cfedad431242dda2c7a71e7ae4c9bb","remoteDER_SHA256":["b197c41d56b8bbafbe393ec1e41c342b67cfedad431242dda2c7a71e7ae4c9bb"],"statsRemoteFingerprint":"b197c41d56b8bbafbe393ec1e41c342b67cfedad431242dda2c7a71e7ae4c9bb"},"browserPeer":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"host","remote":"host","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"b197c41d56b8bbafbe393ec1e41c342b67cfedad431242dda2c7a71e7ae4c9bb","remoteFingerprint":"b27ea6cf6618d25e582f8874da95c7f6ce0c0f5b63d38fd90ff7143565ca3044","remoteDER_SHA256":["b27ea6cf6618d25e582f8874da95c7f6ce0c0f5b63d38fd90ff7143565ca3044"],"statsRemoteFingerprint":"b27ea6cf6618d25e582f8874da95c7f6ce0c0f5b63d38fd90ff7143565ca3044"},"cleanupVerified":true,"socketRebound":true},{"browser":"firefox","engine":"boa","route":"direct","offerer":"browser","cleanup":"navigate","status":"pass","seconds":15.62,"native":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":["mdns:OperationError","mdns:OperationError","mdns:OperationError","mdns:OperationError"],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"host","remote":"prflx","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"cca9ee125291df4461ec49da0d99b8059354921ded1f45b0f89723a49148eddd","remoteFingerprint":"abd226115d6521088a1a8cb0f9f293c1ef54b7ef7049ad0b350532e921a55f2e","remoteDER_SHA256":["abd226115d6521088a1a8cb0f9f293c1ef54b7ef7049ad0b350532e921a55f2e"],"statsRemoteFingerprint":"abd226115d6521088a1a8cb0f9f293c1ef54b7ef7049ad0b350532e921a55f2e"},"browserPeer":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"host","remote":"host","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"abd226115d6521088a1a8cb0f9f293c1ef54b7ef7049ad0b350532e921a55f2e","remoteFingerprint":"cca9ee125291df4461ec49da0d99b8059354921ded1f45b0f89723a49148eddd","remoteDER_SHA256":["cca9ee125291df4461ec49da0d99b8059354921ded1f45b0f89723a49148eddd"],"statsRemoteFingerprint":"cca9ee125291df4461ec49da0d99b8059354921ded1f45b0f89723a49148eddd"},"cleanupVerified":true,"socketRebound":true},{"browser":"firefox","engine":"boa","route":"relay","offerer":"native","cleanup":"close","status":"pass","seconds":14.66,"native":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"relay","remote":"relay","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"73844c1fcf55809e5be36bbb9867392af6f11d3a312fc69115a32450bc1b5e49","remoteFingerprint":"0ddccbe0fa584ca11b4456cb46afc8c5deaced598ac1de8064240edda1cad3e4","remoteDER_SHA256":["0ddccbe0fa584ca11b4456cb46afc8c5deaced598ac1de8064240edda1cad3e4"],"statsRemoteFingerprint":"0ddccbe0fa584ca11b4456cb46afc8c5deaced598ac1de8064240edda1cad3e4"},"browserPeer":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"relay","remote":"relay","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"0ddccbe0fa584ca11b4456cb46afc8c5deaced598ac1de8064240edda1cad3e4","remoteFingerprint":"73844c1fcf55809e5be36bbb9867392af6f11d3a312fc69115a32450bc1b5e49","remoteDER_SHA256":["73844c1fcf55809e5be36bbb9867392af6f11d3a312fc69115a32450bc1b5e49"],"statsRemoteFingerprint":"73844c1fcf55809e5be36bbb9867392af6f11d3a312fc69115a32450bc1b5e49"},"cleanupVerified":true,"socketRebound":true},{"browser":"firefox","engine":"boa","route":"relay","offerer":"native","cleanup":"navigate","status":"pass","seconds":17.98,"native":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"relay","remote":"relay","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"20fee7ab25b6798520b0fc54455ffc3da1ec6ff0b1ab718ede46622bc39b4656","remoteFingerprint":"a02d80be504a7de838e402fee9f53391318ce0f85b220cba4df258ffaaaf3e90","remoteDER_SHA256":["a02d80be504a7de838e402fee9f53391318ce0f85b220cba4df258ffaaaf3e90"],"statsRemoteFingerprint":"a02d80be504a7de838e402fee9f53391318ce0f85b220cba4df258ffaaaf3e90"},"browserPeer":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"relay","remote":"relay","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"a02d80be504a7de838e402fee9f53391318ce0f85b220cba4df258ffaaaf3e90","remoteFingerprint":"20fee7ab25b6798520b0fc54455ffc3da1ec6ff0b1ab718ede46622bc39b4656","remoteDER_SHA256":["20fee7ab25b6798520b0fc54455ffc3da1ec6ff0b1ab718ede46622bc39b4656"],"statsRemoteFingerprint":"20fee7ab25b6798520b0fc54455ffc3da1ec6ff0b1ab718ede46622bc39b4656"},"cleanupVerified":true,"socketRebound":true},{"browser":"firefox","engine":"boa","route":"relay","offerer":"browser","cleanup":"close","status":"pass","seconds":12.73,"native":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"relay","remote":"relay","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"33006df2e875b0fec70cd928a61537a6ebbc67101d52c751dfda9a1f6c88b73b","remoteFingerprint":"75db258d3300473871e7d7ab9b784fbabd7cfee643204d0240203ed8e95095d6","remoteDER_SHA256":["75db258d3300473871e7d7ab9b784fbabd7cfee643204d0240203ed8e95095d6"],"statsRemoteFingerprint":"75db258d3300473871e7d7ab9b784fbabd7cfee643204d0240203ed8e95095d6"},"browserPeer":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"relay","remote":"relay","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"75db258d3300473871e7d7ab9b784fbabd7cfee643204d0240203ed8e95095d6","remoteFingerprint":"33006df2e875b0fec70cd928a61537a6ebbc67101d52c751dfda9a1f6c88b73b","remoteDER_SHA256":["33006df2e875b0fec70cd928a61537a6ebbc67101d52c751dfda9a1f6c88b73b"],"statsRemoteFingerprint":"33006df2e875b0fec70cd928a61537a6ebbc67101d52c751dfda9a1f6c88b73b"},"cleanupVerified":true,"socketRebound":true},{"browser":"firefox","engine":"boa","route":"relay","offerer":"browser","cleanup":"navigate","status":"pass","seconds":16.01,"native":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"relay","remote":"relay","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"5e02cd1d2a4ab65f37b6a7c16d9ad3484aae71abba80729e768333d34a7cbc01","remoteFingerprint":"659b27e8919125e5bdaedf605fa40c18e23321a3c194bcc3d02df734d1e971a3","remoteDER_SHA256":["659b27e8919125e5bdaedf605fa40c18e23321a3c194bcc3d02df734d1e971a3"],"statsRemoteFingerprint":"659b27e8919125e5bdaedf605fa40c18e23321a3c194bcc3d02df734d1e971a3"},"browserPeer":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"relay","remote":"relay","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"659b27e8919125e5bdaedf605fa40c18e23321a3c194bcc3d02df734d1e971a3","remoteFingerprint":"5e02cd1d2a4ab65f37b6a7c16d9ad3484aae71abba80729e768333d34a7cbc01","remoteDER_SHA256":["5e02cd1d2a4ab65f37b6a7c16d9ad3484aae71abba80729e768333d34a7cbc01"],"statsRemoteFingerprint":"5e02cd1d2a4ab65f37b6a7c16d9ad3484aae71abba80729e768333d34a7cbc01"},"cleanupVerified":true,"socketRebound":true},{"browser":"firefox","engine":"nova","route":"direct","offerer":"native","cleanup":"close","status":"pass","seconds":10.04,"native":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":["mdns:OperationError","mdns:OperationError","mdns:OperationError","mdns:OperationError"],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"host","remote":"prflx","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"4b72bc28aa547c2ab7e74176810b4252b6ec6e1f227a01d1debee7c4b8941de7","remoteFingerprint":"98616a52c4e651e6f117814b32f241049b580f1ebb7f71f1ee1086c51d2e0e1e","remoteDER_SHA256":["98616a52c4e651e6f117814b32f241049b580f1ebb7f71f1ee1086c51d2e0e1e"],"statsRemoteFingerprint":"98616a52c4e651e6f117814b32f241049b580f1ebb7f71f1ee1086c51d2e0e1e"},"browserPeer":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"host","remote":"host","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"98616a52c4e651e6f117814b32f241049b580f1ebb7f71f1ee1086c51d2e0e1e","remoteFingerprint":"4b72bc28aa547c2ab7e74176810b4252b6ec6e1f227a01d1debee7c4b8941de7","remoteDER_SHA256":["4b72bc28aa547c2ab7e74176810b4252b6ec6e1f227a01d1debee7c4b8941de7"],"statsRemoteFingerprint":"4b72bc28aa547c2ab7e74176810b4252b6ec6e1f227a01d1debee7c4b8941de7"},"cleanupVerified":true,"socketRebound":true},{"browser":"firefox","engine":"nova","route":"direct","offerer":"native","cleanup":"navigate","status":"pass","seconds":11.89,"native":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"unreliable","ordered":false,"maxRetransmits":0},{"label":"reliable","ordered":true,"maxRetransmits":null}],"refused":["mdns:OperationError","mdns:OperationError","mdns:OperationError","mdns:OperationError"],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"host","remote":"prflx","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"fe2ae05d78c4177cccbf74fe4b79cb9cb572b78a61f9b9363cae418a2ba8c11e","remoteFingerprint":"7ec916af072472e4ad8486a6435b8912bad48f5f317a0d6622648f09665050a6","remoteDER_SHA256":["7ec916af072472e4ad8486a6435b8912bad48f5f317a0d6622648f09665050a6"],"statsRemoteFingerprint":"7ec916af072472e4ad8486a6435b8912bad48f5f317a0d6622648f09665050a6"},"browserPeer":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"unreliable","ordered":false,"maxRetransmits":0},{"label":"reliable","ordered":true,"maxRetransmits":null}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"host","remote":"host","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"7ec916af072472e4ad8486a6435b8912bad48f5f317a0d6622648f09665050a6","remoteFingerprint":"fe2ae05d78c4177cccbf74fe4b79cb9cb572b78a61f9b9363cae418a2ba8c11e","remoteDER_SHA256":["fe2ae05d78c4177cccbf74fe4b79cb9cb572b78a61f9b9363cae418a2ba8c11e"],"statsRemoteFingerprint":"fe2ae05d78c4177cccbf74fe4b79cb9cb572b78a61f9b9363cae418a2ba8c11e"},"cleanupVerified":true,"socketRebound":true},{"browser":"firefox","engine":"nova","route":"direct","offerer":"browser","cleanup":"close","status":"pass","seconds":9.39,"native":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":["mdns:OperationError","mdns:OperationError","mdns:OperationError","mdns:OperationError"],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"host","remote":"prflx","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"3289e2c28ba0498aeaeff31d10aeee74bbe63829f55079c97d422dc09b9f9cd5","remoteFingerprint":"c27e42ec5f1f22253bbd80df3eb078e19fcd0687a632b3219aa1ddc4480e3c50","remoteDER_SHA256":["c27e42ec5f1f22253bbd80df3eb078e19fcd0687a632b3219aa1ddc4480e3c50"],"statsRemoteFingerprint":"c27e42ec5f1f22253bbd80df3eb078e19fcd0687a632b3219aa1ddc4480e3c50"},"browserPeer":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"host","remote":"host","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"c27e42ec5f1f22253bbd80df3eb078e19fcd0687a632b3219aa1ddc4480e3c50","remoteFingerprint":"3289e2c28ba0498aeaeff31d10aeee74bbe63829f55079c97d422dc09b9f9cd5","remoteDER_SHA256":["3289e2c28ba0498aeaeff31d10aeee74bbe63829f55079c97d422dc09b9f9cd5"],"statsRemoteFingerprint":"3289e2c28ba0498aeaeff31d10aeee74bbe63829f55079c97d422dc09b9f9cd5"},"cleanupVerified":true,"socketRebound":true},{"browser":"firefox","engine":"nova","route":"direct","offerer":"browser","cleanup":"navigate","status":"pass","seconds":10.73,"native":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":["mdns:OperationError","mdns:OperationError","mdns:OperationError","mdns:OperationError"],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"host","remote":"prflx","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"c1bf308eba2f8ed9dfe475bdd913cb0019b2a27d6fabc48e0ff5e4b19b9308d7","remoteFingerprint":"6d34d823908095955c516df1ee66a6091415e712bf09e307339816c7f69ad6da","remoteDER_SHA256":["6d34d823908095955c516df1ee66a6091415e712bf09e307339816c7f69ad6da"],"statsRemoteFingerprint":"6d34d823908095955c516df1ee66a6091415e712bf09e307339816c7f69ad6da"},"browserPeer":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"host","remote":"host","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"6d34d823908095955c516df1ee66a6091415e712bf09e307339816c7f69ad6da","remoteFingerprint":"c1bf308eba2f8ed9dfe475bdd913cb0019b2a27d6fabc48e0ff5e4b19b9308d7","remoteDER_SHA256":["c1bf308eba2f8ed9dfe475bdd913cb0019b2a27d6fabc48e0ff5e4b19b9308d7"],"statsRemoteFingerprint":"c1bf308eba2f8ed9dfe475bdd913cb0019b2a27d6fabc48e0ff5e4b19b9308d7"},"cleanupVerified":true,"socketRebound":true},{"browser":"firefox","engine":"nova","route":"relay","offerer":"native","cleanup":"close","status":"pass","seconds":10.95,"native":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"relay","remote":"relay","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"5d73d5c5c8e6f9990fc8a381950209609a660c5fb7d7387382b224d7d6f0b33d","remoteFingerprint":"1474feb619f8202bf29212c0a88c88c79f4c1f9d599bfd7f3aca6c4d2d1bd5f5","remoteDER_SHA256":["1474feb619f8202bf29212c0a88c88c79f4c1f9d599bfd7f3aca6c4d2d1bd5f5"],"statsRemoteFingerprint":"1474feb619f8202bf29212c0a88c88c79f4c1f9d599bfd7f3aca6c4d2d1bd5f5"},"browserPeer":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"relay","remote":"relay","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"1474feb619f8202bf29212c0a88c88c79f4c1f9d599bfd7f3aca6c4d2d1bd5f5","remoteFingerprint":"5d73d5c5c8e6f9990fc8a381950209609a660c5fb7d7387382b224d7d6f0b33d","remoteDER_SHA256":["5d73d5c5c8e6f9990fc8a381950209609a660c5fb7d7387382b224d7d6f0b33d"],"statsRemoteFingerprint":"5d73d5c5c8e6f9990fc8a381950209609a660c5fb7d7387382b224d7d6f0b33d"},"cleanupVerified":true,"socketRebound":true},{"browser":"firefox","engine":"nova","route":"relay","offerer":"native","cleanup":"navigate","status":"pass","seconds":12.51,"native":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"unreliable","ordered":false,"maxRetransmits":0},{"label":"reliable","ordered":true,"maxRetransmits":null}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"relay","remote":"relay","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"0b30ac8f0f4d898a4b74a27aab13db8a7a3b3ee94525fb0034f4f3edf736ef52","remoteFingerprint":"aa1e00192e47be80fa65b8be01312843941c4d3be6f64efd91e542afac4afbea","remoteDER_SHA256":["aa1e00192e47be80fa65b8be01312843941c4d3be6f64efd91e542afac4afbea"],"statsRemoteFingerprint":"aa1e00192e47be80fa65b8be01312843941c4d3be6f64efd91e542afac4afbea"},"browserPeer":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"unreliable","ordered":false,"maxRetransmits":0},{"label":"reliable","ordered":true,"maxRetransmits":null}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"relay","remote":"relay","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"aa1e00192e47be80fa65b8be01312843941c4d3be6f64efd91e542afac4afbea","remoteFingerprint":"0b30ac8f0f4d898a4b74a27aab13db8a7a3b3ee94525fb0034f4f3edf736ef52","remoteDER_SHA256":["0b30ac8f0f4d898a4b74a27aab13db8a7a3b3ee94525fb0034f4f3edf736ef52"],"statsRemoteFingerprint":"0b30ac8f0f4d898a4b74a27aab13db8a7a3b3ee94525fb0034f4f3edf736ef52"},"cleanupVerified":true,"socketRebound":true},{"browser":"firefox","engine":"nova","route":"relay","offerer":"browser","cleanup":"close","status":"pass","seconds":9.53,"native":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"relay","remote":"relay","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"13a4f8f39a980b090929a454b38d7db53497c421ddb334aa7c3d4933d826574e","remoteFingerprint":"206eac60207b4aec75dc81abcd6503a12e73ff10b7dfeafdd024a174057ad038","remoteDER_SHA256":["206eac60207b4aec75dc81abcd6503a12e73ff10b7dfeafdd024a174057ad038"],"statsRemoteFingerprint":"206eac60207b4aec75dc81abcd6503a12e73ff10b7dfeafdd024a174057ad038"},"browserPeer":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"relay","remote":"relay","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"206eac60207b4aec75dc81abcd6503a12e73ff10b7dfeafdd024a174057ad038","remoteFingerprint":"13a4f8f39a980b090929a454b38d7db53497c421ddb334aa7c3d4933d826574e","remoteDER_SHA256":["13a4f8f39a980b090929a454b38d7db53497c421ddb334aa7c3d4933d826574e"],"statsRemoteFingerprint":"13a4f8f39a980b090929a454b38d7db53497c421ddb334aa7c3d4933d826574e"},"cleanupVerified":true,"socketRebound":true},{"browser":"firefox","engine":"nova","route":"relay","offerer":"browser","cleanup":"navigate","status":"pass","seconds":10.66,"native":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"relay","remote":"relay","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"d579101f795e3e2aff410ad33c4aaacfe9ffd75f5b5322810f28f2a2ba9645c1","remoteFingerprint":"e423dedc1109801b890fd54b41f6fde1ba653e3dd3548df0f45999b955d1833e","remoteDER_SHA256":["e423dedc1109801b890fd54b41f6fde1ba653e3dd3548df0f45999b955d1833e"],"statsRemoteFingerprint":"e423dedc1109801b890fd54b41f6fde1ba653e3dd3548df0f45999b955d1833e"},"browserPeer":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"relay","remote":"relay","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"e423dedc1109801b890fd54b41f6fde1ba653e3dd3548df0f45999b955d1833e","remoteFingerprint":"d579101f795e3e2aff410ad33c4aaacfe9ffd75f5b5322810f28f2a2ba9645c1","remoteDER_SHA256":["d579101f795e3e2aff410ad33c4aaacfe9ffd75f5b5322810f28f2a2ba9645c1"],"statsRemoteFingerprint":"d579101f795e3e2aff410ad33c4aaacfe9ffd75f5b5322810f28f2a2ba9645c1"},"cleanupVerified":true,"socketRebound":true}],"failure":null,"relayFixture":"bounded Windows UDP front end forwarding unchanged datagrams to WSL coturn; distinct backend socket per client; max 8 clients/256 pending datagrams/1 MiB","profileCache":"C:\\t\\cargo-targets\\genet\\rtc-interop","profileCleanup":"automatic approval review blocked recursive deletion; marked profiles retained for reuse","scope":"native Windows to installed headed browsers, controlled WSL UDP server; no DNS/TCP/TLS/restart/media/carrier acceptance"}
```


## E2h automated browser interoperability - 2026-11-02T20:08:38.697Z

```json
{"schema":1,"gate":"E2h","utc":"2026-11-02T20:08:38.697Z","source":"d644fa0a013364a3d3437b30e98ef3da08dccbb2","node":"v24.11.0","command":["node","support/ci/webrtc_interop.mjs","--server","172.28.40.199:34790","--offerer","native","--cleanup","close"],"expectedCases":8,"completeMatrix":false,"nativeBinarySHA256":"818f9a3e181526e4eddb91c87d3dd8bd30c18c0d845e235e347676d941d2c475","sourceSHA256":{"components/genet-webrtc/examples/interop_host.rs":"821b677b6bf201647cca780409886e0ef46b2cf6c52eaec4baab36d6febe394b","support/ci/webrtc_interop.mjs":"d9e4512aab42d7f9ac5b05896879c3ba034aadc1d3fd7ab0e4063c8af5f30c46","support/ci/webrtc_interop_peer.js":"a74a1d9b0d42a096f7353f6f6abe11cae7a65a3defbea4579d0001ebf4c4f5b6","components/genet-webrtc/Cargo.toml":"a0c7b33c3e49b10ba972f67c1169c674046cb6c5e717caa9f6ed2d1c0e3a4265","components/genet-webrtc/src/lib.rs":"c91da3fea1701e320035a78e2f3d0f52edf89925d7ca25e6a2241828944ff279","components/script-runtime-api/webrtc/bootstrap.js":"b16a9c2cc771628ab6800492789278dec6fccc524c8308e2d03272f3b77c1bdd"},"versions":[{"browser":"chrome","version":"Chrome/156.0.8078.74","headed":true,"visibleWindows":1,"webrtcPreferenceOverrides":false,"networkPermissions":[]}],"cases":[{"browser":"chrome","engine":"boa","route":"direct","offerer":"native","cleanup":"close","status":"fail","phase":"negotiation and transfer","error":"application exchange deadline","observed":{"native":{"connected":true,"connectionState":"connected","iceState":"connected","signaling":"stable","remoteSet":true,"received":["browser:text:λ🙂:reliable"],"sent":["reliable:text","reliable:binary","unreliable:text"],"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"errors":[],"refused":["mdns:OperationError","mdns:OperationError"],"remoteProtocols":["udp","udp"],"gatheringErrors":[],"channelStates":["open","open"],"gathering":"complete"},"browser":{"connected":true,"connectionState":"connected","iceState":"connected","signaling":"stable","remoteSet":true,"received":[],"sent":["reliable:text","reliable:binary","unreliable:text"],"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"errors":[],"refused":[],"remoteProtocols":["udp"],"gatheringErrors":[],"channelStates":["open","open"],"gathering":"complete"}},"observedStats":{"native":{"pair":{"state":"succeeded","nominated":true,"local":"host","remote":"prflx","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"6a8fa85e36344a706be82d224623b315292e4bf4663a100f3ecd6c8d61b9e5b0","remoteFingerprint":"97a0a36c1cba12f2751396d81ca02519de1baf1e090cfb863d73a4da849f9498","statsRemoteFingerprint":"97a0a36c1cba12f2751396d81ca02519de1baf1e090cfb863d73a4da849f9498","remoteAddressesRedacted":true,"peerReflexiveInRemoteList":false,"pairKeys":{"local":"RTCLocalIceCandidate_candidate:0jVOPigh8q7dJxIMDo0egFFSw35e6eI7","remote":"RTCRemoteIceCandidate_candidate:Dexv58c3KZfFCzkpxvjcAq9vTMvXVcWn"},"candidateKeys":[{"id":"RTCLocalIceCandidate_candidate:0jVOPigh8q7dJxIMDo0egFFSw35e6eI7","type":"local-candidate","candidateType":"host","protocol":"udp"},{"id":"RTCRemoteIceCandidate_candidate:Dexv58c3KZfFCzkpxvjcAq9vTMvXVcWn","type":"remote-candidate","candidateType":"prflx","protocol":"udp"}],"remoteDER_SHA256":["97a0a36c1cba12f2751396d81ca02519de1baf1e090cfb863d73a4da849f9498"]},"browser":{"pair":{"state":"in-progress","nominated":true,"local":"host","remote":"host","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"97a0a36c1cba12f2751396d81ca02519de1baf1e090cfb863d73a4da849f9498","remoteFingerprint":"6a8fa85e36344a706be82d224623b315292e4bf4663a100f3ecd6c8d61b9e5b0","statsRemoteFingerprint":"6a8fa85e36344a706be82d224623b315292e4bf4663a100f3ecd6c8d61b9e5b0","remoteAddressesRedacted":false,"peerReflexiveInRemoteList":false,"pairKeys":{"local":"IykTxN2nh","remote":"I04wLIASk"},"candidateKeys":[{"id":"I04wLIASk","type":"remote-candidate","candidateType":"host","protocol":"udp"},{"id":"IHpiwFWvy","type":"local-candidate","candidateType":"host","protocol":"udp"},{"id":"IIhe5aFNg","type":"local-candidate","candidateType":"host","protocol":"tcp"},{"id":"IykTxN2nh","type":"local-candidate","candidateType":"host","protocol":"udp"}],"remoteDER_SHA256":["6a8fa85e36344a706be82d224623b315292e4bf4663a100f3ecd6c8d61b9e5b0"]}},"trace":[]}],"failure":{"phase":"negotiation and transfer","case":{"browser":"chrome","engine":"boa","route":"direct","offerer":"native","cleanup":"close"},"error":"application exchange deadline"},"relayFixture":"bounded Windows UDP front end forwarding unchanged datagrams to WSL coturn; distinct backend socket per client; max 8 clients/256 pending datagrams/1 MiB","profileCache":"C:\\t\\cargo-targets\\genet\\rtc-interop","profileCleanup":"automatic approval review blocked recursive deletion; marked profiles retained for reuse","scope":"native Windows to installed headed browsers, controlled WSL UDP server; no DNS/TCP/TLS/restart/media/carrier acceptance"}
```


## E2h automated browser interoperability - 2026-11-02T20:14:54.498Z

```json
{"schema":1,"gate":"E2h","utc":"2026-11-02T20:14:54.498Z","source":"d644fa0a013364a3d3437b30e98ef3da08dccbb2","node":"v24.11.0","command":["node","support/ci/webrtc_interop.mjs","--server","172.28.40.199:34790","--offerer","native","--cleanup","close"],"expectedCases":8,"completeMatrix":false,"nativeBinarySHA256":"818f9a3e181526e4eddb91c87d3dd8bd30c18c0d845e235e347676d941d2c475","sourceSHA256":{"components/genet-webrtc/examples/interop_host.rs":"821b677b6bf201647cca780409886e0ef46b2cf6c52eaec4baab36d6febe394b","support/ci/webrtc_interop.mjs":"d9e4512aab42d7f9ac5b05896879c3ba034aadc1d3fd7ab0e4063c8af5f30c46","support/ci/webrtc_interop_peer.js":"a74a1d9b0d42a096f7353f6f6abe11cae7a65a3defbea4579d0001ebf4c4f5b6","components/genet-webrtc/Cargo.toml":"a0c7b33c3e49b10ba972f67c1169c674046cb6c5e717caa9f6ed2d1c0e3a4265","components/genet-webrtc/src/lib.rs":"c91da3fea1701e320035a78e2f3d0f52edf89925d7ca25e6a2241828944ff279","components/script-runtime-api/webrtc/bootstrap.js":"b16a9c2cc771628ab6800492789278dec6fccc524c8308e2d03272f3b77c1bdd"},"versions":[{"browser":"chrome","version":"Chrome/156.0.8078.74","headed":true,"visibleWindows":1,"webrtcPreferenceOverrides":false,"networkPermissions":[]},{"browser":"firefox","version":"159.0","headed":true,"visibleWindows":1,"webrtcPreferenceOverrides":false,"networkPermissions":[{"permission":"local-network","before":"prompt","after":"granted","error":null,"reset":"prompt"},{"permission":"loopback-network","before":"prompt","after":"granted","error":null,"reset":"prompt"}]}],"cases":[{"browser":"chrome","engine":"boa","route":"direct","offerer":"native","cleanup":"close","status":"pass","seconds":23.42,"native":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"unreliable","ordered":false,"maxRetransmits":0},{"label":"reliable","ordered":true,"maxRetransmits":null}],"refused":["mdns:OperationError","mdns:OperationError"],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"host","remote":"prflx","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"ae56ca60f151c9b0c9217523bee91342acee6f9a7913b37f6142aa4d8e585d5b","remoteFingerprint":"4a6519497f2e6114accd0c5d10a854bdbe09c6f79e7d0395bb6ea9c0d2979f04","remoteDER_SHA256":["4a6519497f2e6114accd0c5d10a854bdbe09c6f79e7d0395bb6ea9c0d2979f04"],"statsRemoteFingerprint":"4a6519497f2e6114accd0c5d10a854bdbe09c6f79e7d0395bb6ea9c0d2979f04","remoteAddressesRedacted":true,"peerReflexiveInRemoteList":false},"browserPeer":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"unreliable","ordered":false,"maxRetransmits":0},{"label":"reliable","ordered":true,"maxRetransmits":null}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"host","remote":"host","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"4a6519497f2e6114accd0c5d10a854bdbe09c6f79e7d0395bb6ea9c0d2979f04","remoteFingerprint":"ae56ca60f151c9b0c9217523bee91342acee6f9a7913b37f6142aa4d8e585d5b","remoteDER_SHA256":["ae56ca60f151c9b0c9217523bee91342acee6f9a7913b37f6142aa4d8e585d5b"],"statsRemoteFingerprint":"ae56ca60f151c9b0c9217523bee91342acee6f9a7913b37f6142aa4d8e585d5b","remoteAddressesRedacted":false,"peerReflexiveInRemoteList":false},"cleanupVerified":true,"socketRebound":true},{"browser":"chrome","engine":"boa","route":"relay","offerer":"native","cleanup":"close","status":"pass","seconds":15.82,"native":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"unreliable","ordered":false,"maxRetransmits":0},{"label":"reliable","ordered":true,"maxRetransmits":null}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"relay","remote":"relay","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"b66f27476d26970986e54c50aeb5fe41a8a36b0cbe01e3c21c918c580e6eba4f","remoteFingerprint":"f1de5f1fef51ffc2648ee42f482e46e617e1ce98dd724d4bfcd8ee2ee9ffa6a7","remoteDER_SHA256":["f1de5f1fef51ffc2648ee42f482e46e617e1ce98dd724d4bfcd8ee2ee9ffa6a7"],"statsRemoteFingerprint":"f1de5f1fef51ffc2648ee42f482e46e617e1ce98dd724d4bfcd8ee2ee9ffa6a7","remoteAddressesRedacted":true,"peerReflexiveInRemoteList":false},"browserPeer":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"unreliable","ordered":false,"maxRetransmits":0},{"label":"reliable","ordered":true,"maxRetransmits":null}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"relay","remote":"relay","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"f1de5f1fef51ffc2648ee42f482e46e617e1ce98dd724d4bfcd8ee2ee9ffa6a7","remoteFingerprint":"b66f27476d26970986e54c50aeb5fe41a8a36b0cbe01e3c21c918c580e6eba4f","remoteDER_SHA256":["b66f27476d26970986e54c50aeb5fe41a8a36b0cbe01e3c21c918c580e6eba4f"],"statsRemoteFingerprint":"b66f27476d26970986e54c50aeb5fe41a8a36b0cbe01e3c21c918c580e6eba4f","remoteAddressesRedacted":false,"peerReflexiveInRemoteList":false},"cleanupVerified":true,"socketRebound":true},{"browser":"chrome","engine":"nova","route":"direct","offerer":"native","cleanup":"close","status":"pass","seconds":18.28,"native":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":["mdns:OperationError","mdns:OperationError"],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"host","remote":"prflx","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"e19704148a1e5bd9aa3e189505fe1728281bd41ae3f5e4f661569bffa5045369","remoteFingerprint":"8e41b4da25336210f5572a01e8fac76346277f0eee36df26b685fa3186d25bce","remoteDER_SHA256":["8e41b4da25336210f5572a01e8fac76346277f0eee36df26b685fa3186d25bce"],"statsRemoteFingerprint":"8e41b4da25336210f5572a01e8fac76346277f0eee36df26b685fa3186d25bce","remoteAddressesRedacted":true,"peerReflexiveInRemoteList":false},"browserPeer":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"host","remote":"host","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"8e41b4da25336210f5572a01e8fac76346277f0eee36df26b685fa3186d25bce","remoteFingerprint":"e19704148a1e5bd9aa3e189505fe1728281bd41ae3f5e4f661569bffa5045369","remoteDER_SHA256":["e19704148a1e5bd9aa3e189505fe1728281bd41ae3f5e4f661569bffa5045369"],"statsRemoteFingerprint":"e19704148a1e5bd9aa3e189505fe1728281bd41ae3f5e4f661569bffa5045369","remoteAddressesRedacted":false,"peerReflexiveInRemoteList":false},"cleanupVerified":true,"socketRebound":true},{"browser":"chrome","engine":"nova","route":"relay","offerer":"native","cleanup":"close","status":"pass","seconds":13.06,"native":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"unreliable","ordered":false,"maxRetransmits":0},{"label":"reliable","ordered":true,"maxRetransmits":null}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"relay","remote":"relay","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"0ae6d941c695b1f3b321933e906e83cc02cb38eb09f315c747d049e8ab73d70b","remoteFingerprint":"6949893de553ee9ced3bcc51dc026027c78c5d5f6d10396565375f16e5b4f3d8","remoteDER_SHA256":["6949893de553ee9ced3bcc51dc026027c78c5d5f6d10396565375f16e5b4f3d8"],"statsRemoteFingerprint":"6949893de553ee9ced3bcc51dc026027c78c5d5f6d10396565375f16e5b4f3d8","remoteAddressesRedacted":true,"peerReflexiveInRemoteList":false},"browserPeer":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"unreliable","ordered":false,"maxRetransmits":0},{"label":"reliable","ordered":true,"maxRetransmits":null}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"relay","remote":"relay","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"6949893de553ee9ced3bcc51dc026027c78c5d5f6d10396565375f16e5b4f3d8","remoteFingerprint":"0ae6d941c695b1f3b321933e906e83cc02cb38eb09f315c747d049e8ab73d70b","remoteDER_SHA256":["0ae6d941c695b1f3b321933e906e83cc02cb38eb09f315c747d049e8ab73d70b"],"statsRemoteFingerprint":"0ae6d941c695b1f3b321933e906e83cc02cb38eb09f315c747d049e8ab73d70b","remoteAddressesRedacted":false,"peerReflexiveInRemoteList":false},"cleanupVerified":true,"socketRebound":true},{"browser":"firefox","engine":"boa","route":"direct","offerer":"native","cleanup":"close","status":"pass","seconds":13.73,"native":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":["mdns:OperationError","mdns:OperationError","mdns:OperationError","mdns:OperationError"],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"host","remote":"prflx","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"f5ece0d6028810bd9e85ba05c66bba298b8a509be81b913d20dc81fca64b762f","remoteFingerprint":"ab1ddc4a148684c9c4498ffe1005103c20276da558c45cc019d1b533c5f1a2b2","remoteDER_SHA256":["ab1ddc4a148684c9c4498ffe1005103c20276da558c45cc019d1b533c5f1a2b2"],"statsRemoteFingerprint":"ab1ddc4a148684c9c4498ffe1005103c20276da558c45cc019d1b533c5f1a2b2","remoteAddressesRedacted":true,"peerReflexiveInRemoteList":false},"browserPeer":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"host","remote":"host","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"ab1ddc4a148684c9c4498ffe1005103c20276da558c45cc019d1b533c5f1a2b2","remoteFingerprint":"f5ece0d6028810bd9e85ba05c66bba298b8a509be81b913d20dc81fca64b762f","remoteDER_SHA256":["f5ece0d6028810bd9e85ba05c66bba298b8a509be81b913d20dc81fca64b762f"],"statsRemoteFingerprint":"f5ece0d6028810bd9e85ba05c66bba298b8a509be81b913d20dc81fca64b762f","remoteAddressesRedacted":false,"peerReflexiveInRemoteList":null},"cleanupVerified":true,"socketRebound":true},{"browser":"firefox","engine":"boa","route":"relay","offerer":"native","cleanup":"close","status":"pass","seconds":14.51,"native":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"relay","remote":"relay","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"6330ed080f37489eeb968504a8dc0ef81fee98586cba42ca87b7272cfec6852c","remoteFingerprint":"d1f53d402b2778ad02cceab91f10bff02fc52f516430dbecc3f2c211e36790b7","remoteDER_SHA256":["d1f53d402b2778ad02cceab91f10bff02fc52f516430dbecc3f2c211e36790b7"],"statsRemoteFingerprint":"d1f53d402b2778ad02cceab91f10bff02fc52f516430dbecc3f2c211e36790b7","remoteAddressesRedacted":true,"peerReflexiveInRemoteList":false},"browserPeer":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"relay","remote":"relay","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"d1f53d402b2778ad02cceab91f10bff02fc52f516430dbecc3f2c211e36790b7","remoteFingerprint":"6330ed080f37489eeb968504a8dc0ef81fee98586cba42ca87b7272cfec6852c","remoteDER_SHA256":["6330ed080f37489eeb968504a8dc0ef81fee98586cba42ca87b7272cfec6852c"],"statsRemoteFingerprint":"6330ed080f37489eeb968504a8dc0ef81fee98586cba42ca87b7272cfec6852c","remoteAddressesRedacted":false,"peerReflexiveInRemoteList":null},"cleanupVerified":true,"socketRebound":true},{"browser":"firefox","engine":"nova","route":"direct","offerer":"native","cleanup":"close","status":"pass","seconds":10.44,"native":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":["mdns:OperationError","mdns:OperationError","mdns:OperationError","mdns:OperationError"],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"host","remote":"prflx","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"70de621ac27e02b6b7ce105d8b01b46e2c2e724a3dac992eeb8d21155666dcdb","remoteFingerprint":"bfacfa5bf4bed0ed92ca04c0aeb5be6c9b8f3a0ed0c2a3a383e2229a56a9b558","remoteDER_SHA256":["bfacfa5bf4bed0ed92ca04c0aeb5be6c9b8f3a0ed0c2a3a383e2229a56a9b558"],"statsRemoteFingerprint":"bfacfa5bf4bed0ed92ca04c0aeb5be6c9b8f3a0ed0c2a3a383e2229a56a9b558","remoteAddressesRedacted":true,"peerReflexiveInRemoteList":false},"browserPeer":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"host","remote":"host","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"bfacfa5bf4bed0ed92ca04c0aeb5be6c9b8f3a0ed0c2a3a383e2229a56a9b558","remoteFingerprint":"70de621ac27e02b6b7ce105d8b01b46e2c2e724a3dac992eeb8d21155666dcdb","remoteDER_SHA256":["70de621ac27e02b6b7ce105d8b01b46e2c2e724a3dac992eeb8d21155666dcdb"],"statsRemoteFingerprint":"70de621ac27e02b6b7ce105d8b01b46e2c2e724a3dac992eeb8d21155666dcdb","remoteAddressesRedacted":false,"peerReflexiveInRemoteList":null},"cleanupVerified":true,"socketRebound":true},{"browser":"firefox","engine":"nova","route":"relay","offerer":"native","cleanup":"close","status":"pass","seconds":11.31,"native":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"relay","remote":"relay","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"67c0c320f67f8782647f9f207f6bde2a0e56fbb34fe69965e08b2ddd3e201e1e","remoteFingerprint":"6a33d48b8a8fe86764aa7fa6c41ab4b501099b29e1d593a22e0812a98626b6f8","remoteDER_SHA256":["6a33d48b8a8fe86764aa7fa6c41ab4b501099b29e1d593a22e0812a98626b6f8"],"statsRemoteFingerprint":"6a33d48b8a8fe86764aa7fa6c41ab4b501099b29e1d593a22e0812a98626b6f8","remoteAddressesRedacted":true,"peerReflexiveInRemoteList":false},"browserPeer":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"reliable","ordered":true,"maxRetransmits":null},{"label":"unreliable","ordered":false,"maxRetransmits":0}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"relay","remote":"relay","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"6a33d48b8a8fe86764aa7fa6c41ab4b501099b29e1d593a22e0812a98626b6f8","remoteFingerprint":"67c0c320f67f8782647f9f207f6bde2a0e56fbb34fe69965e08b2ddd3e201e1e","remoteDER_SHA256":["67c0c320f67f8782647f9f207f6bde2a0e56fbb34fe69965e08b2ddd3e201e1e"],"statsRemoteFingerprint":"67c0c320f67f8782647f9f207f6bde2a0e56fbb34fe69965e08b2ddd3e201e1e","remoteAddressesRedacted":false,"peerReflexiveInRemoteList":null},"cleanupVerified":true,"socketRebound":true}],"failure":null,"relayFixture":"bounded Windows UDP front end forwarding unchanged datagrams to WSL coturn; distinct backend socket per client; max 8 clients/256 pending datagrams/1 MiB","profileCache":"C:\\t\\cargo-targets\\genet\\rtc-interop","profileCleanup":"automatic approval review blocked recursive deletion; marked profiles retained for reuse","scope":"native Windows to installed headed browsers, controlled WSL UDP server; no DNS/TCP/TLS/restart/media/carrier acceptance"}
```


## E2h automated browser interoperability - 2026-11-02T20:17:02.614Z

```json
{"schema":1,"gate":"E2h","utc":"2026-11-02T20:17:02.614Z","source":"73a2f0ae4be1e9da10b3acc410d910aa355559cb","node":"v24.11.0","command":["node","support/ci/webrtc_interop.mjs","--server","172.28.40.199:34790","--browser","chrome","--engine","boa","--route","direct","--offerer","native","--cleanup","close"],"expectedCases":1,"completeMatrix":false,"nativeBinarySHA256":"5cc71b425cf5a14bfc6f983c0e683b88d7c2d9ef185482d23864d5092e671406","sourceSHA256":{"components/genet-webrtc/examples/interop_host.rs":"821b677b6bf201647cca780409886e0ef46b2cf6c52eaec4baab36d6febe394b","support/ci/webrtc_interop.mjs":"d9d5d004eb0aad1b93a12481476a710cdb73ca7679333ccdfbd5317443cca183","support/ci/webrtc_interop_peer.js":"a74a1d9b0d42a096f7353f6f6abe11cae7a65a3defbea4579d0001ebf4c4f5b6","components/genet-webrtc/Cargo.toml":"a0c7b33c3e49b10ba972f67c1169c674046cb6c5e717caa9f6ed2d1c0e3a4265","components/genet-webrtc/src/lib.rs":"c89452631672d4c12848182ed48f3cb6c9b01951ca77efb689b1952d1adea5bf","components/script-runtime-api/webrtc/bootstrap.js":"b16a9c2cc771628ab6800492789278dec6fccc524c8308e2d03272f3b77c1bdd"},"versions":[{"browser":"chrome","version":"Chrome/156.0.8078.74","headed":true,"visibleWindows":1,"webrtcPreferenceOverrides":false,"networkPermissions":[]}],"cases":[{"browser":"chrome","engine":"boa","route":"direct","offerer":"native","cleanup":"close","status":"pass","seconds":15.51,"native":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"unreliable","ordered":false,"maxRetransmits":0},{"label":"reliable","ordered":true,"maxRetransmits":null}],"refused":["mdns:OperationError","mdns:OperationError"],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"host","remote":"prflx","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"f1a99efc4c58a8cbc1245f4cace5db03bc6dd4993590dcdf9808a746e281a983","remoteFingerprint":"51284c9f4329c83bad2daa7f99b5eeca38caf6c67ebcfb6ecbef885c6b8c9f9e","remoteDER_SHA256":["51284c9f4329c83bad2daa7f99b5eeca38caf6c67ebcfb6ecbef885c6b8c9f9e"],"statsRemoteFingerprint":"51284c9f4329c83bad2daa7f99b5eeca38caf6c67ebcfb6ecbef885c6b8c9f9e","remoteAddressesRedacted":true,"peerReflexiveInRemoteList":false},"browserPeer":{"connected":true,"textMessagesVerified":2,"binaryBytesVerified":48000,"modes":[{"label":"unreliable","ordered":false,"maxRetransmits":0},{"label":"reliable","ordered":true,"maxRetransmits":null}],"refused":[],"gatheringErrors":[],"pair":{"state":"succeeded","nominated":true,"local":"host","remote":"host","localProtocol":"udp","remoteProtocol":"udp"},"localFingerprint":"51284c9f4329c83bad2daa7f99b5eeca38caf6c67ebcfb6ecbef885c6b8c9f9e","remoteFingerprint":"f1a99efc4c58a8cbc1245f4cace5db03bc6dd4993590dcdf9808a746e281a983","remoteDER_SHA256":["f1a99efc4c58a8cbc1245f4cace5db03bc6dd4993590dcdf9808a746e281a983"],"statsRemoteFingerprint":"f1a99efc4c58a8cbc1245f4cace5db03bc6dd4993590dcdf9808a746e281a983","remoteAddressesRedacted":false,"peerReflexiveInRemoteList":false},"cleanupVerified":true,"socketRebound":true}],"failure":null,"relayFixture":"bounded Windows UDP front end forwarding unchanged datagrams to WSL coturn; distinct backend socket per client; max 8 clients/256 pending datagrams/1 MiB","profileCache":"C:\\t\\cargo-targets\\genet\\rtc-interop","profileCleanup":"automatic approval review blocked recursive deletion; marked profiles retained for reuse","scope":"native Windows to installed headed browsers, controlled WSL UDP server; no DNS/TCP/TLS/restart/media/carrier acceptance"}
```


## 2026-11-02: E2h final receiving checks

```json
{"gate":"E2h-final-checks","source":"73a2f0ae4be1e9da10b3acc410d910aa355559cb","utc":"2026-11-02T20:15:38.389695+00:00","environment":{"CARGO_TARGET_DIR":"C:\\t\\cargo-targets\\genet","CARGO_BUILD_BUILD_DIR":"C:\\t\\cargo-targets\\genet\\build","CARGO_PROFILE_DEV_DEBUG":"0"},"rustc":"rustc 1.99.1 (58cf8f9c3 2026-10-13)","lockSHA256Before":"0a42b50cb56df741ad34acc18be60070a2eb3244a3a50d64260e7e67e9754f9f","checks":[{"name":"ordinary native","command":["cargo","test","-p","genet-webrtc","--test","browser","--locked","--offline","-j","1"],"exit":0,"seconds":27.08,"summary":["    Finished `test` profile [unoptimized] target(s) in 8.81s","test result: ok. 16 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 17.03s"]},{"name":"controlled native TURN","command":["cargo","test","-p","genet-webrtc","--test","browser","--locked","--offline","-j","1","--","--ignored","--test-threads=1"],"exit":0,"seconds":25.55,"summary":["    Finished `test` profile [unoptimized] target(s) in 3.12s","test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 16 filtered out; finished in 22.25s"]},{"name":"strict native library and fixture Clippy","command":["cargo","clippy","-p","genet-webrtc","--lib","--example","interop_host","--no-deps","--locked","--offline","-j","1","--","-D","warnings"],"exit":0,"seconds":4.52,"summary":["    Finished `dev` profile [unoptimized] target(s) in 4.22s"]},{"name":"native fixture build","command":["cargo","build","-p","genet-webrtc","--example","interop_host","--locked","--offline","-j","1"],"exit":0,"seconds":8.86,"summary":["    Finished `dev` profile [unoptimized] target(s) in 8.71s"]},{"name":"cold startup deadline smoke","command":["node","support/ci/webrtc_interop.mjs","--server","172.28.40.199:34790","--browser","chrome","--engine","boa","--route","direct","--offerer","native","--cleanup","close"],"exit":0,"seconds":18.27,"summary":["E2h 1/1 passed; compact receipt appended"]},{"name":"WPT receiving runner build","command":["cargo","build","-p","genet-wpt","--no-default-features","--features","webrtc","--locked","--offline","-j","1"],"exit":0,"seconds":65.89,"summary":["    Finished `dev` profile [unoptimized] target(s) in 1m 05s"]},{"name":"Ortet scripted Vano all targets","command":["cargo","check","-p","ortet","--features","scripted-nova","--all-targets","--locked","--offline","-j","1"],"exit":0,"seconds":16.81,"summary":["    Finished `dev` profile [unoptimized] target(s) in 16.59s"]}],"lockSHA256After":"0a42b50cb56df741ad34acc18be60070a2eb3244a3a50d64260e7e67e9754f9f","nativeBinarySHA256":"5cc71b425cf5a14bfc6f983c0e683b88d7c2d9ef185482d23864d5092e671406","wptBinarySHA256":"2ac84674fa2775992efe75059bd522f73ee478b6ed25ba8c4396906dd42dee5b"}
```


## 2026-11-02: final selected WPT identities

```json
{"gate":"E2h-selected-WPT","utc":"2026-11-02T20:18:51.068127+00:00","source":"73a2f0ae4be1e9da10b3acc410d910aa355559cb","binarySHA256":"2ac84674fa2775992efe75059bd522f73ee478b6ed25ba8c4396906dd42dee5b","lockSHA256":"0a42b50cb56df741ad34acc18be60070a2eb3244a3a50d64260e7e67e9754f9f","historicalComparisonSource":"835dca0f37cf7efbd6d791f94752a2e40733c2fa","historicalComparisonBinarySHA256":"246f0e49876fd7669bba40c5dc2c5663e3c82d0bc7b27f780066893f5dff97b1","outcomes":[{"detail":null,"failures":[],"reason":null,"status":"pass","subtests":[{"name":"RTCPeerConnection.length","status":"pass"},{"name":"new RTCPeerConnection()","status":"pass"},{"name":"new RTCPeerConnection(null)","status":"pass"},{"name":"new RTCPeerConnection(undefined)","status":"pass"},{"name":"new RTCPeerConnection({})","status":"pass"},{"name":"new RTCPeerConnection({ certificates: null })","status":"pass"},{"name":"new RTCPeerConnection({ certificates: undefined })","status":"pass"},{"name":"new RTCPeerConnection({ certificates: [] })","status":"pass"},{"name":"new RTCPeerConnection({ certificates: [null] })","status":"pass"},{"name":"new RTCPeerConnection({ certificates: [undefined] })","status":"pass"},{"name":"new RTCPeerConnection({ iceCandidatePoolSize: toNumberThrows })","status":"pass"},{"name":"localDescription initial value","status":"pass"},{"name":"currentLocalDescription initial value","status":"pass"},{"name":"pendingLocalDescription initial value","status":"pass"},{"name":"remoteDescription initial value","status":"pass"},{"name":"currentRemoteDescription initial value","status":"pass"},{"name":"pendingRemoteDescription initial value","status":"pass"},{"name":"signalingState initial value","status":"pass"},{"name":"iceGatheringState initial value","status":"pass"},{"name":"iceConnectionState initial value","status":"pass"},{"name":"connectionState initial value","status":"pass"},{"name":"canTrickleIceCandidates initial value","status":"pass"}],"subtests_passed":22,"subtests_total":22,"engine":"boa","test":"webrtc/RTCPeerConnection-constructor.html","exit_code":0,"command":["C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe","testharness-one","webrtc/RTCPeerConnection-constructor.html","--test-path","C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCPeerConnection-constructor.html","--tests-root","C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests","--engine","boa","--drive-deadline","5"],"seconds":7.98},{"detail":null,"failures":[],"reason":null,"status":"pass","subtests":[{"name":"new RTCIceCandidate()","status":"pass"},{"name":"new RTCIceCandidate({})","status":"pass"},{"name":"new RTCIceCandidate({ ... }) with manually filled default values","status":"pass"},{"name":"new RTCIceCandidate({ sdpMid: null, sdpMLineIndex: null })","status":"pass"},{"name":"new RTCIceCandidate({ candidate: '' })","status":"pass"},{"name":"new RTCIceCandidate({ candidate: null })","status":"pass"},{"name":"new RTCIceCandidate({ ... }) with valid candidate string only","status":"pass"},{"name":"new RTCIceCandidate({ sdpMid: 'audio' })","status":"pass"},{"name":"new RTCIceCandidate({ sdpMLineIndex: 0 })","status":"pass"},{"name":"new RTCIceCandidate({ sdpMid: 'audio', sdpMLineIndex: 0 })","status":"pass"},{"name":"new RTCIceCandidate({ candidate: '', sdpMid: 'audio' }","status":"pass"},{"name":"new RTCIceCandidate({ candidate: null, sdpMLineIndex: 0 }","status":"pass"},{"name":"new RTCIceCandidate({ ... }) with valid candidate string and sdpMid","status":"pass"},{"name":"new RTCIceCandidate({ ... }) with invalid candidate string and sdpMid","status":"pass"},{"name":"new RTCIceCandidate({ ... }) with nondefault values for most fields","status":"pass"},{"name":"new RTCIceCandidate({ ... }) with nondefault values for most fields, tcp candidate","status":"pass"},{"name":"new RTCIceCandidate({ ... }) with nondefault values for all fields","status":"pass"},{"name":"new RTCIceCandidate({ ... }) with nondefault values for all fields, tcp candidate","status":"pass"},{"name":"new RTCIceCandidate({ relayProtocol, url }) cloned vs signaled","status":"pass"},{"name":"new RTCIceCandidate({ ... }) with invalid sdpMid","status":"pass"},{"name":"new RTCIceCandidate({ ... }) with invalid sdpMLineIndex","status":"pass"}],"subtests_passed":21,"subtests_total":21,"engine":"boa","test":"webrtc/RTCIceCandidate-constructor.html","exit_code":0,"command":["C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe","testharness-one","webrtc/RTCIceCandidate-constructor.html","--test-path","C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCIceCandidate-constructor.html","--tests-root","C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests","--engine","boa","--drive-deadline","5"],"seconds":7.14},{"detail":null,"failures":["        [1] addTrack, then creating datachannel, should negotiate properly promise_test: Unhandled rejection with value: object \"TypeError: cannot convert 'null' or 'undefined' to object\"","        [1] addTrack, then creating datachannel, should negotiate properly when max-bundle is used promise_test: Unhandled rejection with value: object \"TypeError: cannot convert 'null' or 'undefined' to object\"","        [1] Renegotiation with audio/video and datachannel should work properly promise_test: Unhandled rejection with value: object \"TypeError: cannot convert 'null' or 'undefined' to object\"","        [1] addTrack, then creating negotiated datachannel, should negotiate properly promise_test: Unhandled rejection with value: object \"TypeError: cannot convert 'null' or 'undefined' to object\"","        [1] addTrack, then creating negotiated datachannel, should negotiate properly when max-bundle is used promise_test: Unhandled rejection with value: object \"TypeError: cannot convert 'null' or 'undefined' to object\"","        [1] Renegotiation with audio/video and negotiated datachannel should work properly promise_test: Unhandled rejection with value: object \"TypeError: cannot convert 'null' or 'undefined' to object\""],"reason":null,"status":"fail","subtests":[{"name":"createDataChannel with no argument should throw TypeError","status":"pass"},{"name":"createDataChannel with closed connection should throw InvalidStateError","status":"pass"},{"name":"createDataChannel attribute default values","status":"pass"},{"name":"createDataChannel with provided parameters should initialize attributes to provided values","status":"pass"},{"name":"createDataChannel with label \"foo\" should succeed","status":"pass"},{"name":"createDataChannel with label null should succeed","status":"pass"},{"name":"createDataChannel with label undefined should succeed","status":"pass"},{"name":"createDataChannel with label lone surrogate should succeed","status":"pass"},{"name":"createDataChannel with ordered false should succeed","status":"pass"},{"name":"createDataChannel with ordered null/undefined should succeed","status":"pass"},{"name":"createDataChannel with maxPacketLifeTime 0 should succeed","status":"pass"},{"name":"createDataChannel with maxRetransmits 0 should succeed","status":"pass"},{"name":"createDataChannel with both maxPacketLifeTime and maxRetransmits undefined should succeed","status":"pass"},{"name":"createDataChannel with both maxPacketLifeTime and maxRetransmits should throw TypeError","status":"pass"},{"name":"createDataChannel with protocol \"foo\" should succeed","status":"pass"},{"name":"createDataChannel with protocol null should succeed","status":"pass"},{"name":"createDataChannel with protocol undefined should succeed","status":"pass"},{"name":"createDataChannel with protocol lone surrogate should succeed","status":"pass"},{"name":"createDataChannel with id 0 and negotiated not set should succeed, but not set the channel's id","status":"pass"},{"name":"createDataChannel with id 1 and negotiated not set should succeed, but not set the channel's id","status":"pass"},{"name":"createDataChannel with id 65534 and negotiated not set should succeed, but not set the channel's id","status":"pass"},{"name":"createDataChannel with id 65535 and negotiated not set should succeed, but not set the channel's id","status":"pass"},{"name":"createDataChannel with id 0 and negotiated true should succeed, and set the channel's id","status":"pass"},{"name":"createDataChannel with id 1 and negotiated true should succeed, and set the channel's id","status":"pass"},{"name":"createDataChannel with id 65534 and negotiated true should succeed, and set the channel's id","status":"pass"},{"name":"createDataChannel with id -1 and negotiated not set should throw TypeError","status":"pass"},{"name":"createDataChannel with id 65536 and negotiated not set should throw TypeError","status":"pass"},{"name":"createDataChannel with id -1 should throw TypeError","status":"pass"},{"name":"createDataChannel with id 65535 should throw TypeError","status":"pass"},{"name":"createDataChannel with id 65536 should throw TypeError","status":"pass"},{"name":"createDataChannel with too long label should throw TypeError","status":"pass"},{"name":"createDataChannel with too long label (2 byte unicode) should throw TypeError","status":"pass"},{"name":"createDataChannel with same label used twice should not throw","status":"pass"},{"name":"createDataChannel with negotiated true and id should succeed","status":"pass"},{"name":"createDataChannel with too long protocol should throw TypeError","status":"pass"},{"name":"createDataChannel with too long protocol (2 byte unicode) should throw TypeError","status":"pass"},{"name":"createDataChannel with maximum length label and protocol should succeed","status":"pass"},{"name":"createDataChannel with negotiated false should succeed","status":"pass"},{"name":"createDataChannel with negotiated false and id 42 should ignore the id","status":"pass"},{"name":"createDataChannel with negotiated true and id not defined should throw TypeError","status":"pass"},{"name":"Channels created (after SCTP connected) should have id assigned","status":"pass"},{"name":"Reusing a data channel id that is in use should throw OperationError","status":"pass"},{"name":"Reusing a data channel id that is in use (after setRemoteDescription) should throw OperationError","status":"pass"},{"name":"Reusing a data channel id that is in use (after setRemoteDescription, negotiated via DCEP) should throw OperationError","status":"pass"},{"name":"New datachannel should be in the connecting state after creation (after connection establishment)","status":"pass"},{"name":"addTrack, then creating datachannel, should negotiate properly","status":"fail"},{"name":"addTrack, then creating datachannel, should negotiate properly when max-bundle is used","status":"fail"},{"name":"Renegotiation with audio/video and datachannel should work properly","status":"fail"},{"name":"New negotiated datachannel should be in the connecting state after creation (after connection establishment)","status":"pass"},{"name":"addTrack, then creating negotiated datachannel, should negotiate properly","status":"fail"},{"name":"addTrack, then creating negotiated datachannel, should negotiate properly when max-bundle is used","status":"fail"},{"name":"Renegotiation with audio/video and negotiated datachannel should work properly","status":"fail"}],"subtests_passed":46,"subtests_total":52,"engine":"boa","test":"webrtc/RTCPeerConnection-createDataChannel.html","exit_code":0,"command":["C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe","testharness-one","webrtc/RTCPeerConnection-createDataChannel.html","--test-path","C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCPeerConnection-createDataChannel.html","--tests-root","C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests","--engine","boa","--drive-deadline","5"],"seconds":9.91},{"detail":null,"failures":[],"reason":null,"status":"pass","subtests":[{"name":"RTCPeerConnectionIceErrorEvent constructed from init parameters","status":"pass"}],"subtests_passed":1,"subtests_total":1,"engine":"boa","test":"webrtc/RTCPeerConnectionIceErrorEvent.html","exit_code":0,"command":["C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe","testharness-one","webrtc/RTCPeerConnectionIceErrorEvent.html","--test-path","C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCPeerConnectionIceErrorEvent.html","--tests-root","C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests","--engine","boa","--drive-deadline","5"],"seconds":7.49},{"detail":null,"failures":[],"reason":null,"status":"pass","subtests":[{"name":"RTCPeerConnection.length","status":"pass"},{"name":"new RTCPeerConnection()","status":"pass"},{"name":"new RTCPeerConnection(null)","status":"pass"},{"name":"new RTCPeerConnection(undefined)","status":"pass"},{"name":"new RTCPeerConnection({})","status":"pass"},{"name":"new RTCPeerConnection({ certificates: null })","status":"pass"},{"name":"new RTCPeerConnection({ certificates: undefined })","status":"pass"},{"name":"new RTCPeerConnection({ certificates: [] })","status":"pass"},{"name":"new RTCPeerConnection({ certificates: [null] })","status":"pass"},{"name":"new RTCPeerConnection({ certificates: [undefined] })","status":"pass"},{"name":"new RTCPeerConnection({ iceCandidatePoolSize: toNumberThrows })","status":"pass"},{"name":"localDescription initial value","status":"pass"},{"name":"currentLocalDescription initial value","status":"pass"},{"name":"pendingLocalDescription initial value","status":"pass"},{"name":"remoteDescription initial value","status":"pass"},{"name":"currentRemoteDescription initial value","status":"pass"},{"name":"pendingRemoteDescription initial value","status":"pass"},{"name":"signalingState initial value","status":"pass"},{"name":"iceGatheringState initial value","status":"pass"},{"name":"iceConnectionState initial value","status":"pass"},{"name":"connectionState initial value","status":"pass"},{"name":"canTrickleIceCandidates initial value","status":"pass"}],"subtests_passed":22,"subtests_total":22,"engine":"nova","test":"webrtc/RTCPeerConnection-constructor.html","exit_code":0,"command":["C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe","testharness-one","webrtc/RTCPeerConnection-constructor.html","--test-path","C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCPeerConnection-constructor.html","--tests-root","C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests","--engine","nova","--drive-deadline","5"],"seconds":3.39},{"detail":null,"failures":[],"reason":null,"status":"pass","subtests":[{"name":"new RTCIceCandidate()","status":"pass"},{"name":"new RTCIceCandidate({})","status":"pass"},{"name":"new RTCIceCandidate({ ... }) with manually filled default values","status":"pass"},{"name":"new RTCIceCandidate({ sdpMid: null, sdpMLineIndex: null })","status":"pass"},{"name":"new RTCIceCandidate({ candidate: '' })","status":"pass"},{"name":"new RTCIceCandidate({ candidate: null })","status":"pass"},{"name":"new RTCIceCandidate({ ... }) with valid candidate string only","status":"pass"},{"name":"new RTCIceCandidate({ sdpMid: 'audio' })","status":"pass"},{"name":"new RTCIceCandidate({ sdpMLineIndex: 0 })","status":"pass"},{"name":"new RTCIceCandidate({ sdpMid: 'audio', sdpMLineIndex: 0 })","status":"pass"},{"name":"new RTCIceCandidate({ candidate: '', sdpMid: 'audio' }","status":"pass"},{"name":"new RTCIceCandidate({ candidate: null, sdpMLineIndex: 0 }","status":"pass"},{"name":"new RTCIceCandidate({ ... }) with valid candidate string and sdpMid","status":"pass"},{"name":"new RTCIceCandidate({ ... }) with invalid candidate string and sdpMid","status":"pass"},{"name":"new RTCIceCandidate({ ... }) with nondefault values for most fields","status":"pass"},{"name":"new RTCIceCandidate({ ... }) with nondefault values for most fields, tcp candidate","status":"pass"},{"name":"new RTCIceCandidate({ ... }) with nondefault values for all fields","status":"pass"},{"name":"new RTCIceCandidate({ ... }) with nondefault values for all fields, tcp candidate","status":"pass"},{"name":"new RTCIceCandidate({ relayProtocol, url }) cloned vs signaled","status":"pass"},{"name":"new RTCIceCandidate({ ... }) with invalid sdpMid","status":"pass"},{"name":"new RTCIceCandidate({ ... }) with invalid sdpMLineIndex","status":"pass"}],"subtests_passed":21,"subtests_total":21,"engine":"nova","test":"webrtc/RTCIceCandidate-constructor.html","exit_code":0,"command":["C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe","testharness-one","webrtc/RTCIceCandidate-constructor.html","--test-path","C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCIceCandidate-constructor.html","--tests-root","C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests","--engine","nova","--drive-deadline","5"],"seconds":3.53},{"detail":null,"failures":["        [1] addTrack, then creating datachannel, should negotiate properly promise_test: Unhandled rejection with value: object \"TypeError: Cannot read property 'getUserMedia' of undefined.\"","        [1] addTrack, then creating datachannel, should negotiate properly when max-bundle is used promise_test: Unhandled rejection with value: object \"TypeError: Cannot read property 'getUserMedia' of undefined.\"","        [1] Renegotiation with audio/video and datachannel should work properly promise_test: Unhandled rejection with value: object \"TypeError: Cannot read property 'getUserMedia' of undefined.\"","        [1] addTrack, then creating negotiated datachannel, should negotiate properly promise_test: Unhandled rejection with value: object \"TypeError: Cannot read property 'getUserMedia' of undefined.\"","        [1] addTrack, then creating negotiated datachannel, should negotiate properly when max-bundle is used promise_test: Unhandled rejection with value: object \"TypeError: Cannot read property 'getUserMedia' of undefined.\"","        [1] Renegotiation with audio/video and negotiated datachannel should work properly promise_test: Unhandled rejection with value: object \"TypeError: Cannot read property 'getUserMedia' of undefined.\""],"reason":null,"status":"fail","subtests":[{"name":"createDataChannel with no argument should throw TypeError","status":"pass"},{"name":"createDataChannel with closed connection should throw InvalidStateError","status":"pass"},{"name":"createDataChannel attribute default values","status":"pass"},{"name":"createDataChannel with provided parameters should initialize attributes to provided values","status":"pass"},{"name":"createDataChannel with label \"foo\" should succeed","status":"pass"},{"name":"createDataChannel with label null should succeed","status":"pass"},{"name":"createDataChannel with label undefined should succeed","status":"pass"},{"name":"createDataChannel with label lone surrogate should succeed","status":"pass"},{"name":"createDataChannel with ordered false should succeed","status":"pass"},{"name":"createDataChannel with ordered null/undefined should succeed","status":"pass"},{"name":"createDataChannel with maxPacketLifeTime 0 should succeed","status":"pass"},{"name":"createDataChannel with maxRetransmits 0 should succeed","status":"pass"},{"name":"createDataChannel with both maxPacketLifeTime and maxRetransmits undefined should succeed","status":"pass"},{"name":"createDataChannel with both maxPacketLifeTime and maxRetransmits should throw TypeError","status":"pass"},{"name":"createDataChannel with protocol \"foo\" should succeed","status":"pass"},{"name":"createDataChannel with protocol null should succeed","status":"pass"},{"name":"createDataChannel with protocol undefined should succeed","status":"pass"},{"name":"createDataChannel with protocol lone surrogate should succeed","status":"pass"},{"name":"createDataChannel with id 0 and negotiated not set should succeed, but not set the channel's id","status":"pass"},{"name":"createDataChannel with id 1 and negotiated not set should succeed, but not set the channel's id","status":"pass"},{"name":"createDataChannel with id 65534 and negotiated not set should succeed, but not set the channel's id","status":"pass"},{"name":"createDataChannel with id 65535 and negotiated not set should succeed, but not set the channel's id","status":"pass"},{"name":"createDataChannel with id 0 and negotiated true should succeed, and set the channel's id","status":"pass"},{"name":"createDataChannel with id 1 and negotiated true should succeed, and set the channel's id","status":"pass"},{"name":"createDataChannel with id 65534 and negotiated true should succeed, and set the channel's id","status":"pass"},{"name":"createDataChannel with id -1 and negotiated not set should throw TypeError","status":"pass"},{"name":"createDataChannel with id 65536 and negotiated not set should throw TypeError","status":"pass"},{"name":"createDataChannel with id -1 should throw TypeError","status":"pass"},{"name":"createDataChannel with id 65535 should throw TypeError","status":"pass"},{"name":"createDataChannel with id 65536 should throw TypeError","status":"pass"},{"name":"createDataChannel with too long label should throw TypeError","status":"pass"},{"name":"createDataChannel with too long label (2 byte unicode) should throw TypeError","status":"pass"},{"name":"createDataChannel with same label used twice should not throw","status":"pass"},{"name":"createDataChannel with negotiated true and id should succeed","status":"pass"},{"name":"createDataChannel with too long protocol should throw TypeError","status":"pass"},{"name":"createDataChannel with too long protocol (2 byte unicode) should throw TypeError","status":"pass"},{"name":"createDataChannel with maximum length label and protocol should succeed","status":"pass"},{"name":"createDataChannel with negotiated false should succeed","status":"pass"},{"name":"createDataChannel with negotiated false and id 42 should ignore the id","status":"pass"},{"name":"createDataChannel with negotiated true and id not defined should throw TypeError","status":"pass"},{"name":"Channels created (after SCTP connected) should have id assigned","status":"pass"},{"name":"Reusing a data channel id that is in use should throw OperationError","status":"pass"},{"name":"Reusing a data channel id that is in use (after setRemoteDescription) should throw OperationError","status":"pass"},{"name":"Reusing a data channel id that is in use (after setRemoteDescription, negotiated via DCEP) should throw OperationError","status":"pass"},{"name":"New datachannel should be in the connecting state after creation (after connection establishment)","status":"pass"},{"name":"addTrack, then creating datachannel, should negotiate properly","status":"fail"},{"name":"addTrack, then creating datachannel, should negotiate properly when max-bundle is used","status":"fail"},{"name":"Renegotiation with audio/video and datachannel should work properly","status":"fail"},{"name":"New negotiated datachannel should be in the connecting state after creation (after connection establishment)","status":"pass"},{"name":"addTrack, then creating negotiated datachannel, should negotiate properly","status":"fail"},{"name":"addTrack, then creating negotiated datachannel, should negotiate properly when max-bundle is used","status":"fail"},{"name":"Renegotiation with audio/video and negotiated datachannel should work properly","status":"fail"}],"subtests_passed":46,"subtests_total":52,"engine":"nova","test":"webrtc/RTCPeerConnection-createDataChannel.html","exit_code":0,"command":["C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe","testharness-one","webrtc/RTCPeerConnection-createDataChannel.html","--test-path","C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCPeerConnection-createDataChannel.html","--tests-root","C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests","--engine","nova","--drive-deadline","5"],"seconds":6.08},{"detail":null,"failures":[],"reason":null,"status":"pass","subtests":[{"name":"RTCPeerConnectionIceErrorEvent constructed from init parameters","status":"pass"}],"subtests_passed":1,"subtests_total":1,"engine":"nova","test":"webrtc/RTCPeerConnectionIceErrorEvent.html","exit_code":0,"command":["C:\\t\\cargo-targets\\genet\\debug\\genet-wpt.exe","testharness-one","webrtc/RTCPeerConnectionIceErrorEvent.html","--test-path","C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests\\webrtc\\RTCPeerConnectionIceErrorEvent.html","--tests-root","C:\\Users\\mark_\\Code\\repos\\genet\\tests\\wpt\\tests","--engine","nova","--drive-deadline","5"],"seconds":2.94}],"matchedCases":[{"engine":"boa","test":"webrtc/RTCPeerConnection-constructor.html","beforeCases":22,"afterCases":22,"missing":[],"added":[],"changed":[]},{"engine":"boa","test":"webrtc/RTCIceCandidate-constructor.html","beforeCases":21,"afterCases":21,"missing":[],"added":[],"changed":[]},{"engine":"boa","test":"webrtc/RTCPeerConnection-createDataChannel.html","beforeCases":52,"afterCases":52,"missing":[],"added":[],"changed":[]},{"engine":"nova","test":"webrtc/RTCPeerConnection-constructor.html","beforeCases":22,"afterCases":22,"missing":[],"added":[],"changed":[]},{"engine":"nova","test":"webrtc/RTCIceCandidate-constructor.html","beforeCases":21,"afterCases":21,"missing":[],"added":[],"changed":[]},{"engine":"nova","test":"webrtc/RTCPeerConnection-createDataChannel.html","beforeCases":52,"afterCases":52,"missing":[],"added":[],"changed":[]}],"sourceSHA256":{"tests/wpt/tests/resources/testharness.js":"d2399236c2a09c429804ff2299ad6629e17e2b53f17a74dd341e936adb11ae3e","tests/wpt/tests/webrtc/RTCPeerConnection-helper.js":"9709abd9c5a024c8bdbedd1cfab8a8f2c47b9358a987905cedc2a770ded69d2d","tests/wpt/tests/webrtc/RTCPeerConnection-constructor.html":"31cf6a8f9f9d9c36d4a00a6ca1c224b493b6ee39b848dc4e380ce5b96988d074","tests/wpt/tests/webrtc/RTCIceCandidate-constructor.html":"9bab537b6f3618bda3410d37bba9d63bbdec6116ebe9958759e67fb9fe7d5449","tests/wpt/tests/webrtc/RTCPeerConnection-createDataChannel.html":"0356665f81d466804a4fd06d73075cc03767a5b4944e3f7d62e1da002c49c71f","tests/wpt/tests/webrtc/RTCPeerConnectionIceErrorEvent.html":"4232b52ef4fbd19f75bf86164c9750ab00fadafcb0d9bf21b7e27ffb40ee25c3"},"scope":"Same archived selected source cases, not a new whole-manifest baseline or a WPT gain attribution; six media assertions per engine remain failed.","failure":null}
```


## 2026-11-02: E2h final source, boundaries and cleanup audit

The full 32-case transport matrix passed with installed headed Chrome 156.0.8078.74 and Firefox 159.0, both engines, both offerer roles, direct/forced TURN and close/navigation. Every case verified both text messages and the 48,000-byte binary in each direction, channel modes, selected UDP route, remote certificate, wrapper revocation and actual socket ownership/rebind. Firefox's origin-scoped local/loopback-network grants were reset; no camera/microphone permission or WebRTC preference override was used.

Final review then corrected observation privacy. Eight targeted cases across both browsers/engines/routes pass with remote stats addresses null, remote URL/relayProtocol absent, and no peer-reflexive candidates in the public remote-candidate list. The record carries working-file hashes because that run preceded its source commit. A final cold-start case, 16 ordinary native tests, two explicitly selected coturn tests, strict native library/fixture Clippy, WPT receiving runner build and Ortet all-target receiving check pass at the final committed source. The selected historical WPT case identities/results are unchanged: 89/95 plus the separate ICE-error constructor 1/1 per engine. Six media assertions per engine remain failed. This is a preserved selected case map, not full WebRTC/media conformance or a newly measured WPT gain.

Failed exploratory records remain above. The first fixture used Windows' small executable-entry stack; it now uses an 8 MiB runtime thread. The browser/schema/ownership probes exposed candidate stats namespace gaps, an undriven multicast route, Firefox launcher/window handling and navigation readiness. Firefox's direct WSL server path timed out in this environment; the final bounded Windows UDP front end forwards unchanged packets to real coturn using distinct backend sockets. Gathering errors are retained independently of successful candidate pairs. A later startup-inclusive transfer deadline failed once; its unchanged retry passed all eight cases, and startup and transfer now have separate bounded deadlines. One strict Clippy run refused a nested if in the privacy repair; the final let-chain passes.

Earlier exploratory `profilesRemoved: true` fields were optimistic and incorrect: automatic approval review blocked recursive deletion, including the verified literal-path retry. They are corrected by this appended audit, not overwritten. Both generated profiles remain in the stable target cache, with all browser/native/fixture owners stopped. The task-owned foreground coturn process was verified before SIGINT; its automatic service remains inactive and disabled. No new isolated Cargo target, Cargo home or worktree was created. DNS/mDNS resolution, TCP/TLS, restart/server replacement, public/cross-machine deployment, browser-guest, media and application-carrier cutover retain separate gates.

The ignored root lock is reconstructed from the full archived receiving lock plus this delta; only the Murm source revision differs. No package or dependency edge changed.

```diff
--- archived full receiving lock
+++ E2h receiving lock
@@ -4567,7 +4567,7 @@
 [[package]]
 name = "murm-rtc"
 version = "0.1.0"
-source = "git+https://github.com/merely-made/murm.git?rev=c58b70421d31ed9f2288dc44152dcb6d719552a9#c58b70421d31ed9f2288dc44152dcb6d719552a9"
+source = "git+https://github.com/merely-made/murm.git?rev=01449d2822a8219f7531b2ea8ed59989535a86eb#01449d2822a8219f7531b2ea8ed59989535a86eb"
 dependencies = [
  "bytes",
  "rtc",
```

```json
{"utc":"2026-11-02T20:20:58.383030+00:00","source":"73a2f0ae4be1e9da10b3acc410d910aa355559cb","supplier":"01449d2822a8219f7531b2ea8ed59989535a86eb","core":"70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38","fullMatrix":{"source":"d644fa0a013364a3d3437b30e98ef3da08dccbb2","utc":"2026-11-02T20:03:54.787Z","uniqueCases":32,"nativeBinarySHA256":"eee9ff809eb71d647a9679881b494dfebadb05d8642c9e2c5536a88b8b041536"},"privacyFollowup":{"recordedHead":"d644fa0a013364a3d3437b30e98ef3da08dccbb2","utc":"2026-11-02T20:14:54.498Z","passingCases":8,"workingSourceSHA256":{"components/genet-webrtc/examples/interop_host.rs":"821b677b6bf201647cca780409886e0ef46b2cf6c52eaec4baab36d6febe394b","support/ci/webrtc_interop.mjs":"d9e4512aab42d7f9ac5b05896879c3ba034aadc1d3fd7ab0e4063c8af5f30c46","support/ci/webrtc_interop_peer.js":"a74a1d9b0d42a096f7353f6f6abe11cae7a65a3defbea4579d0001ebf4c4f5b6","components/genet-webrtc/Cargo.toml":"a0c7b33c3e49b10ba972f67c1169c674046cb6c5e717caa9f6ed2d1c0e3a4265","components/genet-webrtc/src/lib.rs":"c91da3fea1701e320035a78e2f3d0f52edf89925d7ca25e6a2241828944ff279","components/script-runtime-api/webrtc/bootstrap.js":"b16a9c2cc771628ab6800492789278dec6fccc524c8308e2d03272f3b77c1bdd"},"note":"Privacy working bytes were promoted in 73a2f0ae4be; final changes collapse a nested if, clarify a network comment and start the transfer deadline after startup. Final native/WPT/Clippy/Ortet checks and cold-start smoke use that committed source."},"lockSHA256":"0a42b50cb56df741ad34acc18be60070a2eb3244a3a50d64260e7e67e9754f9f","localCargoOverrides":"No task-local patch, Cargo home or target override beyond the recorded stable target/build paths. Existing repository configuration retained.","nativeFixtureSHA256":"821b677b6bf201647cca780409886e0ef46b2cf6c52eaec4baab36d6febe394b","coturnFixtureSHA256":"745ad015cf1c9c8703698794a7c43f8da92c622c5f6d1f1812d3ad7282db9875","ownedProcessesAfterStop":{"nativePeer":0,"profileBrowsers":0,"coturn":0,"coturnService":"inactive and disabled"},"retainedProfileCache":{"owner":"Genet RTC browser interoperability fixture","path":"C:\\t\\cargo-targets\\genet\\rtc-interop","reason":"automatic approval review rejected verified recursive deletion as blocked by policy; marker-owned profiles retained for reuse","chromeBytes":99462130,"firefoxBytes":109002473},"newIsolatedTargetCargoHomeOrWorktree":false}
```

Partial independent coturn observations (bounded summary only):

```json
{"coturnObservedPartialWindow":{"scope":"Partial stdout observations across exploratory and final runs; output was truncated, so these are not total or per-matrix counts.","allocations":35,"permissions":107,"refreshes":215,"lifetimeZero":36,"selectedLines":["5005: session 000000000000000074: CREATE_PERMISSION processed, success","5005: session 000000000000000075: CREATE_PERMISSION processed, success","5011: session 000000000000000074: refreshed, lifetime=0","5011: session 000000000000000075: refreshed, lifetime=0"]}}
```
