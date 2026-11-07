use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use crate::emulator::Emulator;
use std::cell::RefCell;

pub fn bin_serialize<S>(x: &[u8], s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_bytes(x)
}

use crate::AbsintheError;

pub fn bin_deserialize_mac<'de, D>(d: D) -> Result<[u8; 6], D::Error>
where
    D: Deserializer<'de>,
{
    bin_deserialize(d)?
        .try_into()
        .map_err(|_| de::Error::custom("MAC address must contain six bytes"))
}

pub fn bin_deserialize<'de, D>(d: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    use core::fmt;

    struct DataVisitor;

    impl<'de> de::Visitor<'de> for DataVisitor {
        type Value = Vec<u8>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a byte array")
        }

        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_byte_buf(v.to_owned())
        }

        fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(v)
        }
    }

    d.deserialize_byte_buf(DataVisitor)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareConfig {
    pub product_name: String,
    #[serde(
        serialize_with = "bin_serialize",
        deserialize_with = "bin_deserialize_mac"
    )]
    pub io_mac_address: [u8; 6],
    pub platform_serial_number: String,
    pub platform_uuid: String,
    pub root_disk_uuid: String,
    pub board_id: String,
    pub os_build_num: String,
    #[serde(serialize_with = "bin_serialize", deserialize_with = "bin_deserialize")]
    pub platform_serial_number_enc: Vec<u8>,
    #[serde(serialize_with = "bin_serialize", deserialize_with = "bin_deserialize")]
    pub platform_uuid_enc: Vec<u8>,
    #[serde(serialize_with = "bin_serialize", deserialize_with = "bin_deserialize")]
    pub root_disk_uuid_enc: Vec<u8>,
    #[serde(serialize_with = "bin_serialize", deserialize_with = "bin_deserialize")]
    pub rom: Vec<u8>,
    #[serde(serialize_with = "bin_serialize", deserialize_with = "bin_deserialize")]
    pub rom_enc: Vec<u8>,
    pub mlb: String,
    #[serde(serialize_with = "bin_serialize", deserialize_with = "bin_deserialize")]
    pub mlb_enc: Vec<u8>,
}

impl HardwareConfig {
    pub fn from_validation_data(_data: &[u8]) -> Result<Self, AbsintheError> {
        Err(AbsintheError(-10001))
    }
}

pub struct ValidationCtx {
    emulator: RefCell<Emulator>,
    context: u64,
    established: bool,
}

unsafe impl Send for ValidationCtx {}

impl ValidationCtx {
    pub fn new(
        cert_chain: &[u8],
        out_request_bytes: &mut Vec<u8>,
        hw_config: &HardwareConfig,
    ) -> Result<Self, AbsintheError> {
        let mut emulator = Emulator::new(hw_config.clone())?;
        let cert = emulator.bytes(cert_chain)?;
        let outputs = emulator.allocate(24)?;
        emulator.call(
            0xb1db0,
            &[
                cert,
                cert_chain.len() as u64,
                outputs,
                outputs + 8,
                outputs + 16,
            ],
        )?;
        let context = emulator.read_u64(outputs)?;
        let request = emulator.output(outputs + 8, outputs + 16)?;
        *out_request_bytes = request;
        Ok(Self {
            emulator: RefCell::new(emulator),
            context,
            established: false,
        })
    }

    pub fn key_establishment(&mut self, response: &[u8]) -> Result<(), AbsintheError> {
        if self.established {
            return Err(AbsintheError(-10002));
        }
        let emulator = self.emulator.get_mut();
        let input = emulator.bytes(response)?;
        emulator.call(0xb1dd0, &[self.context, input, response.len() as u64])?;
        self.established = true;
        Ok(())
    }

    pub fn sign(&self) -> Result<Vec<u8>, AbsintheError> {
        if !self.established {
            return Err(AbsintheError(-10002));
        }
        let mut emulator = self
            .emulator
            .try_borrow_mut()
            .map_err(|_| AbsintheError(-10002))?;
        let outputs = emulator.allocate(16)?;
        emulator.call(0xb1df0, &[self.context, 0, 0, outputs, outputs + 8])?;
        emulator.output(outputs, outputs + 8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_mac_returns_error() {
        let value =
            serde::de::value::BorrowedBytesDeserializer::<serde::de::value::Error>::new(&[1, 2, 3]);
        assert!(bin_deserialize_mac(value).is_err());
    }

    #[test]
    fn validation_context_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<ValidationCtx>();
    }

    #[test]
    fn extracting_hardware_returns_error() {
        assert!(HardwareConfig::from_validation_data(&[]).is_err());
    }
}

