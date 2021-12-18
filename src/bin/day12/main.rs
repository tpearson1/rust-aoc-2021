use std::{
    collections::{HashMap, HashSet},
    fs,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = "src/bin/day12/input.txt";
    let input = fs::read_to_string(filename).map_err(|_| "Could not read input contents")?;

    let graph = CaveGraph::from_str(&input).ok_or("Failed to parse input as graph")?;

    let result = graph.traverse(false)?;
    println!("Path count (no second visits): {}", result);

    let result = graph.traverse(true)?;
    println!("Path count (up to one second visit): {}", result);

    Ok(())
}

type CaveId = u32;

#[derive(Debug)]
struct Cave {
    id: CaveId,
    big: bool,
}

impl Cave {
    fn from(id: CaveId, name: &str) -> Self {
        Self {
            id,
            big: Self::is_big(name),
        }
    }

    fn is_big(name: &str) -> bool {
        name.chars()
            .next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false)
    }
}

#[derive(Debug)]

pub struct CaveGraph {
    vertices: HashMap<String, CaveId>,
    adjacency_list: HashMap<CaveId, Vec<Cave>>,
}

impl CaveGraph {
    pub fn from_str(input: &str) -> Option<CaveGraph> {
        let mut vertices: HashMap<&str, CaveId> = HashMap::new();
        let mut adjacency_list: HashMap<CaveId, Vec<Cave>> = HashMap::new();

        let mut id_counter = 0;

        let edge = input
            .lines()
            .map(|line| line.split_once('-'))
            .collect::<Option<Vec<_>>>()?;
        for (start, end) in edge {
            let start_id = *vertices.entry(start).or_insert_with(|| {
                id_counter += 1;
                id_counter
            });
            let end_id = *vertices.entry(end).or_insert_with(|| {
                id_counter += 1;
                id_counter
            });

            adjacency_list
                .entry(start_id)
                .or_default()
                .push(Cave::from(end_id, end));
            adjacency_list
                .entry(end_id)
                .or_default()
                .push(Cave::from(start_id, start));
        }

        let vertices = vertices
            .iter()
            .map(|(name, &id)| (name.to_string(), id))
            .collect();

        Some(Self {
            vertices,
            adjacency_list,
        })
    }

    fn cave_id(&self, name: &str) -> Option<CaveId> {
        self.vertices.get(name).copied()
    }

    #[cfg(test)]
    fn adjacent_to(&self, id: CaveId) -> HashSet<CaveId> {
        self.adjacency_list
            .get(&id)
            .map(|list| list.iter().map(|cave| cave.id).collect())
            .unwrap_or_default()
    }

    const START_CAVE: &'static str = "start";
    const END_CAVE: &'static str = "end";

    pub fn traverse(&self, allow_second_visit: bool) -> Result<u64, &'static str> {
        let start = self
            .cave_id(Self::START_CAVE)
            .ok_or("No start cave found")?;
        let end = self.cave_id(Self::END_CAVE).ok_or("No end cave found")?;

        #[derive(Clone)]
        struct PathState {
            allow_another_visit: bool,
            visited_small_caves: HashSet<CaveId>,
        }

        let mut path_states: Vec<PathState> = Vec::new();

        let mut stack = Vec::new();

        stack.push((start, path_states.len()));
        path_states.push(PathState {
            allow_another_visit: allow_second_visit,
            visited_small_caves: HashSet::from([start]),
        });

        let mut total_path_count = 0;

        while let Some((cave_id, path_idx)) = stack.pop() {
            let adjacent = self.adjacency_list.get(&cave_id);
            if let Some(adjacent) = adjacent {
                for cave in adjacent {
                    if cave.id == end {
                        total_path_count += 1;
                        continue;
                    }

                    let state = &mut path_states[path_idx];
                    let mut allow_another_visit = state.allow_another_visit;
                    if !cave.big && state.visited_small_caves.contains(&cave.id) {
                        if state.allow_another_visit && cave.id != start {
                            allow_another_visit = false;
                        } else {
                            continue;
                        }
                    }

                    let state = &path_states[path_idx];
                    stack.push((cave.id, path_states.len()));

                    let mut new_state = state.clone();
                    if !cave.big {
                        new_state.allow_another_visit = allow_another_visit;
                        new_state.visited_small_caves.insert(cave.id);
                    }
                    path_states.push(new_state);
                }
            }
        }

        Ok(total_path_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIMPLE_INPUT: &str = "\
start-A
start-b
A-c
A-b
b-d
A-end
b-end";

    #[test]
    fn test_parse_input() {
        let graph = CaveGraph::from_str(SIMPLE_INPUT).unwrap();

        let start = graph.cave_id("start").unwrap();
        let a = graph.cave_id("A").unwrap();
        let b = graph.cave_id("b").unwrap();
        let c = graph.cave_id("c").unwrap();
        let d = graph.cave_id("d").unwrap();
        let end = graph.cave_id("end").unwrap();
        assert_eq!(graph.vertices.len(), 6);

        assert_eq!(graph.adjacent_to(start), HashSet::from([a, b]));
        assert_eq!(graph.adjacent_to(a), HashSet::from([start, b, c, end]));
        assert_eq!(graph.adjacent_to(b), HashSet::from([start, a, d, end]));
        assert_eq!(graph.adjacent_to(c), HashSet::from([a]));
        assert_eq!(graph.adjacent_to(d), HashSet::from([b]));
        assert_eq!(graph.adjacent_to(end), HashSet::from([a, b]));
    }

    const MEDIUM_INPUT: &str = "\
dc-end
HN-start
start-kj
dc-start
dc-HN
LN-dc
HN-end
kj-sa
kj-HN
kj-dc";

    const LARGE_INPUT: &str = "\
fs-end
he-DX
fs-he
start-DX
pj-DX
end-zg
zg-sl
zg-pj
pj-he
RW-he
fs-DX
pj-RW
zg-RW
start-pj
he-WI
zg-he
pj-fs
start-RW";

    #[test]
    fn test_traverse() {
        let graph = CaveGraph::from_str(SIMPLE_INPUT).unwrap();
        let result = graph.traverse(false).unwrap();
        assert_eq!(result, 10);
        let result = graph.traverse(true).unwrap();
        assert_eq!(result, 36);

        let graph = CaveGraph::from_str(MEDIUM_INPUT).unwrap();
        let result = graph.traverse(false).unwrap();
        assert_eq!(result, 19);
        let result = graph.traverse(true).unwrap();
        assert_eq!(result, 103);

        let graph = CaveGraph::from_str(LARGE_INPUT).unwrap();
        let result = graph.traverse(false).unwrap();
        assert_eq!(result, 226);
        let result = graph.traverse(true).unwrap();
        assert_eq!(result, 3509);
    }
}
