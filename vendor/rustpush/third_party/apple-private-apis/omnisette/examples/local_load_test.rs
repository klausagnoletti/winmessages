#[cfg(feature = "local-adi")]
fn main() {
    use omnisette::adi_proxy::ADIProxy;
    use omnisette::store_services_core::StoreServicesCoreADIProxy;
    let path = std::path::PathBuf::from(std::env::args().nth(1).expect("pass the config dir"));
    match StoreServicesCoreADIProxy::new(&path) {
        Ok(proxy) => {
            println!("LOADED_OK serial={} device={}", proxy.get_serial_number(), proxy.get_device_identifier());
        }
        Err(e) => {
            println!("LOAD_FAILED {e:?}");
            std::process::exit(1);
        }
    }
}

#[cfg(not(feature = "local-adi"))]
fn main() {
    println!("build with --features local-adi");
}
