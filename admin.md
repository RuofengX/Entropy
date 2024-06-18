### Pull the image
```bash
$ podman pull registry.access.redhat.com/rhel8/postgresql-16 
```

### Run database daemon
```bash
$ podman run -p 5432:5432 -e POSTGRESQL_ADMIN_PASSWORD=123456 -d postgresql-16:latest 
```
