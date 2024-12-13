# rankers

> [!WARNING]
> Very early stage.

*A service that provides different ranking methods for a wide range of social software.*

## Features

Current state:

- [Hacker News](https://news.ycombinator.com/)
- [Quality News](https://news.social-protocols.org/) (WIP)

## Setup for Development

If you're using `nix` and `direnv`, you can just navigate into this repository and run:

```
direnv allow
```

This will put you into a development shell that provides all the necessary dependencies for development.

Then, you can run the following to setup the dev environment:

```
just setup-dev-environment
```

## Development Workflows

Several workflows are documented in the `justfile`.
Run `just` to get an overview.

