//! Information about directory authorities
//!
//! From a client's point of view, an authority's role is to sign the
//! consensus directory.

use std::net::SocketAddr;

use derive_deftly::Deftly;
use getset::Getters;
use tor_config::define_list_builder_accessors;
use tor_config::derive::prelude::*;
use tor_llcrypto::pk::rsa::RsaIdentity;

/// The contact information for all directory authorities this implementation is
/// aware of.
///
/// This data structure makes use of proposal 330 in order to distinguish
/// authorities by their responsibilities, hence why the fields are divided.
#[derive(Debug, Clone, Deftly, Eq, PartialEq, Getters)]
#[derive_deftly(TorConfig)]
pub struct AuthorityContacts {
    // NOTE: We aren't using list builders here, since these never actually used sub-builders,
    // and therefore has a slightly different API.
    // We might later decide to change these to proper list-builders;
    // if so, it will be an API break.
    //
    /// The [`RsaIdentity`] keys that may be used to sign valid consensus documents.
    #[deftly(tor_config(no_magic, setter(skip), default = "default_v3idents()"))]
    #[getset(get = "pub")]
    v3idents: Vec<RsaIdentity>,

    /// The endpoints of authorities where upload of router descriptors and other
    /// documents is possible.
    ///
    /// This section is primarily of interest for relays.
    ///
    /// The use of nested a [`Vec`] serves the purpose to assign multiple IPs to
    /// a single logical authority, such as having an IPv4 and IPv6 address.
    #[deftly(tor_config(no_magic, setter(skip), default = "default_uploads()"))]
    #[getset(get = "pub")]
    uploads: Vec<Vec<SocketAddr>>,

    /// The endpoints of authorities where download of network documents is possible.
    ///
    /// This section is primarily of interest for directory mirrors.
    ///
    /// The use of nested a [`Vec`] serves the purpose to assign multiple IPs to
    /// a single logical authority, such as having an IPv4 and IPv6 address.
    #[deftly(tor_config(no_magic, setter(skip), default = "default_downloads()"))]
    #[getset(get = "pub")]
    downloads: Vec<Vec<SocketAddr>>,

    /// The endpoints of authorities where voting for consensus documents is possible.
    ///
    /// This section is primarily of interest for other directory authorities.
    ///
    /// The use of nested a [`Vec`] serves the purpose to assign multiple IPs to
    /// a single logical authority, such as having an IPv4 and IPv6 address.
    #[deftly(tor_config(no_magic, setter(skip), default = "default_votes()"))]
    #[getset(get = "pub")]
    votes: Vec<Vec<SocketAddr>>,
}

define_list_builder_accessors! {
    struct AuthorityContactsBuilder {
        pub v3idents: [RsaIdentity],
        pub uploads: [Vec<SocketAddr>],
        pub downloads: [Vec<SocketAddr>],
        pub votes: [Vec<SocketAddr>],
    }
}

/// Returns a list of the default [`RsaIdentity`]s for the directory authorities.
fn default_v3idents() -> Vec<RsaIdentity> {
    fn rsa(hex: &str) -> RsaIdentity {
        RsaIdentity::from_hex(hex).expect("invalid hex?!?")
    }

    vec![
        rsa("88F29CB5FE86A688E31990A3B20BD562D0C089E1"), // ATORDAeuclive
        rsa("9572A9D4141A2AFC43C416E978D805E7C98B3B25"), // ATORDAuselive
        rsa("FFBFD3D2E92EEAA9162335FF8DF259CC62713784"), // ATORDAuswlive
        rsa("6CE85CF74AB78E4D350E0418234B97F47AB32A20"), // AnyoneAshLive
        rsa("39C78145CFDF464E624626D4F78A315387132082"), // AnyoneHilLive
        rsa("5F18C895685A4207E0778FEB2A9CE4C90DABE7A6"), // AnyoneHelLive
        rsa("271F7D1592BF37AEB67BF48164928720EF9D0648"), // AnyoneFalLive
    ]
}

/// Returns a list of the [`SocketAddr`] for the directory authorities.
///
/// The nested [`Vec`] serves dual-stack purposes
/// (i.e. many IPs mapping to one logical authority).
fn default_uploads() -> Vec<Vec<SocketAddr>> {
    /// Converts a [`str`] to a [`SocketAddr`].
    fn sa(s: &str) -> SocketAddr {
        s.parse().expect("invalid socket address?!?")
    }

    vec![
        // ATORDAeuclive
        vec![sa("49.13.145.234:9230")],
        // ATORDAuselive
        vec![sa("5.161.108.187:9230")],
        // ATORDAuswlive
        vec![sa("5.78.90.106:9230")],
        // AnyoneAshLive
        vec![sa("5.161.228.187:9230")],
        // AnyoneHilLive
        vec![sa("5.78.94.15:9230")],
        // AnyoneHelLive
        vec![sa("95.216.32.105:9230")],
        // AnyoneFalLive
        vec![sa("176.9.29.53:9230")],
    ]
}

/// For now, an alias to [`default_uploads()`].
fn default_downloads() -> Vec<Vec<SocketAddr>> {
    default_uploads()
}

/// For now, an alias to [`default_uploads()`].
fn default_votes() -> Vec<Vec<SocketAddr>> {
    default_uploads()
}

#[cfg(test)]
mod test {
    // @@ begin test lint list maintained by maint/add_warning @@
    #![allow(clippy::bool_assert_comparison)]
    #![allow(clippy::clone_on_copy)]
    #![allow(clippy::dbg_macro)]
    #![allow(clippy::mixed_attributes_style)]
    #![allow(clippy::print_stderr)]
    #![allow(clippy::print_stdout)]
    #![allow(clippy::single_char_pattern)]
    #![allow(clippy::unwrap_used)]
    #![allow(clippy::unchecked_time_subtraction)]
    #![allow(clippy::useless_vec)]
    #![allow(clippy::needless_pass_by_value)]
    #![allow(clippy::string_slice)] // See arti#2571
    //! <!-- @@ end test lint list maintained by maint/add_warning @@ -->
    #![allow(clippy::unnecessary_wraps)]

    use super::*;

    #[test]
    fn default_auths() {
        let dflt = AuthorityContacts::builder().build().unwrap();
        assert_eq!(dflt.v3idents, default_v3idents());
        assert_eq!(dflt.uploads, default_uploads());
        assert_eq!(dflt.downloads, default_downloads());
        assert_eq!(dflt.votes, default_votes());

        assert_eq!(dflt.v3idents.len(), 7);
        assert_eq!(dflt.uploads.len(), 7);
        assert_eq!(
            dflt.v3idents[6],
            RsaIdentity::from_hex("271F7D1592BF37AEB67BF48164928720EF9D0648").unwrap()
        );
        assert_eq!(dflt.uploads[6], vec!["176.9.29.53:9230".parse().unwrap()]);
        assert_eq!(dflt.uploads[6], dflt.downloads[6]);
        assert_eq!(dflt.uploads[6], dflt.votes[6]);
    }
}
