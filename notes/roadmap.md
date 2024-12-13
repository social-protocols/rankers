# Roadmap

## RAM

- [ ] iteration on error handling
    - [ ] middleware layer to log requests
    - [ ] find unhandled edge cases

## General

- [ ] once data model is more stable, setup dev workflow with cargo watch
- [ ] setup configuration options -> if only certain rankings are required, some tasks don't need to be added to the scheduler (e.g., QN rankings require periodical sampling of stats)

## Simulation

- [ ] provide different probability models to simulate realistic vote streams (for different use cases / ranking methods)

## Quality News Upvote Share Prediction

- [ ] put upvote share by rank calculation on a scheduler (doesn't need to be recalculated ad hoc, estimation every now and then is enough)
- [ ] create a model that estimates page coefficients (schedule can be more drawn out, eg daily)
- [ ] should the data be stored in another db? could help performance since 1500 items need to be retrieved and saved in every iteration

## Rankings

- [ ] come up with scheme to add other ranks if there are several pages
- [ ] come up with ranking endpoints that clearly distinguish between comment ranking and global ranking
- [ ] Reddit `hot` metric
- [ ] `global_brain` scoring

## API

- [ ] sketch out documentation and guides

## CI/CD

- [ ] setup tests
- [ ] setup CI pipeline

## Deployment

- [ ] setup Earthfile for ci and deployment
- [ ] create docker image
- [ ] create a flake output for build

## Clients

- [ ] look into gRPC for generating clients
- [ ] create first client library
