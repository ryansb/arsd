# arsd - AWS Role Speed Dial

A desktop app to manage sessions and roles for one or more AWS SSO domains.

## Configuration

Configs are in the os-appropriate application configuration dir. On MacOS this is `/Users/your.name/Library/Application Support/io.rsb.arsd`. The desktop app automatically creates a `config.yaml` file with base values.

Below, see an example with one partition set up with an Amazon Web Services Identity and Access Management Identity Center (formerly known as AWS SSO). You will need your start URL, account ID, and region of the IAM Identity Center install.

```yaml
partitions:
- start_url: https://yourcloud.awsapps.com/start#
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

- [VS Code](https://code.visualstudio.com/) + [Volar](https://marketplace.visualstudio.com/items?itemName=Vue.volar) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## Type Support For `.vue` Imports in TS

Since TypeScript cannot handle type information for `.vue` imports, they are shimmed to be a generic Vue component type by default. In most cases this is fine if you don't really care about component prop types outside of templates. However, if you wish to get actual prop types in `.vue` imports (for example to get props validation when using manual `h(...)` calls), you can enable Volar's Take Over mode by following these steps:

1. Run `Extensions: Show Built-in Extensions` from VS Code's command palette, look for `TypeScript and JavaScript Language Features`, then right click and select `Disable (Workspace)`. By default, Take Over mode will enable itself if the default TypeScript extension is disabled.
2. Reload the VS Code window by running `Developer: Reload Window` from the command palette.

You can learn more about Take Over mode [here](https://github.com/johnsoncodehk/volar/discussions/471).

# Credits

[Awsume](https://awsu.me) for inspiring the open-in-console functionality.

Amazon Web Services service icons from [AWS](https://aws.amazon.com/architecture/icons/)