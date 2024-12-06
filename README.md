# rankers

> [!WARNING]
> Early stage project, don't use in production yet.

*A service that provides different ranking methods for a wide range of social software.*

## Features

Implemented so far:

- [Hacker News](https://news.ycombinator.com/)
- [Quality News](https://news.social-protocols.org/)

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

