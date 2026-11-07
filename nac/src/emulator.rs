use crate::{failure, macho, nac::HardwareConfig, AbsintheError};
use std::collections::HashMap;
use unicorn_engine::{
    unicorn_const::{Arch, Mode, Prot},
    RegisterX86, Unicorn,
};

const HEAP: u64 = 0x10000000;
const HEAP_SIZE: u64 = 0x1000000;
const STACK: u64 = 0x300000;
const STACK_SIZE: u64 = 0x100000;
const STOP: u64 = 0x900000;
const HOOK: u64 = 0xd00000;
const PROPERTY_READER: u64 = 0x39540;
const ARGUMENTS: [RegisterX86; 6] = [
    RegisterX86::RDI,
    RegisterX86::RSI,
    RegisterX86::RDX,
    RegisterX86::RCX,
    RegisterX86::R8,
    RegisterX86::R9,
];

#[derive(Clone)]
enum Object {
    Data(Vec<u8>),
    String(String),
    Dictionary(HashMap<u64, u64>),
}

struct State {
    hardware: HardwareConfig,
    objects: Vec<Object>,
    heap: u64,
    allocations: HashMap<u64, u64>,
    free: Vec<(u64, u64)>,
    iterator: bool,
    fault: Option<AbsintheError>,
}

type Machine<'a> = Unicorn<'a, State>;

pub(crate) struct Emulator {
    machine: Machine<'static>,
}

fn allocate(uc: &mut Machine<'_>, size: u64) -> Result<u64, AbsintheError> {
    let size = size
        .max(1)
        .checked_add(15)
        .ok_or_else(|| failure("Allocation overflow"))?
        & !15;
    let state = uc.get_data_mut();
    let ptr = if let Some(i) = state.free.iter().position(|(_, n)| *n >= size) {
        let (ptr, capacity) = state.free.swap_remove(i);
        if capacity > size {
            state.free.push((ptr + size, capacity - size));
        }
        ptr
    } else {
        let next = state
            .heap
            .checked_add(size)
            .ok_or_else(|| failure("Allocation overflow"))?;
        if next > HEAP + HEAP_SIZE {
            return Err(failure("Emulator heap exhausted"));
        }
        let ptr = state.heap;
        state.heap = next;
        ptr
    };
    state.allocations.insert(ptr, size);
    uc.mem_write(ptr, &vec![0; size as usize])?;
    Ok(ptr)
}

fn read_u64(uc: &Machine<'_>, ptr: u64) -> Result<u64, AbsintheError> {
    let mut value = [0; 8];
    uc.mem_read(ptr, &mut value)?;
    Ok(u64::from_le_bytes(value))
}

fn add_object(uc: &mut Machine<'_>, object: Object) -> u64 {
    let objects = &mut uc.get_data_mut().objects;
    objects.push(object);
    objects.len() as u64
}

fn object(uc: &Machine<'_>, id: u64) -> Result<Object, AbsintheError> {
    uc.get_data()
        .objects
        .get(id.wrapping_sub(1) as usize)
        .cloned()
        .ok_or_else(|| failure(format!("Invalid CF object {id:#x}")))
}

fn cstring(uc: &Machine<'_>, ptr: u64) -> Result<String, AbsintheError> {
    let mut bytes = Vec::new();
    for i in 0..4096 {
        let mut byte = [0];
        uc.mem_read(ptr + i, &mut byte)?;
        if byte[0] == 0 {
            return String::from_utf8(bytes).map_err(failure);
        }
        bytes.push(byte[0]);
    }
    Err(failure("Unterminated guest string"))
}

fn cfstring(uc: &Machine<'_>, ptr: u64) -> Result<String, AbsintheError> {
    if let Some(Object::String(s)) = uc.get_data().objects.get(ptr.wrapping_sub(1) as usize) {
        return Ok(s.clone());
    }
    let address = read_u64(uc, ptr + 16)?;
    let length = read_u64(uc, ptr + 24)?;
    if length > 4096 {
        return Err(failure("Invalid CFString length"));
    }
    String::from_utf8(uc.mem_read_as_vec(address, length as usize)?).map_err(failure)
}

