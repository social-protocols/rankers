# Roadmap

## General

- [ ] create a model that estimates page coefficients (schedule can be more drawn out, eg daily)
- [ ] put upvote share by rank calculation on a scheduler (doesn't need to be recalculated ad hoc, estimation every now and then is enough)
- [ ] once data model is more stable, setup dev workflow with cargo watch
- [ ] create first client library
- [ ] setup configuration options -> if only certain rankings are required, some tasks don't need to be added to the scheduler (e.g., QN rankings require periodical sampling of stats)
- [ ] look into gRPC for generating clients

## Rankings

- [ ] come up with scheme to add other ranks if there are several pages
- [ ] come up with ranking endpoints that clearly distinguish between comment ranking and global ranking
- [ ] Reddit `hot` metric
- [ ] `global_brain` scoring

## API

- [ ] register posts after receiving first vote instead of having separate `create_post` endpoint -> this means that `submission_time` (and potentially some other information about the post) must be in the vote event
- [ ] `get_ranking/:method` endpoint (i.e., one endpoint for all rankings)
- [ ] `get_score` endpoint (requiring: `post_id`, `method`)
- [ ] sketch out documentation and guides

## CI/CD

- [ ] setup tests
- [ ] setup CI pipeline

## Deployment

- [ ] create docker image
- [ ] create a flake output for build

## Simulation

- [ ] use typescript to create simulation client, julia just doesn't cut it
- [ ] provide different probability models to simulate realistic vote streams (for different use cases / ranking methods)
