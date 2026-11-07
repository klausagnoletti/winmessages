use crate::{failure, AbsintheError};
use sha1::{Digest, Sha1};
use std::{path::PathBuf, time::Duration};

pub(crate) struct Segment {
    pub address: u64,
    pub size: u64,
    pub offset: usize,
    pub file_size: usize,
}

pub(crate) struct Image {
    pub bytes: Vec<u8>,
    pub segments: Vec<Segment>,
    pub binds: Vec<(u64, String)>,
}

fn range(b: &[u8], p: usize, n: usize) -> Result<&[u8], AbsintheError> {
    b.get(p..p.checked_add(n).ok_or_else(|| failure("Mach-O overflow"))?)
        .ok_or_else(|| failure("Truncated Mach-O"))
}
fn u32_at(b: &[u8], p: usize) -> Result<u32, AbsintheError> {
    Ok(u32::from_le_bytes(range(b, p, 4)?.try_into().unwrap()))
}
fn u64_at(b: &[u8], p: usize) -> Result<u64, AbsintheError> {
    Ok(u64::from_le_bytes(range(b, p, 8)?.try_into().unwrap()))
}
fn string_at(b: &[u8], p: usize) -> Result<String, AbsintheError> {
    let tail = b.get(p..).ok_or_else(|| failure("Invalid string offset"))?;
    let len = tail
        .iter()
        .position(|x| *x == 0)
        .ok_or_else(|| failure("Unterminated symbol"))?;
    String::from_utf8(tail[..len].to_vec()).map_err(failure)
}
fn leb(b: &[u8], p: &mut usize) -> Result<u64, AbsintheError> {
    let mut value = 0;
    for shift in (0..64).step_by(7) {
        let byte = range(b, *p, 1)?[0];
        *p += 1;
        value |= u64::from(byte & 127) << shift;
        if byte & 128 == 0 {
            return Ok(value);
        }
    }
    Err(failure("Invalid LEB128"))
}

pub(crate) fn load() -> Result<Image, AbsintheError> {
    let path = if let Some(path) = std::env::var_os("WINMESSAGES_NAC_BIN") {
        PathBuf::from(path)
    } else {
        std::env::current_exe()
            .map_err(failure)?
            .with_file_name("IMDAppleServices")
    };
    let binary = if path.exists() || std::env::var_os("WINMESSAGES_NAC_BIN").is_some() {
        std::fs::read(path).map_err(failure)?
    } else {
        std::thread::spawn(|| -> Result<Vec<u8>, AbsintheError> {
            let client = reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(120))
                .build()
                .map_err(failure)?;
            let response = client
                .get("https://github.com/JJTech0130/nacserver/raw/main/IMDAppleServices")
                .send()
                .map_err(failure)?
                .error_for_status()
                .map_err(failure)?;
            Ok(response.bytes().map_err(failure)?.to_vec())
        })
        .join()
        .map_err(|_| failure("Binary download thread failed"))??
    };
    if format!("{:x}", Sha1::digest(&binary)) != "e1181ccad82e6629d52c6a006645ad87ee59bd13" {
        return Err(failure("IMDAppleServices SHA-1 mismatch"));
    }
    parse(&binary)
}

