// SPDX-License-Identifier: MPL-2.0
//! Shared WebRTC protocol implementation and explicit native host adapter.
//!
//! [`rtc`] is the one pinned protocol API for consumers. It exposes the full
//! peer connection rather than an application-specific channel profile. The
//! default build owns no socket, thread or async runtime. Native hosts can
//! select `udp` and drive [`udp::UdpPeer`] from their own event loop.
//!
//! Web APIs, application framing, invitations, identities and admission belong
//! to consumers. Candidate discovery and relay allocation beyond a concrete
//! local host address remain explicit host integration work.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// The pinned Sans-I/O protocol implementation shared by all adapters.
pub use rtc;

#[cfg(all(feature = "udp", not(target_arch = "wasm32")))]
pub mod udp;
