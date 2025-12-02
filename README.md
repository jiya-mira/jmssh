# jmssh

`jmssh` is a local-first SSH profile manager and connection helper.

It lives on top of your existing `ssh` and focuses on:
+ Security – passwords only live in the OS credential store
+ Simplicity – a small CLI instead of a heavy terminal suite
+ Local-first – no cloud, no telemetry, no background daemon

If you are searching for alternatives around `Xshell`, `Termius`, `SecureCRT`, `MobaXterm`, `Tabby` etc., but prefer plain OpenSSH + terminal + a thin helper, `jmssh` is for you.

---

## Design goals

### Secure by default
+ SSH passwords are never stored in the `jmssh` database.
+ Secrets are stored only in OS credential vaults (macOS Keychain, Windows Credential Manager, Linux Secret Service / keyutils, etc.).
+ All connections are executed via the system `ssh` binary.

### Simple mental model
+ A profile is: `label` + `host` + `user` + `port` + `auth_mode`.
+ Connecting is: `jmssh connect <label>`.
+ No custom SSH implementation, no hidden agents.

### Local-first, product-oriented
+ Single binary, no services to run, no browser required.
+ Future GUI / tooling builds on the same core instead of replacing it.

---

## Installation

Current distribution: prebuilt binaries from GitHub Releases.
1. Open the Releases page, for example:
`https://github.com/OWNER/REPO/releases`
2. Download the binary for your platform
(e.g. `jmssh-aarch64-apple-darwin`, `jmssh-x86_64-unknown-linux-gnu`, Windows build, etc.).
3. Put it somewhere on your `PATH` and make it executable if needed
(e.g. move to `~/bin` or `/usr/local/bin` and run `chmod +x jmssh` on Unix-like systems).

After that, you can run `jmssh` directly in your terminal.

---

## Quick start

### 1. Initialize local data

Run once per machine:

```bash
jmssh init
```

This creates the local SQLite database and runtime files under your user config directory.

### 2. Create a profile

#### Example:

```bash
jmssh profile add prod-web --host=example.com --user=ubuntu --port=22 --mode=auto
```

#### Key points:
+ `prod-web` – human-readable profile label
+ `example.com` – hostname or IP
+ `--user` – SSH user (default is `root` when creating)
+ `--port` – SSH port (default `22` when creating)
+ `--mode` – authentication mode:
  - `auto` – use ssh agent / default `ssh` behaviour
  - `password` – OS keyring + optional `sshpass` (Unix)
  - `key` – explicit private key (reserved for richer key support in future)

#### Useful profile commands:
+ `jmssh profile list` – list all profiles
+ `jmssh profile show prod-web` – show one profile
+ `jmssh profile set prod-web --user=ubuntu -–mode=password` – update in place

Only fields you pass are changed; the rest stay as they are.

### 3. Store a password (optional)

For `--mode=password` you usually store the password once:

```bash
jmssh password set prod-web
```

#### You’ll see:

```bash
password for 'prod-web':
```

The password is read with echo disabled and stored in the OS credential store.
`jmssh` never keeps its own plaintext copy.

#### Later:
+ `jmssh password show prod-web` – print the stored password
+ `jmssh password clear prod-web` – remove it from the credential store

If no password is stored, `jmssh` just runs `ssh` and lets it ask for the password as usual.

### 4. Connect

#### Everyday usage:

```bash
jmssh connect prod-web
```

`jmssh` resolves the profile, prints a short colorized summary (profile label, `user@host:port`, `auth_mode`), then hands control over to the system `ssh` process.
Exit codes follow `ssh`, so you can script around `jmssh connect` just like you would with `ssh`.

---

## About sshpass (Unix, for password mode)

When `auth_mode=password` and a password is stored in the OS keyring:
+ On Unix-like systems, if `sshpass` is available, `jmssh` uses it to automatically feed the password to `ssh`.
+ If `sshpass` is not available, `jmssh` falls back to plain `ssh` and you type the password manually.

Typical installation hints (adjust for your environment):
+ Debian / Ubuntu:
```bash
sudo apt install sshpass
```
+ Fedora:
```bash
sudo dnf install sshpass
```
+ Arch Linux:
```bash
sudo pacman -S sshpass
```
+ macOS: use your preferred package manager or build from source
(for example via a third-party Homebrew tap, MacPorts, or your internal repo).

`jmssh` will automatically detect if `sshpass` is missing and print a short message before falling back to normal `ssh` behaviour.
Password storage in the OS keyring works independently of `sshpass` – you can always use:
+ `jmssh password set/show/clear`

even if you never install `sshpass`.

---

## Who is jmssh for?

`jmssh` is a good fit if you:
+ live in the terminal and prefer OpenSSH + CLI over heavy GUI clients
+ maintain multiple servers and want clean, named profiles instead of long `ssh` commands
+ care about security and want passwords handled only by the OS credential store
+ manage SSH access in small teams and need something easy to explain and audit
+ work in constrained or controlled environments, for example:
  + headless / no-GUI servers
  + locked-down corporate or government environments
  + “信创” / compliance-driven setups where tools must be simple, local, and transparent

If you need a full SSH GUI (tabs, file browser, port manager, etc.), `jmssh` is not a replacement – it is a small, script-friendly companion focused on profiles and auth.

---

## Current feature set (v0.1.x)
+ Local profile management
  + `jmssh profile add / set / rm / show / list`
+ Simple connect command
  + `jmssh connect `
+ Password handling via OS credential store
  + `jmssh password set / show / clear `
+ Optional password autofill on Unix using `sshpass`
+ Colorful, compact CLI output
  + consistently shows profile label
  + highlights `user@host:port`
  + indicates active `auth_mode`

More advanced capabilities (like multi-hop jump chains) already exist internally but are deliberately kept out of this first page to keep the initial experience lightweight.

---

## Roadmap and feedback

### Likely directions (driven by real-world usage):
+ Package manager integration: `brew`, `winget`, AUR, etc.
+ A small browser-based GUI via `jmssh gui` for managing profiles and auth
+ Better Windows password automation (ConPTY-based)
+ Richer key handling and per-profile key selection
+ Tagging, grouping, searching profiles for larger fleets

### You can influence the roadmap by:
+ opening issues for features or UX improvements
+ voting / discussing on existing issues
+ contributing wording improvements for CLI help, logs, and docs

---

## License

This project is licensed under the MIT License.

In practical terms:
+ You may freely use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of this software.
+ You must include the original copyright and license notice in any substantial portion of the software.
+ The software is provided “as is”, without any warranty of any kind, express or implied.
+ The authors and contributors are not liable for any claim, damages, or other liability arising from the use of this software.

For the exact legal text, see the `LICENSE` file in the repository.

---

## SEO / discoverability keywords

`ssh client`, `ssh profile manager`, `ssh connection manager`, `ssh password manager`,
`jump host`, `bastion host`, `terminal ssh tool`, `devops`, `sre`, `sysadmin`,
`local-first`, `credential store`, `keychain`, `credential manager`,
`sshpass`, `Xshell`, `Termius`, `SecureCRT`, `MobaXterm`, `Tabby`.