fn parse(binary: &[u8]) -> Result<Image, AbsintheError> {
    if range(binary, 0, 4)? != [0xca, 0xfe, 0xba, 0xbe] {
        return Err(failure("Expected universal Mach-O"));
    }
    let be = |p| -> Result<usize, AbsintheError> {
        Ok(u32::from_be_bytes(range(binary, p, 4)?.try_into().unwrap()) as usize)
    };
    let mut selected = None;
    for i in 0..be(4)? {
        let p = 8 + i * 20;
        if be(p)? == 0x01000007 {
            selected = Some(range(binary, be(p + 8)?, be(p + 12)?)?);
            break;
        }
    }
    let b = selected.ok_or_else(|| failure("No x86_64 slice"))?;
    if u32_at(b, 0)? != 0xfeedfacf {
        return Err(failure("Expected 64-bit Mach-O"));
    }
    let mut segments = Vec::new();
    let mut sections = Vec::new();
    let (mut symbols, mut strings, mut indirect) = (0, 0, 0);
    let mut bind_streams = Vec::new();
    let mut p = 32;
    for _ in 0..u32_at(b, 16)? {
        let cmd = u32_at(b, p)?;
        let len = u32_at(b, p + 4)? as usize;
        if len < 8 {
            return Err(failure("Invalid load command"));
        }
        range(b, p, len)?;
        match cmd {
            0x19 => {
                segments.push(Segment {
                    address: u64_at(b, p + 24)?,
                    size: u64_at(b, p + 32)?,
                    offset: u64_at(b, p + 40)? as usize,
                    file_size: u64_at(b, p + 48)? as usize,
                });
                for i in 0..u32_at(b, p + 64)? as usize {
                    let s = p + 72 + i * 80;
                    let kind = u32_at(b, s + 64)? & 0xff;
                    if kind == 6 || kind == 7 {
                        sections.push((
                            u64_at(b, s + 32)?,
                            u64_at(b, s + 40)?,
                            u32_at(b, s + 68)? as usize,
                        ));
                    }
                }
            }
            2 => {
                symbols = u32_at(b, p + 8)? as usize;
                strings = u32_at(b, p + 16)? as usize;
            }
            0xb => indirect = u32_at(b, p + 56)? as usize,
            0x22 | 0x80000022 => {
                for field in [16, 24, 32] {
                    bind_streams.push((
                        u32_at(b, p + field)? as usize,
                        u32_at(b, p + field + 4)? as usize,
                    ));
                }
            }
            _ => {}
        }
        p += len;
    }
    let mut binds = Vec::new();
    for (addr, size, start) in sections {
        for i in 0..size as usize / 8 {
            let idx = u32_at(b, indirect + (start + i) * 4)?;
            if idx & 0xc0000000 != 0 {
                continue;
            }
            let stridx = u32_at(b, symbols + idx as usize * 16)? as usize;
            binds.push((addr + i as u64 * 8, string_at(b, strings + stridx)?));
        }
    }
    for (offset, size) in bind_streams {
        let stream = range(b, offset, size)?;
        let (mut cursor, mut segment, mut location) = (0, 0usize, 0u64);
        let mut symbol = String::new();
        let mut kind = 1;
        while cursor < stream.len() {
            let byte = stream[cursor];
            cursor += 1;
            let imm = byte & 15;
            let mut count = 0;
            let mut skip = 0;
            match byte & 0xf0 {
                0x00 | 0x10 | 0x30 => {}
                0x20 => {
                    leb(stream, &mut cursor)?;
                }
                0x40 => {
                    symbol = string_at(stream, cursor)?;
                    cursor += symbol.len() + 1;
                }
                0x50 => kind = imm,
                0x60 => {
                    if leb(stream, &mut cursor)? != 0 {
                        return Err(failure("Nonzero binding addend"));
                    }
                }
                0x70 => {
                    segment = imm as usize;
                    location = leb(stream, &mut cursor)?;
                }
                0x80 => location = location.wrapping_add(leb(stream, &mut cursor)?),
                0x90 => count = 1,
                0xa0 => {
                    count = 1;
                    skip = leb(stream, &mut cursor)?;
                }
                0xb0 => {
                    count = 1;
                    skip = imm as u64 * 8;
                }
                0xc0 => {
                    count = leb(stream, &mut cursor)?;
                    skip = leb(stream, &mut cursor)?;
                }
                _ => return Err(failure("Unsupported binding opcode")),
            }
            for _ in 0..count {
                if kind != 1 {
                    return Err(failure("Unsupported binding type"));
                }
                let seg = segments
                    .get(segment)
                    .ok_or_else(|| failure("Invalid binding segment"))?;
                binds.push((seg.address + location, symbol.clone()));
                location = location.wrapping_add(8).wrapping_add(skip);
            }
        }
    }
    Ok(Image {
        bytes: b.to_vec(),
        segments,
        binds,
    })
}
