# arsd - AWS Role Speed Dial

A desktop app to manage sessions and roles for one or more AWS SSO domains.

## Feature Wishlist

- Support default heirarchy of roles (prefer readonly, then working, then ...)
- Allow tagging for environments or other account groupings
- Dump available roles to AWS config
- Support credential-process for CLI and SDKs such as CDK/Go SDK V1 that lack native SSO integration
- Support multiple SSO domains

## Usage

1. Download the `.dmg` from [Releases](https://github.com/ryansb/arsd/releases)
2. Drag `arsd.app` to `/Applications`
3. Control-click `arsd.app` and select "Open"
4. You will see an "Unidentified Developer" prompt. Select continue, or build the application locally
5. Follow the `Configuration` steps to add your SSO partition
6. Optionally, add aliases for accounts and roles to make them easier to remember

## Configuration

Configs are in the os-appropriate application configuration dir. On MacOS this is `/Users/your.name/Library/Application Support/io.rsb.arsd`. Under left-side menu, the config path is copyable if you don't already have a config file set up.

Below, find an example with one partition set up with an Amazon Web Services Identity and Access Management Identity Center (formerly known as AWS SSO). You will need your start URL, account ID, and region of the IAM Identity Center install.

```yaml
partitions:
- start_url: https://d-123abc.awsapps.com/start#
  region: us-west-2
  account_id: 999888777666
aliases:
  accounts:
    "awsadmin+centralbilling@example.zone": payer
  roles:
    Annoyingly-Long-Role-Name: Abbrev
```

# Development Environment

So far this has only ever been developed or tested on MacOS. Godspeed.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Vue](https://marketplace.visualstudio.com/items?itemName=Vue.volar) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

# Credits

[Awsume](https://awsu.me) for inspiring the open-in-console functionality.

Amazon Web Services service icons from [AWS](https://aws.amazon.com/architecture/icons/)
