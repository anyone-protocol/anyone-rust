# Anyone network

This fork builds Arti for the Anyone network instead of Tor. The network identity is compiled in,
so the client joins the Anyone network with no configuration.

## What differs from upstream

| File | Content |
| --- | --- |
| `crates/tor-dircommon/src/authority.rs` | The directory authorities: the v3 identities allowed to sign a consensus, and their directory addresses. |
| `crates/tor-dircommon/data/fallback_dirs.rs` | The fallback caches Arti downloads its first directory documents from. Generated. |
| `crates/tor-dirclient/src/request.rs` | The download size limit per microdescriptor and per router descriptor, raised from 8 KiB to 20 KiB. |

The Anon client ships an empty fallback list and bootstraps from the authorities, so the fallback
list here is the authorities themselves.

With only Anyone identities able to sign a consensus, a Tor consensus cannot validate, and the
client cannot fall back to the Tor network.

Relay families on the Anyone network are large, so their `family` lines make many descriptors
larger than 8 KiB. Upstream's limit truncates those downloads, and the client never learns about
the affected relays. The directory authorities reject any router descriptor over 20,000 bytes
(`MAX_DESCRIPTOR_UPLOAD_SIZE` in the Anon client), and a microdescriptor is smaller than the router
descriptor it comes from, so 20 KiB fits every descriptor the network accepts. If the authorities'
limit is raised, this limit has to be raised with it. A full batch of 500 digests is limited to
10 MiB.

## Regenerating the network identity

The authority list compiled into the Anon client, `src/app/config/auth_dirs.inc` in
[ator-protocol](https://github.com/anyone-protocol/ator-protocol), is the source of truth.

```
anyone/build-network-config.py <path to auth_dirs.inc> [--toml FILE] [--evidence DIR]
```

The script:

1. reads the authority list from `auth_dirs.inc`
2. fetches each authority's ed25519 identity, which `auth_dirs.inc` does not carry, and checks it
   three ways: the authority's own descriptor, the current consensus, and its microdescriptor
   served by two other authorities
3. checks that `authority.rs` lists the same identities and addresses in the same order
4. writes `fallback_dirs.rs` only if every check passes

`authority.rs` is edited by hand and checked by the script. No signature is verified by the
script. Arti verifies the consensus signatures at bootstrap, and a wrong fallback identity fails
closed because the channel handshake to that relay is rejected.

`--toml` also writes the identity as an Arti config file, for running an unmodified upstream Arti
against the Anyone network. `--evidence` keeps the fetched documents and a JSON summary.

Arti warns on an unrecognized config key and carries on with its defaults, so a misspelled key in
such a file leaves Tor's authorities in place. Check the log for `unrecognized configuration key`.

## Terms and conditions

Arti runs no command until the user has agreed to the Anyone terms and conditions at
https://anyone.io/terms, the same rule the Anon client applies. `--help` and `--version` work
without agreement. Agreement is any one of:

- `application.agree_to_terms = true` in the config, or `-o application.agree_to_terms=true`
- `arti proxy --agree-to-terms`, which also records the agreement in the state directory
- a `terms-agreement` file in the state directory from an earlier agreement
- answering `y` at the prompt, which is shown when standard input is a terminal

Without any of these and without a terminal, Arti exits with an error.

The `terms-agreement` file counts only if it contains exactly `agreed`, with no trailing newline,
and passes Arti's file permission checks. Arti logs a warning when it ignores the file. To agree
without the prompt, use the config option rather than writing the file.

The check runs before every command, so a command added upstream is covered with no change here.
The default build has only `proxy`. Upstream's tests for the optional `hss`, `hsc` and `keys`
commands do not agree to the terms, so they fail when those features are enabled.

| File | Content |
| --- | --- |
| `crates/arti/src/terms.rs` | The check, the prompt, and the agreement file. |
| `crates/arti/src/lib.rs` | The `--agree-to-terms` flag on `proxy`, and the check before any command runs. |
| `crates/arti/src/cfg.rs` | The `application.agree_to_terms` option. |
| `crates/arti/src/arti-example-config.toml` | The option in the example config. |
| `crates/arti-client/src/config.rs` | A public accessor for the state directory. |
| `crates/arti/tests/testcases/arti/terms-not-agreed.*` | A test that `arti proxy` refuses to start without agreement. |

## Branding

What the program prints carries the Anyone name, as the Anon client's output does. The source keeps
upstream's wording: `crates/arti/src/branding.rs` replaces the words on their way out, so messages
added or reworded upstream are covered with no change here.

It replaces the words `Tor` with `Anyone`, `a Tor` with `an Anyone`, and `Arti`, and `arti` as the
program name, with `anyone-rust`. A word that is part of a longer name is kept, such as
`Tor-Family-Preference`, `arti-client`, a path like `/usr/local/bin/arti`, or `C Tor`.

It applies to:

- console and file log lines, which also leave out the crate each message comes from, as the Anon
  client's do. Log filters still use crate names, such as `tor_dirmgr=debug`.
- help text, `--version`, and command-line errors
- the error printed when the program exits on a failure

The replacement covers everything printed, so a path, hostname, or relay nickname containing the
word `Tor` or `Arti` is shown with the replacement too.

Outside the output, the default folders are named `anyone-rust`, such as `~/.config/anyone-rust`
on Linux, and so are the config files, `anyone-rust.toml` and `anyone-rust.d/`. Crate names,
identifiers, the cargo binary name `arti`, protocol messages on the wire, documentation, and
copyright notices stay as upstream.

| File | Content |
| --- | --- |
| `crates/arti/src/branding.rs` | The replacement, the log writer, and the two exit paths. |
| `crates/arti/src/lib.rs` | The early log writer and the exit paths go through `branding.rs`. |
| `crates/arti/src/logging.rs` | The console and file log writers go through `branding.rs`, without crate names. |
| `crates/tor-config-path/src/arti_client_paths.rs` | The default folder name. |
| `crates/arti-client/src/config.rs` | The default config file names. |
| `crates/arti/tests/testcases/arti/help.stdout` | The expected `--help` output. |

## Container image

`.github/workflows/build-and-publish-image.yml` builds the `Dockerfile` on every push to `main`
and publishes `ghcr.io/anyone-protocol/anyone-rust:main` and `:<commit sha>`.

- `anyone-rust proxy` runs as the `anyone-rust` user, uid 10001, with SOCKS on `0.0.0.0:9150`.
- `XDG_CONFIG_HOME=/etc` makes `/etc/anyone-rust/anyone-rust.toml` and
  `/etc/anyone-rust/anyone-rust.d/` the default config, so it applies under any command, not only
  the default one.
- `/etc/anyone-rust/anyone-rust.toml` holds container settings only. Extra settings go in files
  mounted under `/etc/anyone-rust/anyone-rust.d/`.
- State and cache live on the `/var/lib/anyone-rust` volume.
- The image stops with SIGINT, because the program does not exit on SIGTERM.
- The image does not agree to the terms and conditions for the user. Run it as
  `proxy --agree-to-terms`, set the option in a mounted config file, or run it with `-it` to
  answer the prompt. With `-t` but not `-i` (in Compose, `tty: true` without
  `stdin_open: true`), it waits at the prompt indefinitely.

The build context is an allowlist in `.dockerignore`. The workspace has members under `examples/`
and `maint/`, and cargo will not load it without them, so those directories are copied along with
`crates/`.
