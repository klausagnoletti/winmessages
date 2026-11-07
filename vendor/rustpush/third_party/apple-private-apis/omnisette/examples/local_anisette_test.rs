#[cfg(feature = "local-adi")]
#[tokio::main(flavor = "current_thread")]
async fn main() {
    use omnisette::local_adi::LocalADIProvider;
    use omnisette::{AnisetteProvider, LoginClientInfo};
    use std::collections::HashMap;
    use std::path::PathBuf;

    let path = PathBuf::from(std::env::args().nth(1).expect("pass the config dir"));
    let info = LoginClientInfo {
        ak_context_type: String::new(),
        client_app_name: String::new(),
        client_bundle_id: String::new(),
        mme_client_info: String::new(),
        mme_client_info_akd: String::new(),
        akd_user_agent: String::new(),
        browser_user_agent: String::new(),
        hardware_headers: HashMap::new(),
        push_token: None,
        update_account_bundle_id: String::new(),
    };
    let mut provider = LocalADIProvider::new(info, path).expect("init");
    match provider.get_anisette_headers().await {
        Ok(headers) => {
            println!("ANISETTE_OK");
            let mut keys: Vec<_> = headers.keys().cloned().collect();
            keys.sort();
            for k in keys {
                println!("{k}: {}", headers.get(&k).unwrap());
            }
        }
        Err(e) => {
            println!("ANISETTE_FAILED {e:?}");
            std::process::exit(1);
        }
    }
}

#[cfg(not(feature = "local-adi"))]
fn main() {
    println!("build with --features local-adi");
}
