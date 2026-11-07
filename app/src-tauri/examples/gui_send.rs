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

fn baked() -> MacOSConfig {
    let mac: [u8; 6] = b64("qGC2H1Am").try_into().unwrap();
    MacOSConfig {
        inner: HardwareConfig {
            product_name: "Macmini7,1".to_string(),
            io_mac_address: mac,
            platform_serial_number: "C07SW06DG1J1".to_string(),
            platform_uuid: "9CA28140-E736-59AB-9AC6-792E507D2184".to_string(),
            root_disk_uuid: "2CB26D1C-D73C-4F92-970B-F87341D4144E".to_string(),
            board_id: "Mac-35C5E08120C7EEAF".to_string(),
            os_build_num: "22G120".to_string(),
            platform_serial_number_enc: b64("zxsZESjSTtPw9SyBsELYoEM="),
            platform_uuid_enc: b64("N7rgxPQOa8kAoaUFzrgOoJI="),
            root_disk_uuid_enc: b64("RMD8Rrsnoq/nRCd5qG5pxvc="),
            rom: b64("qGC26soK"),
            rom_enc: b64("oygSjpinM48Z7vkz7Ttqaek="),
            mlb: String::from_utf8(b64("QzA3NjQ5NzAwN0ZHVjZYRFc=")).unwrap(),
            mlb_enc: b64("mqJmiIGO40uj6irTvezB4K4="),
        },
        version: "13.6.4".to_string(),
        protocol_version: 1660,
        device_id: Uuid::new_v4().to_string(),
        icloud_ua: "com.apple.iCloudHelper/282 CFNetwork/1408.0.4 Darwin/22.5.0".to_string(),
        aoskit_version: "com.apple.AOSKit/282 (com.apple.accountsd/113)".to_string(),
        udid: Some("55A1CFBF5BB56AD1159BD2CB7D6FF546E48EAAE4BF16188A07B1FB9C83138CA2".to_string()),
    }
}

#[tokio::main]
async fn main() {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    let _ = pretty_env_logger::try_init();

    let data = PathBuf::from(std::env::args().nth(1).expect("usage: gui_send <data_dir> <target> <text>"));
    let target = std::env::args().nth(2).expect("target");
    let text = std::env::args().nth(3).expect("text");

    let ks_path = data.join("keystore.plist");
    keystore::init_keystore(keystore::software::SoftwareKeystore {
        state: plist::from_file(&ks_path).unwrap_or_default(),
        update_state: Box::new(move |state| {
            let _ = plist::to_file_xml(&ks_path, state);
        }),
        encryptor: keystore::software::NoEncryptor,
    });

    let config = Arc::new(baked());
    let saved: SavedState = plist::from_file(data.join("registration.plist")).expect("no registration.plist");
    println!("loaded registration, reconnecting to apple push...");
    let (connection, _err) = APSConnectionResource::new(config.clone(), Some(saved.push)).await;

    let services = &[&MADRID_SERVICE, &MULTIPLEX_SERVICE, &FACETIME_SERVICE, &VIDEO_SERVICE];
    let client = IMClient::new(
        connection,
        saved.users,
        saved.identity,
        services,
        data.join("id_cache.plist"),
        config.clone(),
        Box::new(|_| {}),
    )
    .await;

    let handle = client.identity.get_handles().await[0].clone();
    println!("sending as {handle}");
    client
        .identity
        .ensure_private_self(&mut *client.identity.cache.lock().await, &handle, true)
        .await
        .ok();

    let mut msg = MessageInst::new(
        ConversationData {
            participants: vec![target.clone()],
            cv_name: None,
            sender_guid: Some(Uuid::new_v4().to_string()),
            after_guid: None,
        },
        &handle,
        Message::Message(NormalMessage::new(text, MessageType::IMessage)),
    );
    match client.send(&mut msg).await {
        Ok(_) => println!("SENT_OK to {target}"),
        Err(e) => println!("SEND_ERR {e:?}"),
    }
}
