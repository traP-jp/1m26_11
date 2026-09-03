use thiserror::Error;

#[derive(Debug, Error)]
#[error("asset URL could not be resolved")]
pub struct AssetUrlResolveError;

pub trait AssetUrlResolver: Send + Sync {
    fn resolve(&self, object_key: &str) -> Result<String, AssetUrlResolveError>;
}

pub(crate) struct UnconfiguredAssetUrlResolver;

pub struct PublicBaseAssetUrlResolver {
    base_url: String,
}

impl PublicBaseAssetUrlResolver {
    #[must_use]
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_owned(),
        }
    }
}

impl AssetUrlResolver for PublicBaseAssetUrlResolver {
    fn resolve(&self, object_key: &str) -> Result<String, AssetUrlResolveError> {
        Ok(format!(
            "{}/{}",
            self.base_url,
            object_key.trim_start_matches('/')
        ))
    }
}

impl AssetUrlResolver for UnconfiguredAssetUrlResolver {
    fn resolve(&self, _object_key: &str) -> Result<String, AssetUrlResolveError> {
        Err(AssetUrlResolveError)
    }
}

#[cfg(test)]
mod tests {
    use super::{AssetUrlResolver, PublicBaseAssetUrlResolver};

    #[test]
    fn joins_public_base_url_and_object_key() {
        let resolver = PublicBaseAssetUrlResolver::new("https://storage.example/assets/");

        let url = resolver
            .resolve("/v1/problems/example.png")
            .expect("public URL should be resolved");

        assert_eq!(
            url,
            "https://storage.example/assets/v1/problems/example.png"
        );
    }
}
