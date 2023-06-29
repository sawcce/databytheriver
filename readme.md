# Experimental Database

This is an attempt to make a distributed database framework,
currently in the process of exploring how it can be achieved!

Goals:
- Make a horizontally scalable database (using some form of sharding)
- The queries happen on the database side __only__.
  - Queries are defined as http endpoints
  - No query language used between the backend and database
  - Additional query data may still be passed as url query parameters
- Provide tools to model data effectively (such as relationships between documents)
- Reduce type safety issues by working close to the data and providing tools to
create interfaces (exporting your schemas as typescript or jsdoc for instance)
- Providing optimization and fallback mechanisms
- Provide mechanisms to easily transition between different model versions even
if it involves dropping fields.

Current state:
I'm trying to streamline the development process. 
Which means finding a way to create a database struct based on a list of models
and create the required endpoints.  

One way to make the deployment process easier would be to deploy the shards as
dynamically linkable libraries which would then consumed by the shards and dispatcher
instances.

# Project structure 
- /dispatcher: Code for the dispatcher
- /lib: library code 
- /macros: Procedural and utility macros to make your life easier (directly included in /lib)
- /shard: Code for an individual db shard
- /shared: Where you define models and code that is shared between the dispatcher/db

# Current concerns
- How to handle authentication/authorization
- Whether or not the developer should have direct access to the way the db acts or if
you only have control over the dispatcher? 
  - Then how do you handle auth?
  - How to provide a nice api if the db interface isn't always the same
- Deployment pipeline

# Testing note

If you want to try it out right now:
- Clone the repo
- Launch both shards:
```
cd shard
cargo run -- .\test-a0c.csv test-a0c 8080
cargo run -- .\test-b1d.csv test-b1d 8081
```

Launch the dispatcher:

```
cd dispatcher
cargo run
```

Then you can try some queries:

```
GET http://127.0.0.1:8080/get_user?country=United States
```