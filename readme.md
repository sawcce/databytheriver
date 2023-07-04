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
I have made macros that create DataShard (The struct that hold a db shard)/Model structs very easily.
I'm trying to make a shard work well and make it provide everything that you'd need to 
manipulate data.
Dynamic lib loading for the data shard works and so model manipulation happens in
/shared!

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
- How do you ensure that dynamic libraries aren't being tampered with
  - Malicious code could be put in there by a library
  - The way it currently works, the dynamic library has complete access over the configuration of the actix server. This isn't a problem in itself when self-hosting but could be an issue in "serverless" environment or if the file has been injected with malicious code.
- Who is the source of truth (Shard/Dispatcher?) -> how can you make sure something hasn't been modified to an undesired state
  - A data shard could be replaced by a malicious program (if multiple shards run, how do you notify an admin that there has been a problem with a query?)


# Testing note

If you want to try it out right now:
- Clone the repo
- Compile the shared library
```
cd shared
cargo build
```
- Copy the path to the dynamic library's file (ex: shared/target/debug/shared.dll)
- Launch one or both shards:
```
cd shard
cargo run -- <path to csv file> test-a0c 8080 <path to dynamic libary dll/so file> 
```
- Set the environment variable `INSTANCES` (on the dispatcher machine) to contain a semicolon list of all the shard addresses (`ex: 127.0.0.1:8080;127.0.0.1:8081`)
- Launch the dispatcher:
```
cd dispatcher
cargo run
```

## Doesn't work (data loading not implemented yet)
Data shards don't automatically load the data currently (read next section)

Then you can try some queries:

```
GET http://<address of the dispatcher>:8000/get_user?country=United States
```

## Try this instead
This will instead query a single data shard (you can change the port if you didn't directly copy the instructions)

```
GET http://127.0.0.1:8080/insert_user?id=321&first_name=Mary&last_name=Smith&country=Australia&address=78 Leaf St.&city=Sydney
GET http://127.0.0.1:8080/get_user
```