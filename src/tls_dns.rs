use std::net::SocketAddr;

use hickory_resolver::config::{ResolverConfig, ResolverOpts};
use hickory_resolver::TokioAsyncResolver;
use hyper::client::connect::dns::Name;
use reqwest::dns::{Addrs, Resolve, Resolving};
use tracing::{debug, debug_span, Instrument};

#[derive(Debug, Clone)]
pub struct TlsDns {
    resolver: TokioAsyncResolver,
}

impl TlsDns {
    pub fn new() -> Self {
        let resolver =
            TokioAsyncResolver::tokio(ResolverConfig::cloudflare_tls(), ResolverOpts::default());

        Self { resolver }
    }
}

impl Resolve for TlsDns {
    fn resolve(&self, name: Name) -> Resolving {
        let span = debug_span!("TlsDns::resolve", %name);

        let resolver = self.resolver.clone();

        Box::pin(
            async move {
                let ip = resolver.lookup_ip(name.as_str()).await?;

                debug!(?ip, "dns lookup done");

                Ok(Box::new(ip.into_iter().map(|ip| SocketAddr::new(ip, 0))) as Addrs)
            }
            .instrument(span),
        )
    }
}
