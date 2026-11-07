use base64::Engine as _;
use rustpush::facetime::{FACETIME_SERVICE, VIDEO_SERVICE};
use rustpush::findmy::MULTIPLEX_SERVICE;
use rustpush::macos::{HardwareConfig, MacOSConfig};
use rustpush::{
    APSConnectionResource, APSState, ConversationData, IDSNGMIdentity, IDSUser, IMClient, Message,
    MessageInst, MessageType, NormalMessage, MADRID_SERVICE,
};
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize)]
struct SavedState {
    push: APSState,
    identity: IDSNGMIdentity,
    users: Vec<IDSUser>,
    #[allow(dead_code)]
    apple_id: String,
}

fn b64(s: &str) -> Vec<u8> {
    base64::engine::general_purpose::STANDARD.decode(s).unwrap()
}

fn fresh() -> MacOSConfig {
    let mac: [u8; 6] = b64("8BiYGS+5").try_into().unwrap();
    MacOSConfig {
        inner: HardwareConfig {
            product_name: "Macmini7,1".to_string(),
            io_mac_address: mac,
            platform_serial_number: "C074I2C2G1J1".to_string(),
            platform_uuid: "0A2E9382-D51C-4400-AEFB-1B19B6D5EE14".to_string(),
            root_disk_uuid: "F8C0D135-3F3C-40E5-B199-D5FF21D8F4B3".to_string(),
            board_id: "Mac-35C5E08120C7EEAF".to_string(),
            os_build_num: "22G120".to_string(),
            platform_serial_number_enc: b64("4lelmafcRQrXVcLPm9GvMRw="),
            platform_uuid_enc: b64("HIKkWRTCX+4XZlTVL/P/Nf4="),
            root_disk_uuid_enc: b64("ljmdTna7NJMWlrfSCEJxSIQ="),
            rom: b64("8BiYGS+5"),
            rom_enc: b64("sNBW72jXL8UDlCeTwURdA2E="),
            mlb: "C0244179903Z8PVQY".to_string(),
            mlb_enc: b64("hyFQDhdmfJxWU6bcxWEyxXM="),
        },
        version: "13.6.4".to_string(),
        protocol_version: 1660,
        device_id: Uuid::new_v4().to_string(),
        icloud_ua: "com.apple.iCloudHelper/282 CFNetwork/1408.0.4 Darwin/22.5.0".to_string(),
        aoskit_version: "com.apple.AOSKit/282 (com.apple.accountsd/113)".to_string(),
        udid: Some(Uuid::new_v4().to_string().to_uppercase().replace('-', "")),
    }
}

#[tokio::main]
async fn main() {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    let _ = pretty_env_logger::try_init();

    let data = PathBuf::from(std::env::args().nth(1).expect("usage: fresh_probe <data_dir> <target>"));
    let target = std::env::args().nth(2).expect("target");

    let ks_path = data.join("keystore.plist");
    keystore::init_keystore(keystore::software::SoftwareKeystore {
        state: plist::from_file(&ks_path).unwrap_or_default(),
        update_state: Box::new(move |state| {
            let _ = plist::to_file_xml(&ks_path, state);
        }),
        encryptor: keystore::software::NoEncryptor,
    });

    let config = Arc::new(fresh());
    let saved: SavedState = plist::from_file(data.join("registration.plist")).expect("no registration.plist");
    println!("FRESH: new serial {} — FRESH apns activation (new push cert for new hardware)...", config.inner.platform_serial_number);
    let _ = saved.push;
    let (connection, _err) = APSConnectionResource::new(config.clone(), None).await;

    let services = &[&MADRID_SERVICE, &MULTIPLEX_SERVICE, &FACETIME_SERVICE, &VIDEO_SERVICE];
    let client = IMClient::new(
        connection,
        saved.users,
        saved.identity,
        services,
        data.join("fresh_id_cache.plist"),
        config.clone(),
        Box::new(|_| {}),
    )
    .await;

    println!("FRESH: forcing re-registration with the new hardware identity...");
    match client.identity.refresh_now().await {
        Ok(_) => println!("FRESH: REREG_OK"),
        Err(e) => println!("FRESH: REREG_ERR {e:?}"),
    }

    let handle = client.identity.get_handles().await[0].clone();
    println!("FRESH: handles = {:?}", client.identity.get_handles().await);
    println!("FRESH: sending as {handle} to {target}");

    let mut msg = MessageInst::new(
        ConversationData {
            participants: vec![target.clone()],
            cv_name: None,
            sender_guid: Some(Uuid::new_v4().to_string()),
            after_guid: None,
        },
        &handle,
        Message::Message(NormalMessage::new("fresh identity test".to_string(), MessageType::IMessage)),
    );
    match client.send(&mut msg).await {
        Ok(_) => println!("FRESH: SENT_OK to {target}"),
        Err(e) => println!("FRESH: SEND_ERR {e:?}"),
    }
}
