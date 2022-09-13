use anyhow::Result;
use libc::*;
use std::convert::Into;
use std::ffi::CString;

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum BundelPolicy {
    Balanced = 1,
    MaxCompat,
    MaxBundle,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum IceTransportPolicy {
    None = 1,
    Relay,
    Public,
    All,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum RtcpMuxPolicy {
    Negotiate = 1,
    Require,
}

#[repr(C)]
pub struct RawRTCIceServer {
    credential: *const c_char,
    urls: *const *const c_char,
    urls_size: u8,
    urls_capacity: u8,
    username: *const c_char,
}

impl Drop for RawRTCIceServer {
    fn drop(&mut self) {
        unsafe {
            let _ = CString::from_raw(self.credential as *mut c_char);
            let _ = CString::from_raw(self.username as *mut c_char);
            for url in Vec::from_raw_parts(
                self.urls as *mut *const c_char,
                self.urls_size as usize,
                self.urls_capacity as usize,
            ) {
                let _ = CString::from_raw(url as *mut c_char);
            }
        }
    }
}

#[repr(C)]
pub struct RawRTCPeerConnectionConfigure {
    bundle_policy: u8,        // BundelPolicy
    ice_transport_policy: u8, // IceTransportPolicy
    peer_identity: *const c_char,
    rtcp_mux_policy: u8, // RtcpMuxPolicy
    ice_servers: *const RawRTCIceServer,
    ice_servers_size: u8,
    ice_servers_capacity: u8,
    ice_candidate_pool_size: u8,
}

impl Drop for RawRTCPeerConnectionConfigure {
    fn drop(&mut self) {
        unsafe {
            let _ = CString::from_raw(self.peer_identity as *mut c_char);
            let _ = Vec::from_raw_parts(
                self.ice_servers as *mut RawRTCIceServer,
                self.ice_servers_size as usize,
                self.ice_servers_capacity as usize,
            );
        }
    }
}

impl RawRTCPeerConnectionConfigure {
    pub fn into_raw(self) -> *const Self {
        Box::into_raw(Box::new(self))
    }

    pub fn from_raw(raw: *const Self) -> Box<Self> {
        unsafe { Box::from_raw(raw as *mut Self) }
    }
}

/// RTCIceServer
///
/// An array of RTCIceServer objects, each describing one server which may be used
/// by the ICE agent; these are typically STUN and/or TURN servers.
/// If this isn't specified, the connection attempt will be made with no STUN or
/// TURN server available, which limits the connection to local peers.
#[derive(Default, Clone, Debug)]
pub struct RTCIceServer {
    /// The credential to use when logging into the server.
    /// This is only used if the RTCIceServer represents a TURN server.
    pub credential: Option<String>,
    /// If the RTCIceServer is a TURN server, then this is the username to use
    /// during the authentication process.
    pub username: Option<String>,
    /// This required property is either a single string or an array of strings,
    /// each specifying a URL which can be used to connect to the server.
    pub urls: Option<Vec<String>>,
}

impl Into<RawRTCIceServer> for RTCIceServer {
    fn into(self) -> RawRTCIceServer {
        let (urls, urls_size, urls_capacity) = self
            .urls
            .map(|v| {
                v.iter()
                    .map(|s| CString::new(s.clone()).unwrap().into_raw() as *const c_char)
                    .collect::<Vec<*const c_char>>()
                    .into_raw_parts()
            })
            .unwrap_or((std::ptr::null_mut(), 0, 0));

        RawRTCIceServer {
            credential: self
                .credential
                .map(|s| CString::new(s).unwrap().into_raw())
                .unwrap_or(std::ptr::null_mut()),
            username: self
                .username
                .map(|s| CString::new(s).unwrap().into_raw())
                .unwrap_or(std::ptr::null_mut()),
            urls_capacity: urls_capacity as u8,
            urls_size: urls_size as u8,
            urls,
        }
    }
}

/// RTCPeerConnection
///
/// The RTCPeerConnection is a newly-created RTCPeerConnection,
/// which represents a connection between the local device and a remote peer.
#[derive(Default, Clone, Debug)]
pub struct RTCConfiguration {
    /// Specifies how to handle negotiation of candidates when the remote peer
    /// is not compatible with the SDP BUNDLE standard. If the remote endpoint
    /// is BUNDLE-aware, all media tracks and data channels are bundled onto a
    /// single transport at the completion of negotiation, regardless of policy
    /// used, and any superfluous transports that were created initially are
    /// closed at that point.
    ///
    /// In technical terms, a BUNDLE lets all media flow between two peers flow
    /// across a single 5-tuple;
    /// that is, from a single IP and port on one peer to a single IP and port
    /// on the other peer, using the same transport protocol.
    pub bundle_policy: Option<BundelPolicy>,
    /// The current ICE transport policy; if the policy isn't specified, all is
    /// assumed by default, allowing all candidates to be considered
    pub ice_transport_policy: Option<IceTransportPolicy>,
    /// A string which specifies the target peer identity for the
    /// RTCPeerConnection.
    /// If this value is set (it defaults to null), the RTCPeerConnection will
    /// not connect to a remote peer unless it can successfully authenticate
    /// with the given name.
    pub peer_identity: Option<String>,
    /// The RTCP mux policy to use when gathering ICE candidates, in order to
    /// support non-multiplexed RTCP.
    pub rtcp_mux_policy: Option<RtcpMuxPolicy>,
    /// An array of RTCIceServer objects, each describing one server which may
    /// be used by the ICE agent; these are typically STUN and/or TURN servers.
    /// If this isn't specified, the connection attempt will be made with no
    /// STUN or TURN server available, which limits the connection to local
    /// peers.
    pub ice_servers: Option<Vec<RTCIceServer>>,
    /// An unsigned 16-bit integer value which specifies the size of the
    /// prefetched ICE candidate pool.
    /// The default value is 0 (meaning no candidate prefetching will occur).
    /// You may find in some cases that connections can be established more
    /// quickly by allowing the ICE agent to start fetching ICE candidates
    /// before you start trying to connect, so that they're already available
    /// for inspection when RTCPeerConnection.setLocalDescription() is called.
    pub ice_candidate_pool_size: Option<u8>,
}

impl Into<RawRTCPeerConnectionConfigure> for RTCConfiguration {
    fn into(self) -> RawRTCPeerConnectionConfigure {
        let (ice_servers, ice_servers_size, ice_servers_capacity) = self
            .ice_servers
            .map(|i| {
                i.iter()
                    .map(|s| s.clone().into())
                    .collect::<Vec<RawRTCIceServer>>()
                    .into_raw_parts()
            })
            .unwrap_or((std::ptr::null_mut(), 0, 0));
        
        RawRTCPeerConnectionConfigure {
            bundle_policy: self.bundle_policy.map(|i| i as u8).unwrap_or(0),
            ice_transport_policy: self.ice_transport_policy.map(|i| i as u8).unwrap_or(0),
            peer_identity: self
                .peer_identity
                .map(|s| CString::new(s).unwrap().into_raw())
                .unwrap_or(std::ptr::null_mut()),
            rtcp_mux_policy: self.rtcp_mux_policy.map(|i| i as u8).unwrap_or(0),
            ice_candidate_pool_size: self.ice_candidate_pool_size.unwrap_or(0),
            ice_servers_capacity: ice_servers_capacity as u8,
            ice_servers_size: ice_servers_size as u8,
            ice_servers,
        }
    }
}