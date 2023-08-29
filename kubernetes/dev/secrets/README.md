Each of the files in .gitignore is a secret that needs to be created in this folder.

Creating the secrets manually here allow them not to be deployed to the code repo.

The script below can be used to generate the secrets for your local cluster:

```sh
openssl genrsa -out jwt.key 3072
openssl rsa -in jwt.key -pubout -out jwt.pub
echo -n $(openssl rand -base64 36) > grafana.pwd
```
