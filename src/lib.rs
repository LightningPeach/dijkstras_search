use std::{
    collections::BTreeMap,
    iter::Iterator,
    ops::Add,
    rc::{Rc, Weak},
};

pub trait Edge {
    type Cost: Default + Ord + Add<Self::Cost, Output = Self::Cost> + Clone;

    fn cost(&self) -> Self::Cost;
}

pub struct ShortestPath<Node> {
    prev_map: BTreeMap<Rc<Node>, Rc<Node>>,
    sequence: Vec<Rc<Node>>,
}

impl<Node> ShortestPath<Node> where Node: Eq + Ord {
    pub fn new(prev_map: BTreeMap<Rc<Node>, Rc<Node>>, start: Rc<Node>, goal: Rc<Node>) -> Self {
        let mut sequence = Vec::new();
        sequence.push(goal.clone());

        let mut this = goal;
        loop {
            if this == start.clone() {
                break
            }

            match prev_map.get(&this) {
                Some(prev) => {
                    sequence.push(prev.clone());
                    this = prev.clone();
                },
                None => break,
            }
        }

        ShortestPath {
            prev_map: prev_map,
            sequence: sequence,
        }
    }

    pub fn prev(&self, node: Rc<Node>) -> Option<Rc<Node>> {
        self.prev_map.get(&node).map(Clone::clone)
    }

    pub fn path(self) -> Vec<Rc<Node>> {
        self.sequence
    }
}

pub trait Graph
where
    Self: Sized,
{
    type Node: Ord + std::fmt::Debug;
    type Edge: Edge + std::fmt::Debug;

    fn neighbors(&self, node: &Self::Node) -> Vec<(Weak<Self::Node>, Self::Edge)>;

    fn path(&self, start: Rc<Self::Node>) -> BTreeMap<Rc<Self::Node>, Rc<Self::Node>> {
        let mut distance: BTreeMap<Rc<Self::Node>, <Self::Edge as Edge>::Cost> = BTreeMap::new();
        let mut prev = BTreeMap::new();
        distance.insert(start, Default::default());

        let mut visited = BTreeMap::new();
        loop {
            let maybe = distance
                .iter()
                .filter(|&(n, _)| visited.get(n).is_none())
                .min_by(|&(_, left), &(_, right)| left.cmp(right))
                .map(|(n, cost)| (n.clone(), cost.clone()));
            let (min, min_cost) = match maybe {
                Some(m) => m,
                None => break,
            };

            visited.insert(min.clone(), ());

            for (this, edge) in self.neighbors(&min) {
                if let Some(this) = this.upgrade() {
                    let alt = min_cost.clone() + edge.cost();
                    let this_distance = distance.get(&this);
                    if this_distance.is_none() || this_distance.unwrap().clone() >= alt {
                        distance.insert(this.clone(), alt);
                        prev.insert(this.clone(), min.clone());
                    }
                }
            }
        }

        prev
    }
}

#[cfg(test)]
mod test {
    use super::{ShortestPath, Edge, Graph};

    use std::rc::{Rc, Weak};

    #[derive(Default, Clone, Debug)]
    struct EdgeImpl {
        from: Weak<u8>,
        to: Weak<u8>,
        weight: u32,
    }

    impl Edge for EdgeImpl {
        type Cost = u32;

        fn cost(&self) -> Self::Cost {
            self.weight.clone()
        }
    }

    struct GraphImpl {
        nodes: Vec<Rc<u8>>,
        edges: Vec<EdgeImpl>,
    }

    impl Graph for GraphImpl {
        type Node = u8;
        type Edge = EdgeImpl;

        fn neighbors(&self, node: &Self::Node) -> Vec<(Weak<Self::Node>, Self::Edge)> {
            self.edges.iter().filter_map(|e| {
                match (*e.from.upgrade().unwrap() == *node, *e.to.upgrade().unwrap() == *node) {
                    (true, _) => Some((e.to.clone(), e.clone())),
                    (_, true) => Some((e.from.clone(), e.clone())),
                    _ => None,
                }
            }).collect()
        }
    }

    impl GraphImpl {
        pub fn new(count: u8) -> Self {
            GraphImpl {
                nodes: (0..count).map(Rc::new).collect(),
                edges: Vec::new(),
            }
        }

        pub fn insert(&mut self, from: u8, to: u8, weight: u32) {
            let from = from as usize;
            let to = to as usize;
            if from < self.nodes.len() && to < self.nodes.len() {
                self.edges.push(EdgeImpl {
                    from: Rc::downgrade(&self.nodes[from]),
                    to: Rc::downgrade(&self.nodes[to]),
                    weight,
                })
            }
        }
    }

    #[test]
    fn test_0() {
        let mut graph = GraphImpl::new(10);
        graph.insert(0, 1, 10);
        graph.insert(1, 9, 50);
        graph.insert(1, 2, 10);
        graph.insert(2, 3, 40);
        graph.insert(3, 9, 10);

        let prev = graph.path(graph.nodes[0].clone());
        let path = ShortestPath::new(prev, graph.nodes[0].clone(), graph.nodes[9].clone());

        assert_eq!(vec![Rc::new(9), Rc::new(1), Rc::new(0)], path.path());
    }

    #[test]
    fn test_1() {
        let mut graph = GraphImpl::new(10);
        graph.insert(0, 1, 10);
        graph.insert(1, 9, 50);
        graph.insert(1, 2, 10);
        graph.insert(2, 3, 10);
        graph.insert(3, 9, 10);

        let prev = graph.path(graph.nodes[0].clone());
        let path = ShortestPath::new(prev, graph.nodes[0].clone(), graph.nodes[9].clone());

        assert_eq!(vec![Rc::new(9), Rc::new(3), Rc::new(2), Rc::new(1), Rc::new(0)], path.path());
    }
}
