## Kubernetes

This structure uses Kustomize to organize multiple Kubernetes environments.

The `base` folder contains what should be common to all environments.

Each environment should have a folder that references the `base` config, and then
a set of overlays that are specific to the environment.

To run a specific environment, execute:

`kubectl apply -k ENVIRONMENT`

### Development

Unlike `docker compose up --build -d`, running `kubectl apply -k dev` will not
build the containers in case they changed. You can run `sh ./dev.sh` that is a
utility command to turn the cluster off, build all microservices using Docker,
and turn and cluster back on.

### Production

IMPORTANT: search this folder for TODO and follow the instructions, most of them
are related to things that need to be done before deploying to production.
