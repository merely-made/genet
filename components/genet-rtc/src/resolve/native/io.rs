// SPDX-License-Identifier: MPL-2.0
use socket2::{Domain, Protocol, Socket, Type};
use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpStream, UdpSocket};

pub(super) enum Io {
    Udp {
        socket: UdpSocket,
        sent: bool,
        buffer: Vec<u8>,
    },
    Tcp {
        stream: TcpStream,
        connecting: bool,
        output: Vec<u8>,
        sent: usize,
        input: Vec<u8>,
        limit: usize,
    },
}
impl Io {
    pub fn udp(server: SocketAddr) -> io::Result<Self> {
        let bind = if server.is_ipv4() {
            "0.0.0.0:0"
        } else {
            "[::]:0"
        };
        let socket = UdpSocket::bind(bind)?;
        socket.set_nonblocking(true)?;
        // Connected UDP restricts replies to the selected recursive server.
        socket.connect(server)?;
        Ok(Self::Udp {
            socket,
            sent: false,
            buffer: vec![0; 65536],
        })
    }
    pub fn tcp(server: SocketAddr, packet: &[u8], limit: usize) -> io::Result<Self> {
        let socket = Socket::new(
            Domain::for_address(server),
            Type::STREAM,
            Some(Protocol::TCP),
        )?;
        socket.set_nonblocking(true)?;
        let connecting = match socket.connect(&server.into()) {
            Ok(()) => false,
            Err(e) if connecting_error(&e) => true,
            Err(e) => return Err(e),
        };
        let mut output = (packet.len() as u16).to_be_bytes().to_vec();
        output.extend_from_slice(packet);
        Ok(Self::Tcp {
            stream: socket.into(),
            connecting,
            output,
            sent: 0,
            input: Vec::new(),
            limit,
        })
    }
    pub fn poll(&mut self, packet: &[u8]) -> io::Result<Option<Vec<u8>>> {
        match self {
            Self::Udp {
                socket,
                sent,
                buffer,
            } => {
                if !*sent {
                    match socket.send(packet) {
                        Ok(n) if n == packet.len() => *sent = true,
                        Ok(_) => return Err(io::ErrorKind::WriteZero.into()),
                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => return Ok(None),
                        Err(e) => return Err(e),
                    }
                }
                match socket.recv(buffer) {
                    Ok(n) => Ok(Some(buffer[..n].to_vec())),
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => Ok(None),
                    Err(e) => Err(e),
                }
            }
            Self::Tcp {
                stream,
                connecting,
                output,
                sent,
                input,
                limit,
            } => {
                if *connecting {
                    if let Some(e) = stream.take_error()? {
                        return Err(e);
                    }
                    match stream.peer_addr() {
                        Ok(_) => *connecting = false,
                        Err(e)
                            if e.kind() == io::ErrorKind::NotConnected || connecting_error(&e) =>
                        {
                            return Ok(None);
                        }
                        Err(e) => return Err(e),
                    }
                }
                if *sent < output.len() {
                    match stream.write(&output[*sent..]) {
                        Ok(0) => return Err(io::ErrorKind::WriteZero.into()),
                        Ok(n) => *sent += n,
                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => return Ok(None),
                        Err(e) => return Err(e),
                    }
                }
                if *sent < output.len() {
                    return Ok(None);
                }
                if let Some(packet) = frame(input, *limit)? {
                    return Ok(Some(packet));
                }
                let mut buffer = [0; 4096];
                match stream.read(&mut buffer) {
                    Ok(0) => return Err(io::ErrorKind::UnexpectedEof.into()),
                    Ok(n) => {
                        if input.len() + n > *limit + 2 {
                            return Err(io::ErrorKind::InvalidData.into());
                        }
                        input.extend_from_slice(&buffer[..n]);
                    }
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => return Ok(None),
                    Err(e) => return Err(e),
                }
                frame(input, *limit)
            }
        }
    }
}
fn frame(input: &mut Vec<u8>, limit: usize) -> io::Result<Option<Vec<u8>>> {
    if input.len() < 2 {
        return Ok(None);
    }
    let size = usize::from(u16::from_be_bytes([input[0], input[1]]));
    if size < 12 || size > limit {
        return Err(io::ErrorKind::InvalidData.into());
    }
    if input.len() < size + 2 {
        return Ok(None);
    }
    let result = input[2..size + 2].to_vec();
    input.drain(..size + 2);
    Ok(Some(result))
}
fn connecting_error(e: &io::Error) -> bool {
    if e.kind() == io::ErrorKind::WouldBlock {
        return true;
    }
    #[cfg(unix)]
    if e.raw_os_error() == Some(libc::EINPROGRESS) {
        return true;
    }
    #[cfg(windows)]
    if e.raw_os_error() == Some(10035) {
        return true;
    }
    false
}
