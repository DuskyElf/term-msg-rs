use serde::{Serialize, Deserialize};

pub const PORT: u16 = 6969;

pub enum Error {
    InvalidEvent,
    IoError(String),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ServerInfo {
    pub server_name: String,
    pub is_protected: bool,
}

#[derive(Serialize, Deserialize)]
pub enum ServerToClientEvent {
    // Handshake
    PingResponce {
        identifier: u64,
        server_info: ServerInfo,
    },

    // Verification
    VerificationFailed,
    VerificationSuccessful,

    // Login
    UsernameTaken,
    InvalidUsername,
    LoginSuccessful,

    // Disconnect
    Disconnect,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientToServerEvent {
    Ping { identifier: u64 },
    Verify { password: String },
    Login { username: String },
    Disconnect,
}

pub enum NetworkPhase {
    Ping,
    Verify,
    Login,
}

impl ClientToServerEvent {
    pub fn expected(self, phase: NetworkPhase) -> Result<Self, Error> {
        match phase {
            NetworkPhase::Ping => {
                if let Self::Ping { identifier: _ } = self {
                    Ok(self)
                } else {
                    Err(Error::InvalidEvent)
                }
            }
            NetworkPhase::Verify => {
                if let Self::Verify { password: _ } = self {
                    Ok(self)
                } else {
                    Err(Error::InvalidEvent)
                }
            }
            NetworkPhase::Login => {
                if let Self::Login { username: _ } = self {
                    Ok(self)
                } else {
                    Err(Error::InvalidEvent)
                }
            }
        }
    }
}

impl ServerToClientEvent {
    pub fn expected(self, phase: NetworkPhase) -> Result<Self, Error> {
        match phase {
            NetworkPhase::Ping => {
                if let Self::PingResponce { identifier: _, server_info: _ } = self {
                    Ok(self)
                } else {
                    Err(Error::InvalidEvent)
                }
            }
            NetworkPhase::Verify => {
                match self {
                    Self::VerificationFailed | Self::VerificationSuccessful => Ok(self),
                    _ => Err(Error::InvalidEvent),
                }
            }
            NetworkPhase::Login => {
                match self {
                    Self::InvalidUsername
                    | Self::UsernameTaken
                    | Self::LoginSuccessful => Ok(self),
                    _ => Err(Error::InvalidEvent),
                }
            }
        }
    }
}
