use crate::adi_proxy::ADIProxyAnisetteProvider;
use crate::anisette_headers_provider::AnisetteHeadersProvider;
use crate::store_services_core::StoreServicesCoreADIProxy;
use crate::{AnisetteError, AnisetteProvider, LoginClientInfo};
use std::collections::HashMap;
use std::path::PathBuf;

pub struct LocalADIProvider {
    inner: ADIProxyAnisetteProvider<StoreServicesCoreADIProxy<'static>>,
}

unsafe impl Send for LocalADIProvider {}

impl LocalADIProvider {
    pub fn new(_info: LoginClientInfo, path: PathBuf) -> Result<LocalADIProvider, AnisetteError> {
        std::fs::create_dir_all(&path)?;
        let proxy = StoreServicesCoreADIProxy::new(&path)?;
        let inner = ADIProxyAnisetteProvider::new(proxy, path)?;
        Ok(LocalADIProvider { inner })
    }
}

impl AnisetteProvider for LocalADIProvider {
    fn get_anisette_headers(
        &mut self,
    ) -> impl std::future::Future<Output = Result<HashMap<String, String>, AnisetteError>> + Send
    {
        async move { self.inner.get_anisette_headers(false).await }
    }
}
