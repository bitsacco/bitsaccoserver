# Bitsacco Server - Contributing

We welcome your contributions to the Bitsacco OS project!

## Guidelines

1. Fork this repo to your github, then clone to your local
2. Create a new working branch. Usually `username/feature` when creating a new branch
3. Push to your fork, then create a PR to upstream
4. Ask for a review from `@okjodom` or any other project maintainer
5. Use [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/) guidelines for your commits.

## Project setup

```bash
yarn install
```

## Compile and run the project

### docker dev

- `yarn start` to start all the services in a docker compose environment
- `yarn stop` to shutdown all the services run via docker compose

## individual services

```bash
# general pattern to run an app
$ yarn dev <app>

# for example, to run the swap microservice
$ yarn dev swap

# to run the api gateway service
$ yarn dev api
```

## Run tests

```bash
# unit tests
$ yarn test

# target a specific test
$ yarn test <test-name-or-file-path>

# watch for changes and re-run tests
$ yarn test:watch
$ yarn test:watch <test-name-or-file-path>

# e2e tests
$ yarn test:e2e

# debug tests
$ yarn test:debug

# test coverage
$ yarn test:cov

```
