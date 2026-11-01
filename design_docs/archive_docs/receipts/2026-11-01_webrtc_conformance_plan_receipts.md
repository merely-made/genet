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
