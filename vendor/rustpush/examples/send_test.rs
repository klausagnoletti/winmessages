use base64::Engine as _;
use rustpush::macos::{HardwareConfig, MacOSConfig};
use rustpush::facetime::{FACETIME_SERVICE, VIDEO_SERVICE};
use rustpush::findmy::MULTIPLEX_SERVICE;
use rustpush::{
    authenticate_apple, default_provider, login_apple_delegates, register, APSConnectionResource,
    AppleAccount, ConversationData, IDSNGMIdentity, IMClient, LoginDelegate, LoginState, Message,
    MessageInst, MessageType, NormalMessage, OSConfig, MADRID_SERVICE,
};
use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

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

fn prompt(label: &str) -> String {
    print!("{label}");
    std::io::stdout().flush().unwrap();
    let mut s = String::new();
    std::io::stdin().read_line(&mut s).unwrap();
    s.trim().to_string()
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

    let email = std::env::args().nth(1).expect("usage: send_test <email> <anisette_dir>");
    let anisette = std::env::args().nth(2).expect("pass the anisette dir");
    let password = prompt("Apple ID password: ");
    let hashed = Sha256::digest(password.as_bytes()).to_vec();

    let config = Arc::new(baked());
    println!("connecting to apple push...");
    let (connection, _err) = APSConnectionResource::new(config.clone(), None).await;
    let gsa = config.get_gsa_config(&*connection.state.read().await, false);
    let anisette_client = default_provider(gsa.clone(), PathBuf::from(anisette));

    println!("logging in...");
    let mut account =
        AppleAccount::new_with_anisette(gsa, anisette_client.clone(), None, Box::new(|_| {}))
            .expect("account init failed");
    let mut state = account
        .login_email_pass(email.as_str(), &hashed)
        .await
        .expect("login_email_pass failed");

    loop {
        match state {
            LoginState::LoggedIn => break,
            LoginState::NeedsDevice2FA | LoginState::Needs2FAVerification | LoginState::NeedsSMS2FA => {
                println!();
                println!("Where should the 2FA code go?");
                println!("  1) Push to your trusted Apple devices (e.g. the t480s)");
                println!("  2) Text (SMS) to a trusted phone number");
                let choice = prompt("Choose 1 or 2: ");
                if choice == "2" {
                    let extras = account.get_auth_extras().await.expect("get_auth_extras failed");
                    if extras.trusted_phone_numbers.is_empty() {
                        println!("No trusted phone numbers on this account.");
                        return;
                    }
                    for (i, p) in extras.trusted_phone_numbers.iter().enumerate() {
                        println!("  [{}] {} (…{})", i + 1, p.number_with_dial_code, p.last_two_digits);
                    }
                    let pick = prompt("Which number? (enter the [number], or just press enter for the first): ");
                    let idx = pick.parse::<usize>().ok().map(|n| n.saturating_sub(1)).unwrap_or(0);
                    let phone = extras
                        .trusted_phone_numbers
                        .get(idx)
                        .unwrap_or(&extras.trusted_phone_numbers[0]);
                    let sent = account
                        .send_sms_2fa_to_devices(phone.id)
                        .await
                        .expect("send sms failed");
                    if let LoginState::NeedsSMS2FAVerification(body) = sent {
                        let code = prompt("Enter the SMS code: ");
                        state = account.verify_sms_2fa(code, body).await.expect("verify sms failed");
                    } else {
                        state = sent;
                    }
                } else {
                    account.send_2fa_to_devices().await.expect("send device 2fa failed");
                    let code = prompt("Enter the code from your Apple devices: ");
                    state = account.verify_2fa(code).await.expect("verify 2fa failed");
                }
            }
            LoginState::NeedsSMS2FAVerification(body) => {
                let code = prompt("Enter the SMS code: ");
                state = account.verify_sms_2fa(code, body).await.expect("verify sms failed");
            }
            other => {
                println!("unexpected login state: {other:?}");
                return;
            }
        }
    }
    println!("LOGGED IN. registering iMessage...");

    account
        .update_postdata("Apple Device", None, &["icloud", "imessage", "facetime"])
        .await
        .expect("update_postdata failed");

    let delegates = login_apple_delegates(
        &account,
        None,
        config.as_ref(),
        &[LoginDelegate::IDS, LoginDelegate::MobileMe],
    )
    .await
    .expect("delegates failed");

    let user = authenticate_apple(delegates.ids.expect("no ids delegate"), config.as_ref())
        .await
        .expect("authenticate_apple failed");
    let mut users = vec![user];

    let services = &[&MADRID_SERVICE, &MULTIPLEX_SERVICE, &FACETIME_SERVICE, &VIDEO_SERVICE];
    let identity = IDSNGMIdentity::new().expect("identity");

    register(
        config.as_ref(),
        &*connection.state.read().await,
        services,
        &mut users,
        &identity,
    )
    .await
    .expect("register failed");
    println!("REGISTERED.");

    let client = IMClient::new(
        connection.clone(),
        users,
        identity,
        services,
        "id_cache.plist".into(),
        config.clone(),
        Box::new(|_| {}),
    )
    .await;

    let handle = client.identity.get_handles().await[0].clone();
    println!("registered as handle: {handle}");
    client
        .identity
        .ensure_private_self(&mut *client.identity.cache.lock().await, &handle, true)
        .await
        .expect("ensure_private_self failed");

    let mut target = prompt("Send to (email, or tel:+15551234567): ");
    if !target.contains(':') {
        target = if target.starts_with('+') {
            format!("tel:{target}")
        } else {
            format!("mailto:{target}")
        };
    }
    let text = prompt("Message: ");

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
