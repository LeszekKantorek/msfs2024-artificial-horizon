//! Typed startup configuration; validation also applies to non-CLI callers.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Source {
    Demo,
    SimConnect,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub source: Source,
    pub listen_address: IpAddr,
    pub port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            source: Source::Demo,
            listen_address: Ipv4Addr::LOCALHOST.into(),
            port: 8080,
        }
    }
}

impl Config {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.port == 0 {
            return Err(ConfigError::InvalidPort);
        }
        if self.listen_address.is_multicast()
            || self.listen_address == IpAddr::V4(Ipv4Addr::BROADCAST)
        {
            return Err(ConfigError::InvalidAddress(self.listen_address));
        }
        crate::providers::validate_source(self.source)
    }

    pub fn socket_address(&self) -> SocketAddr {
        SocketAddr::new(self.listen_address, self.port)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("port must be between 1 and 65535; port 0 is not a supported configuration")]
    InvalidPort,
    #[error(
        "listen address {0} is multicast or broadcast; choose a local interface or wildcard address"
    )]
    InvalidAddress(IpAddr),
    #[error(
        "SimConnect integration is not implemented yet (issues #5 and #6); explicitly select --source demo to start the HTTP skeleton"
    )]
    SimConnectUnavailable,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_and_explicit_lan_configurations_are_valid() {
        let default = Config::default();
        assert_eq!(default.source, Source::Demo);
        assert_eq!(default.socket_address(), "127.0.0.1:8080".parse().unwrap());
        default.validate().unwrap();
        for address in ["0.0.0.0", "192.168.1.20", "::", "::1"] {
            Config {
                listen_address: address.parse().unwrap(),
                ..default.clone()
            }
            .validate()
            .unwrap();
        }
    }

    #[test]
    fn rejects_invalid_configuration_without_cli() {
        assert!(matches!(
            Config {
                port: 0,
                ..Config::default()
            }
            .validate(),
            Err(ConfigError::InvalidPort)
        ));
        for address in ["224.0.0.1", "255.255.255.255", "ff02::1"] {
            assert!(matches!(
                Config {
                    listen_address: address.parse().unwrap(),
                    ..Config::default()
                }
                .validate(),
                Err(ConfigError::InvalidAddress(_))
            ));
        }
        assert!(matches!(
            Config {
                source: Source::SimConnect,
                ..Config::default()
            }
            .validate(),
            Err(ConfigError::SimConnectUnavailable)
        ));
    }
}
