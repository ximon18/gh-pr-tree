# gh-pr-tree

Shows the hierarchical relationship between open pull requests in one or more GitHub repositories.

## tl;dr

> _*Requires that the GitHub CLI is already installed and configured!*_

```
cargo run -- <org>/<repo> [<org>/<repo>..]
```

## Introduction

This tool will query the GitHub API for details of the open pull requests of the specified repositories and serve the results as a website, one page per repository and an index page.

Repository pages will be served at `/<org>/<repo>`.

Each repository page will show a list of the open PR numbers, authors and titles, and, for PRs whose base branch is that of another open PR, a tree-like representation of how the related PRs will merge into each other.

## Configuration

Each GH repo is queried once a minute via the GitHub CLI which must be installed and configured on the same server where gh-pre-tree will run.

HTTP configuration can be adjusted per the instructions [here](https://rocket.rs/guide/v0.5/configuration/).