fn nul_data(s: &str) -> Object {
    let mut bytes = s.trim_end_matches('\0').as_bytes().to_vec();
    bytes.push(0);
    Object::Data(bytes)
}

fn property(hw: &HardwareConfig, key: &str) -> Option<Object> {
    let encrypted = match key {
        "Gq3489ugfi" => Some(&hw.platform_serial_number_enc),
        "Fyp98tpgj" => Some(&hw.platform_uuid_enc),
        "kbjfrfpoJU" => Some(&hw.root_disk_uuid_enc),
        "oycqAZloTNDm" => Some(&hw.rom_enc),
        "abKPld1EcMni" => Some(&hw.mlb_enc),
        _ => None,
    };
    if let Some(data) = encrypted {
        return (!data.is_empty()).then(|| Object::Data(data.clone()));
    }
    match key {
        "IOPlatformSerialNumber" => Some(Object::String(hw.platform_serial_number.clone())),
        "IOPlatformUUID" => Some(Object::String(hw.platform_uuid.clone())),
        "board-id" => Some(nul_data(&hw.board_id)),
        "product-name" => Some(nul_data(&hw.product_name)),
        "IOMACAddress" => Some(Object::Data(hw.io_mac_address.to_vec())),
        "4D1EDE05-38C7-4A6A-9CC6-4BCCA8B38C14:MLB" => {
            Some(Object::Data(hw.mlb.as_bytes().to_vec()))
        }
        "4D1EDE05-38C7-4A6A-9CC6-4BCCA8B38C14:ROM" => Some(Object::Data(hw.rom.clone())),
        _ => None,
    }
}

