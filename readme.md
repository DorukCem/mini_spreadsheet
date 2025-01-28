A cell with an input string is first [parsed](https://en.wikipedia.org/wiki/Parsing) by [tokenizing](https://en.wikipedia.org/wiki/Lexical_analysis) it. For example, an expression such as `= sum(1,2)` would be converted into something like: `[function_name("sum"), left_paranthesis, number(1), comma, number(2), right_paranthesis]`. These tokens represent types within the program, making it possible to reason and work with them. Next, we construct an [Abstract Syntax Tree](https://en.wikipedia.org/wiki/Abstract_syntax_tree) (AST) to compute the final value of the expression.

Once the AST of a cell is computed, it is stored in memory for future use, allowing us to skip parsing when recomputation is needed. During this process, we also identify which cells are referenced, enabling the construction of the computation graph explained below.


### Computation Order

Determining the order of computation is crucial. For example, consider a scenario where cell B1 contains an expression such as `= A1 + 2`. To compute the value of B1, the value of A1 must be available first. Therefore, the computation of A1 must precede that of B. Furthermore, if the value of A1 changes, its dependents (in this case B1) must be updated accordingly.

To manage these dependencies, we construct a [directed graph](https://en.wikipedia.org/wiki/Directed_graph) representing which nodes (cells) enable the computation of others.

Whenever a cell is added, removed, or modified, the graph must be updated to reflect the change. For instance, if cell A2 is added and references B1, this indicates that B1 is required to compute A2. In the graph, this relationship is represented by adding a directed edge from B1 to A2.  

![Example graph](images/graph.png)  

In this example, it is evident that starting the computation from cell C1 avoids any reference errors. While it may be straightforward to determine the starting point and propagation of computation in small graphs, handling larger and more complex graphs requires a systematic approach. Specifically, we need an algorithm to determine the computation order in a generic directed graph. To achieve this, we perform a [topological sort](https://en.wikipedia.org/wiki/Topological_sorting) of the graph. Once sorted, we compute all cells in the determined order.

Additionally, as shown in the example, if the value of B1 changes, we must recompute both A3 and A2 to reflect the updated value of B1.
---

### Cyclic References

In some scenarios, it may be impossible to construct an [acyclic graph](https://en.wikipedia.org/wiki/Directed_acyclic_graph).  

![Example graph 2](images/graph2.png)  

When a cycle is present, it becomes impossible to compute a final result for the cells involved in the cycle, and an error must be returned. To avoid getting stuck in an infinite loop during computation, the topological sorting algorithm must be adapted to identify cells that form a cycle. These cells will be excluded from the computation process and flagged as part of a cycle.

