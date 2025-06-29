# Haxor

A small tool to help deploy your shitty hackathon projects to your own shitty kubernetes clutser

The goal:

You write a small script like the following:

```ts
import { database, deployment } from "@haxor/core";

const db = database("database")
  .provider("postgres")
  .addUser("postgres", "builtin:admin");

const backend = depyloment("backend")
  .buildDocker({
    dockerfile: "./Backend.Dockerfile",
    context: "./backend",
  })
  .env({
    DB_URL: db.connectAs("postgres"),
  })
  .expose({
    8080: {
      host: "api.todo.laspruca.nz",
      tls: true,
    },
  });

const frontend = deployment("fontend")
  .buildDocker({
    dockerfile: "./Backend.Dockerfile",
    context: "./backend",
  })
  .env({
    VITE_API_URL: backend.expose[8080].host,
  })
  .expose({
    8000: {
      host: "todo.laspruca.nz",
      tls: true,
    },
  });

backend.env.add("CORS_URL", frontend.expose[8000].host);
```

Then run `haxor deploy`, and haxor will:
- Provision your databases
- Build your applications
- Upload them to the cluster
- Create ingres routes
- Create ssl certificates