fn dispatch(uc: &mut Machine<'_>, name: &str) -> Result<u64, AbsintheError> {
    if std::env::var_os("WINMESSAGES_NAC_TRACE").is_some() {
        eprintln!("Hook {name}");
    }
    let mut a = [0; 6];
    for (slot, register) in a.iter_mut().zip(ARGUMENTS) {
        *slot = uc.reg_read(register)?;
    }
    match name {
        "_malloc" => allocate(uc, a[0]),
        "_free" => {
            let s = uc.get_data_mut();
            if let Some(size) = s.allocations.remove(&a[0]) {
                s.free.push((a[0], size));
            }
            Ok(0)
        }
        "___memset_chk" | "_memset" | "___bzero" | "_bzero" => {
            let (byte, len) = if name.ends_with("bzero") {
                (0, a[1])
            } else {
                (a[1] as u8, a[2])
            };
            if len > HEAP_SIZE || (name == "___memset_chk" && len > a[3]) {
                return Err(failure("Invalid memset size"));
            }
            uc.mem_write(a[0], &vec![byte; len as usize])?;
            Ok(a[0])
        }
        "_memcpy" | "_memmove" | "___memcpy_chk" => {
            if a[2] > HEAP_SIZE || (name == "___memcpy_chk" && a[2] > a[3]) {
                return Err(failure("Invalid memcpy size"));
            }
            let bytes = uc.mem_read_as_vec(a[1], a[2] as usize)?;
            uc.mem_write(a[0], &bytes)?;
            Ok(a[0])
        }
        "_sysctlbyname" => {
            let key = cstring(uc, a[0])?;
            let value = match key.as_str() {
                "kern.osversion" => Some(uc.get_data().hardware.os_build_num.clone()),
                "hw.model" => Some(uc.get_data().hardware.product_name.clone()),
                _ => None,
            };
            if let Some(value) = value {
                if a[2] != 0 {
                    let size = read_u64(uc, a[2])?;
                    let mut bytes = value.into_bytes();
                    bytes.push(0);
                    if a[1] != 0 {
                        uc.mem_write(a[1], &bytes[..bytes.len().min(size as usize)])?;
                    }
                    uc.mem_write(a[2], &(bytes.len() as u64).to_le_bytes())?;
                }
            }
            Ok(0)
        }
        "_IORegistryEntryFromPath" => Ok(1),
        "_IORegistryEntryCreateCFProperty" => {
            let key = cfstring(uc, a[1])?;
            let value = property(&uc.get_data().hardware, &key);
            if std::env::var_os("WINMESSAGES_NAC_TRACE").is_some() {
                eprintln!("IOKit {key}: {}", value.is_some());
            }
            Ok(value.map(|v| add_object(uc, v)).unwrap_or(0))
        }
        "_CFGetTypeID" => Ok(match object(uc, a[0])? {
            Object::Data(_) => 1,
            Object::String(_) => 2,
            Object::Dictionary(_) => 3,
        }),
        "_CFDataGetTypeID" => Ok(1),
        "_CFStringGetTypeID" => Ok(2),
        "_CFDataGetLength" => match object(uc, a[0])? {
            Object::Data(b) => Ok(b.len() as u64),
            _ => Err(failure("Expected CFData")),
        },
        "_CFDataGetBytes" => {
            if let Object::Data(bytes) = object(uc, a[0])? {
                let end = a[1]
                    .checked_add(a[2])
                    .ok_or_else(|| failure("CFRange overflow"))?;
                let bytes = bytes
                    .get(a[1] as usize..end as usize)
                    .ok_or_else(|| failure("Invalid CFRange"))?;
                uc.mem_write(a[3], bytes)?;
                Ok(0)
            } else {
                Err(failure("Expected CFData"))
            }
        }
        "_CFRelease" | "_IOObjectRelease" => Ok(0),
        "_statfs$INODE64" => Ok(0),
        "_DASessionCreate" => Ok(201),
        "_DADiskCreateFromBSDName" => Ok(202),
        "_DADiskCopyDescription" => {
            let uuid = uc.get_data().hardware.root_disk_uuid.clone();
            let value = add_object(uc, Object::String(uuid));
            Ok(add_object(
                uc,
                Object::Dictionary(HashMap::from([(0xc3c3c3c3c3c3c3c3, value)])),
            ))
        }
        "_CFDictionaryCreateMutable" => Ok(add_object(uc, Object::Dictionary(HashMap::new()))),
        "_CFDictionaryGetValue" => {
            if let Object::Dictionary(d) = object(uc, a[0])? {
                Ok(*d.get(&a[1]).unwrap_or(&0))
            } else {
                Err(failure("Expected CFDictionary"))
            }
        }
        "_CFDictionarySetValue" => {
            if let Some(Object::Dictionary(d)) = uc
                .get_data_mut()
                .objects
                .get_mut(a[0].wrapping_sub(1) as usize)
            {
                d.insert(a[1], a[2]);
                Ok(0)
            } else {
                Err(failure("Expected CFDictionary"))
            }
        }
        "_CFUUIDCreateString" => Ok(a[1]),
        "_CFStringGetLength" => Ok(cfstring(uc, a[0])?.encode_utf16().count() as u64),
        "_CFStringGetMaximumSizeForEncoding" => Ok(if a[1] == 0 {
            a[0]
        } else {
            a[0].saturating_mul(4)
        }),
        "_CFStringGetCString" => {
            let mut bytes = cfstring(uc, a[0])?.into_bytes();
            bytes.push(0);
            if bytes.len() as u64 > a[2] {
                return Ok(0);
            }
            uc.mem_write(a[1], &bytes)?;
            Ok(1)
        }
        "_IOServiceMatching" => Ok(add_object(uc, Object::Dictionary(HashMap::new()))),
        "_IOServiceGetMatchingService" => Ok(92),
        "_IOServiceGetMatchingServices" => {
            uc.get_data_mut().iterator = true;
            uc.mem_write(a[2], &93u32.to_le_bytes())?;
            Ok(0)
        }
        "_IOIteratorNext" => {
            let next = uc.get_data().iterator;
            uc.get_data_mut().iterator = false;
            Ok(if next { 94 } else { 0 })
        }
        "_IORegistryEntryGetParentEntry" => {
            uc.mem_write(a[2], &(a[0] as u32 + 100).to_le_bytes())?;
            Ok(0)
        }
        "_arc4random" => Ok(rand::random::<u32>() as u64),
        _ => Err(failure(format!("Unimplemented imported function {name}"))),
    }
}

