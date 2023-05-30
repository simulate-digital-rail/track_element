/// The current version of this SCI implementation.
pub(crate) const SCI_VERSION: u8 = 0x01;

pub enum SciError {
    UnknownProtocol(u8),
    UnknownMessageType(u8),
    Other(String),
}

pub(crate) fn str_to_sci_name(name: &str) -> Vec<u8> {
    let mut new_name = vec![b'_'; 20];
    if name.len() < 20 {
        new_name[..name.len()].clone_from_slice(name.as_bytes());
    } else {
        new_name[..20].clone_from_slice(&name.as_bytes()[..20])
    }
    new_name
}

/// Constants to represent SCI Protocol types.
#[repr(u8)]
pub enum ProtocolType {
    SCIProtocolP = 0x40,
    SCIProtocolLS = 0x30,
}

impl TryFrom<u8> for ProtocolType {
    type Error = SciError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x40 => Ok(Self::SCIProtocolP),
            0x30 => Ok(Self::SCIProtocolLS),
            v => Err(SciError::UnknownProtocol(v)),
        }
    }
}

/// The message types for SCI messages. Since
/// protocols may use overlapping integer
/// representations, this is not a enum, but a
/// newtype with associated functions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SCIMessageType(u8);

impl SCIMessageType {
    pub const fn sci_version_request() -> Self {
        Self(0x0024)
    }

    pub const fn sci_version_response() -> Self {
        Self(0x0025)
    }

    pub const fn sci_status_request() -> Self {
        Self(0x0021)
    }

    pub const fn sci_status_begin() -> Self {
        Self(0x0022)
    }

    pub const fn sci_status_finish() -> Self {
        Self(0x0023)
    }

    pub const fn sci_timeout() -> Self {
        Self(0x000C)
    }

    pub fn try_as_sci_message_type(&self) -> Result<&str, SciError> {
        match self.0 {
            0x0024 => Ok("VersionRequest"),
            0x0025 => Ok("VersionResponse"),
            0x0021 => Ok("StatusRequest"),
            0x0022 => Ok("StatusBegin"),
            0x0023 => Ok("StatusFinish"),
            0x000C => Ok("Timeout"),
            v => Err(SciError::UnknownMessageType(v)),
        }
    }

    pub fn try_as_scip_message_type(&self) -> Result<&str, SciError> {
        match self.0 {
            0x0001 => Ok("ChangeLocation"),
            0x000B => Ok("LocationStatus"),
            _ => self.try_as_sci_message_type(),
        }
    }

    pub fn try_as_scils_message_type(&self) -> Result<&str, SciError> {
        match self.0 {
            0x0001 => Ok("ShowSignalAspect"),
            0x0002 => Ok("ChangeBrightness"),
            0x0003 => Ok("SignalAspectStatus"),
            0x0004 => Ok("BrightnessStatus"),
            _ => self.try_as_sci_message_type(),
        }
    }
}

impl From<SCIMessageType> for u8 {
    fn from(val: SCIMessageType) -> Self {
        val.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SCIVersionCheckResult {
    NotAllowedToUse = 0,
    VersionsAreNotEqual = 1,
    VersionsAreEqual = 2,
}

impl TryFrom<u8> for SCIVersionCheckResult {
    type Error = SciError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::NotAllowedToUse),
            1 => Ok(Self::VersionsAreEqual),
            2 => Ok(Self::VersionsAreEqual),
            v => Err(SciError::Other(format!(
                "Unknown SCI Version check result `{v:x}`"
            ))),
        }
    }
}

impl TryFrom<u8> for SCIMessageType {
    type Error = SciError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x000C => Ok(Self::sci_timeout()),
            v => Err(SciError::UnknownMessageType(v)),
        }
    }
}

/// The payload of an [`SCITelegram`]. Usually constructed from
/// a slice using [`SCIPayload::from_slice`].
pub struct SCIPayload {
    pub data: [u8; 85],
    pub used: usize,
}

impl Default for SCIPayload {
    fn default() -> Self {
        Self {
            data: [0; 85],
            used: 0,
        }
    }
}

impl SCIPayload {
    fn from_slice(data: &[u8]) -> Self {
        let mut payload = Self {
            used: data.len(),
            ..Default::default()
        };
        payload.data[..data.len()].copy_from_slice(data);
        payload
    }
}

/// An SCI message. You should construct these using the generic
/// and protocol-specific associated functions.
pub struct SCITelegram {
    pub protocol_type: ProtocolType,
    pub message_type: SCIMessageType,
    pub sender: String,
    pub receiver: String,
    pub payload: SCIPayload,
}

impl SCITelegram {
    pub fn version_request(
        protocol_type: ProtocolType,
        sender: &str,
        receiver: &str,
        version: u8,
    ) -> Self {
        Self {
            protocol_type,
            message_type: SCIMessageType::sci_version_request(),
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            payload: SCIPayload::from_slice(&[version]),
        }
    }

    pub fn version_response(
        protocol_type: ProtocolType,
        sender: &str,
        receiver: &str,
        version: u8,
        version_check_result: SCIVersionCheckResult,
        checksum: &[u8],
    ) -> Self {
        let mut payload_data = vec![version_check_result as u8, version, checksum.len() as u8];
        payload_data.append(&mut Vec::from(checksum));
        Self {
            protocol_type,
            message_type: SCIMessageType::sci_version_response(),
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            payload: SCIPayload::from_slice(&payload_data),
        }
    }

    pub fn status_request(protocol_type: ProtocolType, sender: &str, receiver: &str) -> Self {
        Self {
            protocol_type,
            message_type: SCIMessageType::sci_status_request(),
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            payload: SCIPayload::default(),
        }
    }

    pub fn status_begin(protocol_type: ProtocolType, sender: &str, receiver: &str) -> Self {
        Self {
            protocol_type,
            message_type: SCIMessageType::sci_status_begin(),
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            payload: SCIPayload::default(),
        }
    }

    pub fn status_finish(protocol_type: ProtocolType, sender: &str, receiver: &str) -> Self {
        Self {
            protocol_type,
            message_type: SCIMessageType::sci_status_finish(),
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            payload: SCIPayload::default(),
        }
    }

    pub fn timeout(protocol_type: ProtocolType, sender: &str, receiver: &str) -> Self {
        Self {
            protocol_type,
            message_type: SCIMessageType::sci_timeout(),
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            payload: SCIPayload::default(),
        }
    }
}

impl TryFrom<&[u8]> for SCITelegram {
    type Error = SciError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self {
            protocol_type: ProtocolType::try_from(value[0])?,
            message_type: SCIMessageType::try_from(value[1])?,
            sender: String::from_utf8_lossy(&value[2..22]).to_string(),
            receiver: String::from_utf8_lossy(&value[22..42]).to_string(),
            payload: SCIPayload::from_slice(&value[42..]),
        })
    }
}

impl From<SCITelegram> for Vec<u8> {
    fn from(val: SCITelegram) -> Self {
        let mut data = vec![val.protocol_type as u8, val.message_type.into()];
        data.append(&mut str_to_sci_name(&val.sender));
        data.append(&mut str_to_sci_name(&val.receiver));
        if val.payload.used > 0 {
            let mut payload = Vec::from(val.payload.data);
            data.append(&mut payload);
        }
        data
    }
}
