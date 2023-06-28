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
I'm exploring the different ways that a single shard can handle its data. 
Although both the endpoints and the data live on the same process, I wish to 
be able to separate them in the near future so that the developer can focus
on create queries on the dispatcher and not the individual shards.
I have managed to add support for procedural macros which drastically decreases the 
complexity of queries, by making it possible to directly test if a document matches
certain criteria (it is still quite primitive and it's only possible to do
primitive queries). 

I'd like the system to rely on one or more dispatchers which would take
advantage of the distributed network of shards to make complex queries
that don't just rely on one data source.

The current system is very diy, but I think it could be made into an actual
database suitable for hobbyists and small to medium projects.


One of the biggest concerns right now is that since the models are embedded into the
executable it would be quite hard to make mechanisms that could propagate these
changes reliably over a distributed network while causing few disruptions. (That doesn't mean
that it's not possible).


Also this kind of architecture could also make some patterns possible, like 
reserving a shard to a certain model or to specific relationships. Overall,
I think it could bring a lot of flexibility and performance, if done right.

Let's see where this goes! Feel free to contribute, help with code quality or even
suggest ideas!

# Project structure 
- /dispatcher: Code for the dispatcher
- /lib: library code 
- /macros: Procedural and utility macros to make your life easier
- /shard: Code for an individual db shard
- /shared: Where you define models and code that is shared between the dispatcher/db

# Testing note

If you want to try it out right now:
- Clone the repo
- `$cargo run`
- Then you can try some queries out:
```
GET http://127.0.0.1:8080/get_user?country=United States
GET http://127.0.0.1:8080/canadians
```