fn allow_missing_encrypted_property(uc: &mut Machine<'_>) -> Result<(), AbsintheError> {
    let ptr = uc.reg_read(RegisterX86::RSI)?;
    let key = cfstring(uc, ptr)?;
    if [
        "Gq3489ugfi",
        "Fyp98tpgj",
        "kbjfrfpoJU",
        "oycqAZloTNDm",
        "abKPld1EcMni",
    ]
    .contains(&key.as_str())
        && property(&uc.get_data().hardware, &key).is_none()
    {
        uc.reg_write(RegisterX86::RDX, 0)?;
    }
    Ok(())
}

impl Emulator {
    pub fn new(hardware: HardwareConfig) -> Result<Self, AbsintheError> {
        let image = macho::load()?;
        let state = State {
            hardware,
            objects: Vec::new(),
            heap: HEAP,
            allocations: HashMap::new(),
            free: Vec::new(),
            iterator: false,
            fault: None,
        };
        let mut machine = Unicorn::new_with_data(Arch::X86, Mode::MODE_64, state)?;
        for segment in image.segments {
            if segment.size == 0 {
                continue;
            }
            let start = segment.address & !0xfff;
            let end = (segment.address + segment.size + 0xfff) & !0xfff;
            machine.mem_map(start, end - start, Prot::ALL)?;
            machine.mem_write(
                segment.address,
                &image.bytes[segment.offset..segment.offset + segment.file_size],
            )?;
        }
        machine.mem_unmap(0, 0x1000)?;
        machine.mem_map(STACK, STACK_SIZE, Prot::READ | Prot::WRITE)?;
        machine.mem_map(HEAP, HEAP_SIZE, Prot::READ | Prot::WRITE)?;
        machine.mem_map(STOP, 0x1000, Prot::ALL)?;
        machine.mem_map(HOOK, 0x1000, Prot::ALL)?;
        machine.mem_write(HOOK, &[0xc3; 0x1000])?;
        let mut resolved = HashMap::new();
        let mut names = Vec::new();
        for (location, name) in image.binds {
            let address = *resolved.entry(name.clone()).or_insert_with(|| {
                let address = HOOK + names.len() as u64 * 8;
                names.push(name);
                address
            });
            if address >= HOOK + 0x1000 {
                return Err(failure("Import table too large"));
            }
            machine.mem_write(location, &address.to_le_bytes())?;
        }
        machine.add_code_hook(HOOK, HOOK + 0xfff, move |uc, address, _| {
            let result = names
                .get(((address - HOOK) / 8) as usize)
                .ok_or_else(|| failure("Invalid import address"))
                .and_then(|name| dispatch(uc, name))
                .and_then(|value| uc.reg_write(RegisterX86::RAX, value).map_err(Into::into));
            if let Err(error) = result {
                uc.get_data_mut().fault = Some(error);
                let _ = uc.emu_stop();
            }
        })?;
        machine.add_code_hook(PROPERTY_READER, PROPERTY_READER, |uc, _, _| {
            if let Err(error) = allow_missing_encrypted_property(uc) {
                uc.get_data_mut().fault = Some(error);
                let _ = uc.emu_stop();
            }
        })?;
        Ok(Self { machine })
    }

    pub fn allocate(&mut self, size: u64) -> Result<u64, AbsintheError> {
        allocate(&mut self.machine, size)
    }

    pub fn bytes(&mut self, bytes: &[u8]) -> Result<u64, AbsintheError> {
        let ptr = self.allocate(bytes.len() as u64)?;
        self.machine.mem_write(ptr, bytes)?;
        Ok(ptr)
    }

    pub fn read_u64(&self, ptr: u64) -> Result<u64, AbsintheError> {
        read_u64(&self.machine, ptr)
    }

    pub fn output(&self, ptr: u64, len: u64) -> Result<Vec<u8>, AbsintheError> {
        let ptr = self.read_u64(ptr)?;
        let len = self.read_u64(len)?;
        if len > HEAP_SIZE {
            return Err(failure("Invalid output size"));
        }
        Ok(self.machine.mem_read_as_vec(ptr, len as usize)?)
    }

