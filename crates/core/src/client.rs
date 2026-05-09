use super::{
    actor::{ActorProfileDetailed, ActorRepoInfo, DidDocument, RepoDescription},
    config::ServiceConfig,
};
use reqwest::Url;
use serde::de::DeserializeOwned;

#[derive(Debug, Clone)]
pub struct AtprotoClient {
    http: reqwest::Client,
    public_api_base: Url,
    plc_directory_base: Url,
}

impl AtprotoClient {
    pub fn new(services: ServiceConfig) -> Result<Self, ClientError> {
        Ok(Self {
            http: reqwest::Client::new(),
            public_api_base: parse_base_url("public_api_base", &services.public_api_base)?,
            plc_directory_base: parse_base_url("plc_directory_base", &services.plc_directory_base)?,
        })
    }

    pub async fn actor_repo_info(&self, actor: &str) -> Result<ActorRepoInfo, ClientError> {
        let profile = self.get_profile(actor).await?;
        let repo = self.describe_repo(&profile.did).await?;
        Ok(ActorRepoInfo { profile, repo })
    }

    pub async fn get_profile(&self, actor: &str) -> Result<ActorProfileDetailed, ClientError> {
        let url = self.xrpc_url(&self.public_api_base, "app.bsky.actor.getProfile")?;
        self.get_json(url, &[("actor", actor)]).await
    }

    pub async fn describe_repo(&self, repo: &str) -> Result<RepoDescription, ClientError> {
        let did_doc = self.resolve_did_document(repo).await?;
        let pds_base = pds_endpoint(&did_doc)?;
        let url = self.xrpc_url(&pds_base, "com.atproto.repo.describeRepo")?;
        self.get_json(url, &[("repo", repo)]).await
    }

    pub async fn resolve_did_document(&self, did: &str) -> Result<DidDocument, ClientError> {
        if did.starts_with("did:plc:") {
            let url = self
                .plc_directory_base
                .join(&format!("/{did}"))
                .map_err(|source| ClientError::Url { field: "did", value: did.to_string(), source })?;
            return self.get_json(url, &[]).await;
        }

        if let Some(domain) = did.strip_prefix("did:web:") {
            let mut parts = domain.split(':');
            let Some(host) = parts.next() else {
                return Err(ClientError::UnsupportedDid(did.to_string()));
            };

            let path_segments = parts.collect::<Vec<_>>();
            let did_path = if path_segments.is_empty() {
                ".well-known/did.json".to_string()
            } else {
                format!("{}/did.json", path_segments.join("/"))
            };

            let url = Url::parse(&format!("https://{host}/{did_path}")).map_err(|source| ClientError::Url {
                field: "did",
                value: did.to_string(),
                source,
            })?;
            return self.get_json(url, &[]).await;
        }

        Err(ClientError::UnsupportedDid(did.to_string()))
    }

    async fn get_json<T>(&self, url: Url, query: &[(&str, &str)]) -> Result<T, ClientError>
    where
        T: DeserializeOwned,
    {
        let response = self.http.get(url).query(query).send().await?.error_for_status()?;

        response.json::<T>().await.map_err(ClientError::from)
    }

    fn xrpc_url(&self, base: &Url, method: &str) -> Result<Url, ClientError> {
        base.join(&format!("/xrpc/{method}"))
            .map_err(|source| ClientError::Url { field: "xrpc method", value: method.to_string(), source })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("invalid {field} URL {value:?}: {source}")]
    Url {
        field: &'static str,
        value: String,
        source: url::ParseError,
    },
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    #[error("unsupported DID method for repository lookup: {0}")]
    UnsupportedDid(String),
    #[error("DID document for {0} does not advertise an AtprotoPersonalDataServer service")]
    MissingPdsService(String),
}

fn parse_base_url(field: &'static str, value: &str) -> Result<Url, ClientError> {
    Url::parse(value).map_err(|source| ClientError::Url { field, value: value.to_string(), source })
}

fn pds_endpoint(did_doc: &DidDocument) -> Result<Url, ClientError> {
    let endpoint = did_doc
        .service
        .iter()
        .find(|service| service.id == "#atproto_pds" || service.kind == "AtprotoPersonalDataServer")
        .ok_or_else(|| ClientError::MissingPdsService(did_doc.id.clone()))?;

    parse_base_url("serviceEndpoint", &endpoint.service_endpoint)
}
