use open_absinthe::nac::{HardwareConfig, ValidationCtx};
use plist::Value;
use std::{error::Error, io::Cursor, time::Duration};

fn hardware() -> Result<HardwareConfig, Box<dyn Error>> {
    let value = Value::from_reader(Cursor::new(include_bytes!("data.plist")))?;
    let root = value.as_dictionary().ok_or("Expected dictionary")?;
    let io = root
        .get("iokit")
        .and_then(Value::as_dictionary)
        .ok_or("Missing iokit")?;
    let data = |key: &str| -> Result<Vec<u8>, Box<dyn Error>> {
        Ok(io
            .get(key)
            .and_then(Value::as_data)
            .ok_or_else(|| format!("Missing data {key}"))?
            .to_vec())
    };
    let string = |key: &str| -> Result<String, Box<dyn Error>> {
        match io.get(key).ok_or_else(|| format!("Missing string {key}"))? {
            Value::String(s) => Ok(s.clone()),
            Value::Data(b) => Ok(String::from_utf8(b.clone())?
                .trim_end_matches('\0')
                .to_owned()),
            _ => Err(format!("Unexpected type for {key}").into()),
        }
    };
    Ok(HardwareConfig {
        product_name: string("product-name")?,
        io_mac_address: data("IOMACAddress")?
            .try_into()
            .map_err(|_| "Invalid MAC")?,
        platform_serial_number: string("IOPlatformSerialNumber")?,
        platform_uuid: string("IOPlatformUUID")?,
        root_disk_uuid: root
            .get("root_disk_uuid")
            .and_then(Value::as_string)
            .ok_or("Missing root disk UUID")?
            .to_owned(),
        board_id: string("board-id")?,
        os_build_num: String::new(),
        platform_serial_number_enc: data("Gq3489ugfi")?,
        platform_uuid_enc: data("Fyp98tpgj")?,
        root_disk_uuid_enc: data("kbjfrfpoJU")?,
        rom: data("4D1EDE05-38C7-4A6A-9CC6-4BCCA8B38C14:ROM")?,
        rom_enc: data("oycqAZloTNDm")?,
        mlb: string("4D1EDE05-38C7-4A6A-9CC6-4BCCA8B38C14:MLB")?,
        mlb_enc: data("abKPld1EcMni")?,
    })
}

fn field(bytes: &[u8], key: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let value = Value::from_reader(Cursor::new(bytes))?;
    Ok(value
        .as_dictionary()
        .and_then(|d| d.get(key))
        .and_then(Value::as_data)
        .ok_or_else(|| format!("Response missing {key}"))?
        .to_vec())
}

fn run(
    client: &reqwest::blocking::Client,
    cert: &[u8],
    hw: &HardwareConfig,
    label: &str,
) -> Result<(), Box<dyn Error>> {
    let mut request = Vec::new();
    let mut context = ValidationCtx::new(cert, &mut request, hw)?;
    println!("{label}: nac_init request length = {}", request.len());
    let mut body = plist::Dictionary::new();
    body.insert("session-info-request".into(), Value::Data(request));
    let mut xml = Vec::new();
    Value::Dictionary(body).to_writer_xml(&mut xml)?;
    let response = client.post("https://identity.ess.apple.com/WebObjects/TDIdentityService.woa/wa/initializeValidation")
        .header("Content-Type", "application/x-apple-plist").body(xml).send()?.error_for_status()?.bytes()?;
    let session = field(&response, "session-info")?;
    println!("{label}: session-info length = {}", session.len());
    context.key_establishment(&session)?;
    let result = std::thread::spawn(move || context.sign())
        .join()
        .map_err(|_| "Signing thread failed")??;
    if result.is_empty() {
        return Err("Empty validation data".into());
    }
    let prefix = result
        .iter()
        .take(32)
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    println!(
        "{label}: validation data length = {}; first 32 bytes = {prefix}",
        result.len()
    );
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let client = reqwest::blocking::Client::builder()
        .add_root_certificate(reqwest::Certificate::from_der(include_bytes!(
            "AppleIncRootCertificate.cer"
        ))?)
        .timeout(Duration::from_secs(120))
        .build()?;
    let response = client
        .get("http://static.ess.apple.com/identity/validation/cert-1.0.plist")
        .send()?
        .error_for_status()?
        .bytes()?;
    let cert = field(&response, "cert")?;
    println!("Apple certificate length = {}", cert.len());
    let mut hw = hardware()?;
    run(&client, &cert, &hw, "original")?;
    hw.platform_serial_number_enc.clear();
    hw.platform_uuid_enc.clear();
    hw.root_disk_uuid_enc.clear();
    hw.rom_enc.clear();
    hw.mlb_enc.clear();
    run(&client, &cert, &hw, "empty encrypted fields")?;
    Ok(())
}
