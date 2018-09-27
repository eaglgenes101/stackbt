# StackBT

This set of crates is intended as a basic library on to of which to implement game AIs on top of. It contains just the basics: automata, state machines, behavior tree nodes, and enough wrappers and glue to stick them together how you want, along with a small bit of macro assistance. 

This project is also targeting inclusion with or alongside Amethyst and Specs. Policies, practices, and such will be exported from that project as the need arises. 

StackBT is kind of an artifact name. Once upon a time it was intended as a behavior tree built upon stackful coroutines, but discussions about the design convinced me that this was too heavyweight for game AI tasks, so it was reduced in scope and ambition. 

# License

StackBT is free and open source software distributed under the terms of both the MIT License and the Apache License 2.0. 

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.