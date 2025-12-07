# Security Policy

## Supported versions

At this stage of the project, only the **latest released version** of `jmssh` is actively supported.

If you discover a security issue in an older version, please check whether it still applies to the latest release before reporting.

---

## Reporting a vulnerability

If you believe you have found a security vulnerability in `jmssh`, please **do not create a public GitHub issue** with full details.

Instead, please use one of the following:

- Open a private security advisory via GitHub Security Advisories for this repository, or
- If that is not possible, open a minimal public issue and indicate that you would like to share details privately

When reporting, please try to include:

- The version of `jmssh` you are using
- Your OS and architecture
- A clear description of the potential impact
- Steps to reproduce, if possible

---

## What to expect

- We will acknowledge valid reports within a reasonable time frame
- We will investigate the issue and decide whether a fix and/or public advisory is needed
- If a fix is appropriate, it will be released in a new version and the issue will be mentioned in the changelog or release notes

Please note that `jmssh` is a local-first tool that runs on the user’s machine and builds on top of the system `ssh` client.  
It does not operate any central servers or store user credentials outside of the underlying OS credential store.