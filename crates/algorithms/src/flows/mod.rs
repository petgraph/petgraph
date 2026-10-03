//! # Flows
//! 
//! In this module, we provide algorithms to solve various flow problems in graphs.
//! [Flow problems](flow_problems_wikipedia), are a class of problems in which the input is a flow
//! network (a graph with attributes like capacity, cost or demand on its edges), and the goal is to
//! construct a feasible flow. That is assign a value to each edge which respects the constraints
//! and possibly optimizes some objective function (like minimal cost).
//!
//! [flow_problems_wikipedia]: https://en.wikipedia.org/wiki/Network_flow_problem

pub mod maximum_flow;
