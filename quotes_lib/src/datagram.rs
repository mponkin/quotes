//! Module for reading and writing messages as datagrams

use crate::{error::QuotesError, server_message::ServerMessage, subscribe_message::PingMessage};

/// Struct to wrap data as datagrams
#[derive(Debug, PartialEq, Eq)]
pub struct Datagram {
    /// data to send
    pub data: Vec<u8>,
}

impl Datagram {
    /// Create new datagram
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    const HEADER: &[u8; 4] = b"QDTG";

    fn bytes_length(&self) -> usize {
        let data_len = (self.data.len() as u16).to_be_bytes();
        Self::HEADER.len() + data_len.len() + self.data.len()
    }
}

impl Into<Vec<u8>> for Datagram {
    fn into(self) -> Vec<u8> {
        let data_len = (self.data.len() as u16).to_be_bytes();
        let mut buffer = Vec::with_capacity(Self::HEADER.len() + data_len.len() + self.data.len());

        buffer.extend_from_slice(Self::HEADER);
        buffer.extend_from_slice(&data_len);
        buffer.extend_from_slice(&self.data);

        buffer
    }
}

impl From<ServerMessage> for Datagram {
    fn from(value: ServerMessage) -> Self {
        Datagram::new(value.into())
    }
}

impl From<PingMessage> for Datagram {
    fn from(value: PingMessage) -> Self {
        Datagram::new(value.into())
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ParseResult {
    Datagram(Datagram),
    NotEnoughBytes,
    Error,
}

impl From<&[u8]> for ParseResult {
    fn from(value: &[u8]) -> Self {
        const DATA_LEN_SIZE: usize = 2;
        let mandatory_len = Datagram::HEADER.len() + DATA_LEN_SIZE;
        if value.len() < mandatory_len {
            return ParseResult::NotEnoughBytes;
        }

        if value[0..Datagram::HEADER.len()] != *Datagram::HEADER {
            return ParseResult::Error;
        }

        let mut data_len_bytes = [0u8; DATA_LEN_SIZE];
        data_len_bytes.copy_from_slice(
            &value[Datagram::HEADER.len()..Datagram::HEADER.len() + DATA_LEN_SIZE],
        );

        let data_len = u16::from_be_bytes(data_len_bytes) as usize;

        let total_len = data_len + mandatory_len;

        if value.len() < total_len {
            return ParseResult::NotEnoughBytes;
        }

        ParseResult::Datagram(Datagram::new(
            value[mandatory_len..mandatory_len + data_len].to_vec(),
        ))
    }
}

/// Struct to parse datagrams possibly split among multiple messages
pub struct DatagramParser {
    /// leftover partial data from previous read
    buffer: Vec<u8>,
}

impl DatagramParser {
    /// Create new parser
    pub fn new() -> Self {
        Self { buffer: vec![] }
    }

    /// Parse datagrams contained in given data
    pub fn parse(&mut self, data: &[u8]) -> Result<Vec<Datagram>, QuotesError> {
        self.buffer.extend_from_slice(data);

        let mut datagrams = vec![];

        loop {
            let result = ParseResult::from(self.buffer.as_slice());
            match result {
                ParseResult::Datagram(datagram) => {
                    self.buffer.drain(0..datagram.bytes_length());
                    datagrams.push(datagram);
                    if self.buffer.len() == 0 {
                        break;
                    }
                }
                ParseResult::NotEnoughBytes => break,
                ParseResult::Error => return Err(QuotesError::ParseDatagramError),
            }
        }

        Ok(datagrams)
    }
}

mod tests {
    #![allow(unused_imports)]
    use std::u8;

    use super::*;

    #[test]
    fn test_parse_datagram() {
        let data = vec![1, 2, 3, 4];
        let datagram = Datagram::new(data.clone());

        let bytes: Vec<u8> = datagram.into();

        assert_eq!(
            ParseResult::from(bytes.as_slice()),
            ParseResult::Datagram(Datagram::new(data))
        )
    }

    #[test]
    fn test_parse_datagram_not_enough_bytes() {
        let data = vec![1, 2, 3, 4];
        let datagram = Datagram::new(data.clone());

        let bytes: Vec<u8> = datagram.into();

        assert_eq!(
            ParseResult::from(&bytes[0..bytes.len() - 1]),
            ParseResult::NotEnoughBytes
        )
    }
    #[test]
    fn test_parse_error() {
        let data = vec![1, 2, 3, 4];
        let datagram = Datagram::new(data.clone());

        let mut bytes: Vec<u8> = datagram.into();
        bytes[3] = u8::MAX;

        assert_eq!(ParseResult::from(bytes.as_slice()), ParseResult::Error)
    }

    #[test]
    fn test_parse_multiple_datagrams() {
        let datas = vec![vec![1, 2, 3, 4], vec![5, 6, 7, 8, 9, 10]];

        let mut buffer = vec![];

        for data in datas.iter() {
            let datagram = Datagram::new(data.clone());
            let bytes: Vec<u8> = datagram.into();
            buffer.extend_from_slice(&bytes);
        }

        let mut parser = DatagramParser::new();
        let result_datas = parser
            .parse(&buffer)
            .expect("Should parse successfully")
            .into_iter()
            .map(|dg| dg.data)
            .collect::<Vec<_>>();

        assert_eq!(datas, result_datas)
    }

    #[test]
    fn test_parse_partial_datagrams() {
        let datas = vec![vec![1, 2, 3, 4], vec![5, 6, 7, 8, 9, 10]];

        let mut parser = DatagramParser::new();

        let datagram1 = Datagram::new(datas[0].clone());
        let bytes1: Vec<u8> = datagram1.into();

        let result1 = parser
            .parse(&bytes1[0..&bytes1.len() - 1])
            .expect("Should parse successfully");

        assert!(result1.is_empty());

        let mut buffer = vec![];

        buffer.push(bytes1[bytes1.len() - 1]);

        let datagram2 = Datagram::new(datas[1].clone());
        let bytes2: Vec<u8> = datagram2.into();

        buffer.extend_from_slice(&bytes2);

        let result_datas = parser
            .parse(&buffer)
            .expect("Should parse successfully")
            .into_iter()
            .map(|dg| dg.data)
            .collect::<Vec<_>>();

        assert_eq!(datas, result_datas)
    }
}
