## Kubernetes

This structure uses Kustomize to organize multiple Kubernetes environments.

The `base` folder contains what should be common to all environments.
Other folders are environments.

Each environment should have a folder that references the `base` config, and then
a set of overlays and secrets that are specific to the environment.

You need to set your secrets before launching the environment.
Check ./dev/secrets/README.md for more info.

To run a specific environment, execute:

`kubectl apply -k ENVIRONMENT`

### Development

Unlike `docker compose up --build -d`, running `kubectl apply -k dev` will not
build the containers in case they changed. You can run `sh ./dev.sh` that is a
utility command to turn the cluster off, build all microservices using Docker,
and turn and cluster back on.

### TODO

- [] Prometheus working with Grafana
- [] Persistent volumes for Kafka, Loki and Prometheus
- [] Ingress with TLS protection
- [] Cert-manager to auto-create and renew TLS certificates
- [] ArgoCD to watch git repo and create new images on commit to main branch
