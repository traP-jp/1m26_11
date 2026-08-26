use thiserror::Error;

#[derive(Debug, Error)]
#[error("asset URL could not be resolved")]
pub struct AssetUrlResolveError;

pub trait AssetUrlResolver: Send + Sync {
    fn resolve(&self, object_key: &str) -> Result<String, AssetUrlResolveError>;
}

pub(crate) struct UnconfiguredAssetUrlResolver;

impl AssetUrlResolver for UnconfiguredAssetUrlResolver {
    fn resolve(&self, _object_key: &str) -> Result<String, AssetUrlResolveError> {
        Err(AssetUrlResolveError)
    }
}
