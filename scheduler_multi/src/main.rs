use std::collections::{HashMap, HashSet};

fn main() {
    println!("Hello, scheduler!");

    let compute_nodes = vec![1.0, 0.9, 0.8, 0.7, 0.6, 0.9]; // Z_j

    let scheduling_time = 2;
    let start_time = vec![0, 0, 0, 0, 0, 0]; // T_in
    let end_time = vec![8, 4, 9, 6, 5, 6]; // realtime limit, T_out
    let computing_time = vec![3, 1, 2, 8, 2, 1]; // one is higher than limit, others are lower, T_iw

    let matrix = make_matrix(compute_nodes, scheduling_time, start_time, end_time, computing_time);
    let result = find_solutions(&matrix);

    println!("solution is: {:?}", &result);
}

fn make_matrix(compute_nodes: Vec<f64>, scheduling_time: i32, start_time: Vec<i32>, end_time: Vec<i32>, computing_time: Vec<i32>) -> Vec<Vec<i32>> {
    // first step: matrix of relative computing time
    let mut relative_computing_time = vec![];
    for time in &computing_time {
        let mut compute_times_for_task = vec![];

        for node in &compute_nodes {
            compute_times_for_task.push((*time as f32 / *node as f32).ceil() as i32);
        }

        relative_computing_time.push(compute_times_for_task);
    }

    println!("relative computing time is");
    print_matrix(&relative_computing_time);

    // second step: time budget matrix
    let mut time_budget_matrix = vec![];
    for task_index in 0..relative_computing_time.len() {
        let mut time_budget_row = vec![];

        for node_index in 0..relative_computing_time[task_index].len() {
            let time_budget = end_time[task_index]
                - start_time[task_index]
                - scheduling_time
                - relative_computing_time[task_index][node_index];
            time_budget_row.push(time_budget);
        }

        time_budget_matrix.push(time_budget_row);
    }

    println!("time budget matrix is");
    print_matrix(&time_budget_matrix);

    // realtime requirement
    let time_budget_sum: i32 = time_budget_matrix.iter().map(|v| v.iter().sum::<i32>()).sum();
    let realtime_requirement = time_budget_sum >= 0;
    println!("time budget sum = {}, realtime requirement = {}", time_budget_sum, realtime_requirement);

    // map time budget
    let mut time_budget_mapped = vec![];
    for row in &time_budget_matrix {
        let mut row_mapped = vec![];

        for item in row {
            row_mapped.push(if *item > 0 {
                0
            } else {
                -item
            });
        }

        time_budget_mapped.push(row_mapped);
    }

    println!("mapped time budget is");
    print_matrix(&time_budget_mapped);

    time_budget_mapped
}

fn find_solutions(matrix: &Vec<Vec<i32>>) -> HashMap<usize, usize> {
    // subtract mins
    let mut matrix = matrix.clone();
    for row in 0..matrix.len() {
        let row_min = **(&matrix[row].iter().min().unwrap_or(&0));

        for col in 0..matrix[row].len() {
            matrix[row][col] -= row_min;
        }
    }

    for col in 0..matrix[0].len() {
        let mut col_min = matrix[0][col];
        for n in 1..matrix.len() {
            if matrix[n][col] < col_min {
                col_min = matrix[n][col];
            }
        }

        for n in 1..matrix.len() {
            matrix[n][col] -= col_min;
        }
    }

    println!("after sub");
    print_matrix(&matrix);

    // pairs
    let pairs = pairs_for_matrix(&matrix);
    println!("pairs is {:?}", pairs);

    let missing_tasks: HashSet<usize> = (0..matrix[0].len()).filter(|v| !pairs.contains_key(v)).collect();
    let missing_nodes: HashSet<usize> = (0..matrix.len()).filter(|v| !pairs.values().collect::<HashSet<&usize>>().contains(v))
        .collect();

    println!("missing tasks: {:?}", missing_tasks);
    println!("missing nodes: {:?}", missing_nodes);

    if missing_tasks.len() == 0 && missing_nodes.len() == 0 {
        return pairs;
    }

    let mut min_v = i32::MAX;
    for y in 0..matrix.len() {
        for x in 0..matrix[y].len() {
            if missing_tasks.contains(&y) && !missing_nodes.contains(&x) {
                if matrix[x][y] < min_v {
                    min_v = matrix[x][y];
                }
            }
        }
    }

    for y in 0..matrix.len() {
        for x in 0..matrix[y].len() {
            if missing_nodes.contains(&y) {
                matrix[y][x] += min_v;
            }

            if missing_tasks.contains(&x) {
                matrix[y][x] -= min_v;
            }
        }
    }

    return find_solutions(&matrix);
}

fn pairs_for_matrix(matrix: &Vec<Vec<i32>>) -> HashMap<usize, usize> {
    let mut pairs: HashMap<usize, usize> = HashMap::new();

    let is_link = |from: usize, to: usize| -> bool {
        matrix[from][to] == 0
    };

    let already_connected_to_node = |pairs: &HashMap<usize, usize>, to: usize| -> Option<usize> {
        let v: Vec<usize> = pairs.iter()
            .filter_map(|(k, &v)| if v == to { Some(*k) } else { None })
            .collect();
        v.get(0).map(|v| *v)
    };

    let mut possible_connections: HashMap<usize, Vec<usize>> = HashMap::new(); // <Left, Vec<Right>>
    let mut possible_connections_reverse: HashMap<usize, Vec<usize>> = HashMap::new(); // <Right, Vec<Left>>

    for job_row_index in 0..matrix.len() {
        let node_indexes: Vec<usize> = (0..matrix[job_row_index].len()).collect();

        possible_connections.insert(
            job_row_index,
            node_indexes.iter()
                .filter(|v| is_link(job_row_index, **v as usize))
                .map(|v| *v as usize)
                .collect()
        );

        possible_connections_reverse.insert(
            job_row_index,
            node_indexes.iter()
                .filter(|v| is_link(**v as usize, job_row_index))
                .map(|v| *v as usize)
                .collect()
        );
    }

    for job_row_index in 0..matrix.len() {
        let connections_from = &possible_connections[&job_row_index];

        let connections_without_used: Vec<usize> = connections_from.iter()
            .filter(|v| already_connected_to_node(&pairs, **v).is_none())
            .map(|v| *v)
            .collect();

        if connections_without_used.len() > 0 {
            pairs.insert(job_row_index, connections_without_used[0]);
        } else if connections_from.len() > 0 {
            let node0 = possible_connections_reverse[&connections_from[0]][0];
            let connections_from_node0 = &possible_connections[&node0];
            let connections_from_node0_without_used: Vec<usize> = connections_from_node0.iter()
                .filter(|v| already_connected_to_node(&pairs, **v).is_none() && **v > pairs[&node0])
                .map(|v| *v)
                .collect();

            if connections_from_node0_without_used.len() > 0 {
                pairs.insert(job_row_index, pairs[&node0]);

                pairs.remove(&node0);
                pairs.insert(node0, connections_from_node0_without_used[0]);
            }
        }
    }

    pairs
}

fn print_matrix(matrix: &Vec<Vec<i32>>) {
    for row in matrix {
        for item in row {
            print!("{:2} ", item);
        }
        println!();
    }
}