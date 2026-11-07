use base64::Engine as _;
use rustpush::macos::{HardwareConfig, MacOSConfig};
use rustpush::{default_provider, APSConnectionResource, AppleAccount, LoginState, OSConfig};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;

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
        device_id: uuid::Uuid::new_v4().to_string(),
        icloud_ua: "com.apple.iCloudHelper/282 CFNetwork/1408.0.4 Darwin/22.5.0".to_string(),
        aoskit_version: "com.apple.AOSKit/282 (com.apple.accountsd/113)".to_string(),
        udid: Some("55A1CFBF5BB56AD1159BD2CB7D6FF546E48EAAE4BF16188A07B1FB9C83138CA2".to_string()),
    }
}

#[tokio::main]
async fn main() {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    let _ = pretty_env_logger::try_init();
    keystore::init_keystore(keystore::software::SoftwareKeystore {
        state: plist::from_file("keystore.plist").unwrap_or_default(),
        update_state: Box::new(|state| {
            let _ = plist::to_file_xml("keystore.plist", state);
        }),
        encryptor: keystore::software::NoEncryptor,
    });

    let email = std::env::args().nth(1).expect("email");
    let anisette = std::env::args().nth(2).expect("anisette dir");
    let mut password = String::new();
    std::io::stdin().read_to_string(&mut password).unwrap();
    let password = password.trim();
    let hashed = Sha256::digest(password.as_bytes()).to_vec();

    let config = Arc::new(baked());
    let (connection, _err) = APSConnectionResource::new(config.clone(), None).await;
    let client_info = config.get_gsa_config(&*connection.state.read().await, false);
    let aprov = default_provider(client_info.clone(), PathBuf::from(anisette));
    let mut account = match AppleAccount::new_with_anisette(client_info, aprov, None, Box::new(|_| {})) {
        Ok(a) => a,
        Err(e) => {
            println!("PROBE_RESULT: ACCOUNT_ERR {e:?}");
            return;
        }
    };

    match account.login_email_pass(email.as_str(), &hashed).await {
        Ok(state) => match state {
            LoginState::LoggedIn => println!("PROBE_RESULT: LOGGED_IN"),
            LoginState::NeedsDevice2FA
            | LoginState::Needs2FAVerification
            | LoginState::NeedsSMS2FA
            | LoginState::NeedsSMS2FAVerification(_) => {
                println!("PROBE_RESULT: AUTH_OK_NEEDS_2FA ({state:?})")
            }
            other => println!("PROBE_RESULT: OTHER_STATE {other:?}"),
        },
        Err(e) => println!("PROBE_RESULT: LOGIN_ERR {e:?}"),
    }
}