    pub fn call(&mut self, address: u64, args: &[u64]) -> Result<(), AbsintheError> {
        self.machine
            .reg_write(RegisterX86::RSP, STACK + STACK_SIZE - 8)?;
        self.machine
            .reg_write(RegisterX86::RBP, STACK + STACK_SIZE)?;
        self.machine
            .mem_write(STACK + STACK_SIZE - 8, &STOP.to_le_bytes())?;
        for (&arg, register) in args.iter().zip(ARGUMENTS) {
            self.machine.reg_write(register, arg)?;
        }
        let result = self.machine.emu_start(address, STOP, 120_000_000, 0);
        if let Some(error) = self.machine.get_data_mut().fault.take() {
            return Err(error);
        }
        if let Err(error) = result {
            eprintln!(
                "NAC entry {address:#x}, RIP {:#x}",
                self.machine.reg_read(RegisterX86::RIP).unwrap_or(0)
            );
            return Err(error.into());
        }
        if self.machine.reg_read(RegisterX86::RIP)? != STOP {
            return Err(failure("NAC emulation timed out"));
        }
        let status = self.machine.reg_read(RegisterX86::RAX)? as i32;
        if status != 0 {
            return Err(AbsintheError(status));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hardware() -> HardwareConfig {
        HardwareConfig {
            product_name: "Macmini7,1".into(),
            io_mac_address: [1, 2, 3, 4, 5, 6],
            platform_serial_number: "SERIAL".into(),
            platform_uuid: "UUID".into(),
            root_disk_uuid: "DISK".into(),
            board_id: "BOARD".into(),
            os_build_num: "23A344".into(),
            platform_serial_number_enc: Vec::new(),
            platform_uuid_enc: Vec::new(),
            root_disk_uuid_enc: Vec::new(),
            rom: vec![7, 8, 9, 10, 11, 12],
            rom_enc: Vec::new(),
            mlb: "MLB".into(),
            mlb_enc: Vec::new(),
        }
    }

    fn machine() -> Machine<'static> {
        let state = State {
            hardware: hardware(),
            objects: Vec::new(),
            heap: HEAP,
            allocations: HashMap::new(),
            free: Vec::new(),
            iterator: false,
            fault: None,
        };
        let mut uc = Unicorn::new_with_data(Arch::X86, Mode::MODE_64, state).unwrap();
        uc.mem_map(HEAP, HEAP_SIZE, Prot::READ | Prot::WRITE)
            .unwrap();
        uc
    }

    #[test]
    fn empty_encrypted_properties_are_null() {
        let mut uc = machine();
        for key in [
            "Gq3489ugfi",
            "Fyp98tpgj",
            "kbjfrfpoJU",
            "oycqAZloTNDm",
            "abKPld1EcMni",
        ] {
            let key = add_object(&mut uc, Object::String(key.into()));
            uc.reg_write(RegisterX86::RSI, key).unwrap();
            assert_eq!(
                dispatch(&mut uc, "_IORegistryEntryCreateCFProperty").unwrap(),
                0
            );
        }
        uc.get_data_mut().hardware.platform_serial_number_enc = vec![1, 2];
        let key = add_object(&mut uc, Object::String("Gq3489ugfi".into()));
        uc.reg_write(RegisterX86::RSI, key).unwrap();
        let id = dispatch(&mut uc, "_IORegistryEntryCreateCFProperty").unwrap();
        assert!(matches!(object(&uc, id).unwrap(), Object::Data(b) if b == [1, 2]));
    }

    #[test]
    fn compatibility_hook_only_relaxes_missing_encrypted_properties() {
        let mut uc = machine();
        for (key, required) in [
            ("Gq3489ugfi", 0),
            ("IOPlatformSerialNumber", 1),
            ("unknown", 1),
        ] {
            let key = add_object(&mut uc, Object::String(key.into()));
            uc.reg_write(RegisterX86::RSI, key).unwrap();
            uc.reg_write(RegisterX86::RDX, 1).unwrap();
            allow_missing_encrypted_property(&mut uc).unwrap();
            assert_eq!(uc.reg_read(RegisterX86::RDX).unwrap(), required);
        }
        uc.get_data_mut().hardware.platform_serial_number_enc = vec![1];
        let key = add_object(&mut uc, Object::String("Gq3489ugfi".into()));
        uc.reg_write(RegisterX86::RSI, key).unwrap();
        uc.reg_write(RegisterX86::RDX, 1).unwrap();
        allow_missing_encrypted_property(&mut uc).unwrap();
        assert_eq!(uc.reg_read(RegisterX86::RDX).unwrap(), 1);
    }

    #[test]
    fn hardware_properties_preserve_types_and_terminators() {
        let hw = hardware();
        assert!(
            matches!(property(&hw, "product-name"), Some(Object::Data(b)) if b == b"Macmini7,1\0")
        );
        assert!(
            matches!(property(&hw, "IOPlatformSerialNumber"), Some(Object::String(s)) if s == "SERIAL")
        );
        assert!(
            matches!(property(&hw, "IOMACAddress"), Some(Object::Data(b)) if b == [1, 2, 3, 4, 5, 6])
        );
        assert!(property(&hw, "unknown").is_none());
    }

    #[test]
    fn cfdata_range_is_start_and_length() {
        let mut uc = machine();
        let data = add_object(&mut uc, Object::Data(vec![10, 11, 12, 13, 14]));
        for (reg, val) in ARGUMENTS.into_iter().zip([data, 2, 2, HEAP]) {
            uc.reg_write(reg, val).unwrap();
        }
        dispatch(&mut uc, "_CFDataGetBytes").unwrap();
        assert_eq!(uc.mem_read_as_vec(HEAP, 2).unwrap(), [12, 13]);
        uc.reg_write(RegisterX86::RDX, 4).unwrap();
        assert!(dispatch(&mut uc, "_CFDataGetBytes").is_err());
    }

    #[test]
    fn allocations_are_aligned_reused_and_bounded() {
        let mut uc = machine();
        let first = allocate(&mut uc, 3).unwrap();
        let next = allocate(&mut uc, 1).unwrap();
        assert_eq!(next - first, 16);
        uc.reg_write(RegisterX86::RDI, first).unwrap();
        dispatch(&mut uc, "_free").unwrap();
        assert_eq!(allocate(&mut uc, 2).unwrap(), first);
        assert!(allocate(&mut uc, HEAP_SIZE + 1).is_err());
        assert!(allocate(&mut uc, u64::MAX).is_err());
    }

    #[test]
    fn cfstring_terminates_and_rejects_short_buffers() {
        let mut uc = machine();
        let id = add_object(&mut uc, Object::String("hello".into()));
        for (reg, val) in ARGUMENTS.into_iter().zip([id, HEAP, 6, 0]) {
            uc.reg_write(reg, val).unwrap();
        }
        assert_eq!(dispatch(&mut uc, "_CFStringGetCString").unwrap(), 1);
        assert_eq!(uc.mem_read_as_vec(HEAP, 6).unwrap(), b"hello\0");
        uc.reg_write(RegisterX86::RDX, 5).unwrap();
        assert_eq!(dispatch(&mut uc, "_CFStringGetCString").unwrap(), 0);
    }

    #[test]
    fn hardware_plist_roundtrip_keeps_binary_data() {
        let mut hw = hardware();
        hw.platform_uuid_enc = vec![0, 1, 255];
        let mut bytes = Vec::new();
        plist::to_writer_xml(&mut bytes, &hw).unwrap();
        let value = plist::Value::from_reader(std::io::Cursor::new(&bytes)).unwrap();
        let dict = value.as_dictionary().unwrap();
        assert_eq!(dict["io_mac_address"].as_data().unwrap(), hw.io_mac_address);
        assert_eq!(dict["platform_uuid_enc"].as_data().unwrap(), [0, 1, 255]);
        let restored: HardwareConfig = plist::from_bytes(&bytes).unwrap();
        assert_eq!(restored.platform_uuid_enc, hw.platform_uuid_enc);
        assert_eq!(restored.io_mac_address, hw.io_mac_address);
        assert_eq!(restored.product_name, hw.product_name);
    }

    #[test]
    fn contexts_do_not_share_objects_or_iterators() {
        let mut first = machine();
        let second = machine();
        add_object(&mut first, Object::String("first".into()));
        first.get_data_mut().iterator = true;
        assert!(second.get_data().objects.is_empty());
        assert!(!second.get_data().iterator);
    }
}
