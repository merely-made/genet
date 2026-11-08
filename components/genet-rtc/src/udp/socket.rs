//! Physical socket operations shared by both host drivers.
use super::*;

pub(super) enum Socket {
    Host(std::net::UdpSocket),
    #[cfg(feature = "udp")]
    Async(UdpSocket),
}

impl Socket {
    pub(super) fn local_addr(&self) -> std::io::Result<SocketAddr> {
        match self {
            Self::Host(socket) => socket.local_addr(),
            #[cfg(feature = "udp")]
            Self::Async(socket) => socket.local_addr(),
        }
    }
    pub(super) fn send(&self, bytes: &[u8], peer: SocketAddr) -> std::io::Result<usize> {
        match self {
            Self::Host(socket) => socket.send_to(bytes, peer),
            #[cfg(feature = "udp")]
            Self::Async(socket) => socket.try_send_to(bytes, peer),
        }
    }
    pub(super) fn receive(&self, bytes: &mut [u8]) -> std::io::Result<(usize, SocketAddr)> {
        match self {
            Self::Host(socket) => socket.recv_from(bytes),
            #[cfg(feature = "udp")]
            Self::Async(socket) => socket.try_recv_from(bytes),
        }
    }
    #[cfg(feature = "udp")]
    pub(super) async fn wait(
        &self,
        wake: Instant,
        pending: bool,
        floor: Duration,
    ) -> std::io::Result<()> {
        match self {
            Self::Async(socket) => tokio::select! {
                ready = socket.readable() => ready,
                ready = socket.writable(), if pending => ready,
                _ = tokio::time::sleep_until(wake.into()) => Ok(()),
            },
            Self::Host(_) => {
                let wake = wake.min(Instant::now().checked_add(floor).unwrap_or(wake));
                tokio::time::sleep_until(wake.into()).await;
                Ok(())
            }
        }
    }
}
