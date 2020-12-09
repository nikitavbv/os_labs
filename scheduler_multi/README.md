# Dynamic Process Scheduling with realtime constraints

This is an implementation of hungarian algorithm for realtime scheduling.

For this algorithm, we are give with a list of tasks and a list of nodes those tasks to be scheduled on (number of 
tasks is the same as the number of nodes).

Other input information for tasks:
- `start_time` - point in time when task is scheduled.
- `end_time` - point in time for task to be completed (please note that it is not actual time required for task to 
finish, instead it is a deadline. Also, it should be after start time)
- `computing_time` - how much time it takes for task to complete on the fastest node.
- `scheduling_time` - how much time it takes for scheduler to schedule a task.
- `compute_nodes` - node performance factor (0 < F <= 1, 1 is the fastest node.)

Firstly, we build a matrix using input information for realtime tasks: 

`C_ij = end_time_i - start_time_i - scheduling_time - computing_time_i / compute_nodes_i_j`

The resulting matrix should look like this:
```
 3  2  2  1  1  2 
 1  0  0  0  0  0 
 5  4  4  4  3  4 
-4 -5 -6 -8 -10 -5 
 1  0  0  0 -1  0 
 3  2  2  2  2  2 
```

After that, change all positive numbers to 0, and invert all negative.

See `make_matrix` function for building task-node matrix using input information and algorithm described above.

In `find_solutions` we subtract min values from each column/row. As the result, we should get at least one zero in each row/column.

After that, we try to find pairs inside bipartite graph. We can do that with a simple brute force algorithm. It is OK
to skip some nodes at first.

If any task or node is not included in resulting pairs list, we subtract min values (on the intersection of rows and 
columns corresponding to nodes and tasks which are not included) and start a new algorithm iteration.

When all tasks are matched to nodes, then the solution is found.

Output example:
```
{0: 3, 1: 1, 5: 4, 2: 2, 4: 5, 3: 0}
```

Task 0 is scheduled to node 3, task 1 - to node 1, etc.