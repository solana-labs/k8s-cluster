# Deploy cluster of validators in kubernetes
Use w/ https://github.com/solana-labs/solana/tree/master/k8s-cluster

# Setup
Ensure all monogon nodes can pull from the registry you will be pulling your validator docker images from

```
kubectl create ns <namespace>
```

Clone the repo
```
git clone git@github.com:solana-labs/k8s-cluster.git
cd k8s-cluster
```

1) Deploy a bootstrap validator with N regular validators using default genesis/stake/deployment configurations
```
cargo run --bin solana-k8s --
    -n <namespace e.g. greg-test>
    --num-validators N
    --bootstrap-image <registry>/bootstrap-<image-name>:<tag>
    --validator-image <registry>/validator-<image-name>:<tag>
```

2) Run the following for validator/genesis configurations:
```
cargo run --bin solana-k8s -- --help
```


Verify validators have deployed:
```
kubectl get pods -n <namespace>
```
^ `STATUS` should be `Running` and `READY` should be `1/1` for all

Verify validators are connected properly:
```
BOOTSTRAP_POD=$(kubectl get pods -n <namespace> | grep bootstrap | awk '{print $1}')
kubectl exec -it -n <namespace> $BOOTSTRAP_POD -- /bin/bash

solana -ul gossip # should see `--num-validators`+1 nodes (including bootstrap)
solana -ul validators # should see `--num-validators`+1 current validators (including bootstrap)
```
^ if you ran the tar deployment, you should see the Stake by Version as well read `<release-channel>` in the `solana -ul validators` output.

### Notes:
- Due to versioning errors between `kube-rs`/`k8s-openapi` and the Solana monorepo, we use the `solana-cli` for building genesis and not the actual Solana rust libraries.
- Registry needs to be remotely accessible by all monogon nodes
- Have tested deployments of up to 200 validators
- Additional validator commandline flags are coming....stay tuned
