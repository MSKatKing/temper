use crate::biomes::BiomeParameters;
use crate::biomes::rtree::node::Node;
use crate::biomes::rtree::param::{PARAMETER_COUNT, ParameterValues};

mod node;
mod param;

pub use param::ParameterSpace;

pub struct RTree<T: PartialEq + Clone> {
    root: Node<T>,
}

impl<T: PartialEq + Clone> RTree<T> {
    pub fn new(values: Vec<(BiomeParameters, T)>) -> Self {
        debug_assert!(!values.is_empty());

        let leaves = values
            .into_iter()
            .map(|(parameters, value)| Node::leaf(parameters.as_param_list(), value))
            .collect::<Vec<_>>();

        Self {
            root: Self::build(leaves),
        }
    }

    #[inline(always)]
    pub fn search(&self, target: ParameterValues) -> &T {
        self.root.search(target, None).inner()
    }

    fn build(mut children: Vec<Node<T>>) -> Node<T> {
        match children.len() {
            0 => panic!("empty tree"),
            1 => children.pop().unwrap(),
            2..PARAMETER_COUNT => {
                children.sort_by_key(|leaf| {
                    leaf.parameters()
                        .iter()
                        .map(|param| ((param.start + param.end) / 2).abs())
                        .sum::<i64>()
                });

                Node::subtree(children)
            }
            _ => {
                let mut min_cost = i64::MAX;
                let mut min_dimension = 0;
                let mut min_buckets = None;

                for d in 0..7 {
                    Self::sort::<false>(&mut children, d);
                    let buckets = Self::bucketize(children.clone());
                    let total_cost = buckets
                        .iter()
                        .map(|bucket| Self::cost(bucket.parameters()))
                        .sum();

                    if min_cost > total_cost {
                        min_cost = total_cost;
                        min_dimension = d;
                        min_buckets = Some(buckets);
                    }
                }

                let mut min_buckets = min_buckets.unwrap();
                Self::sort::<true>(&mut min_buckets, min_dimension);
                Node::subtree(
                    min_buckets
                        .into_iter()
                        .map(|node| {
                            let Node::SubTree { children, .. } = node else {
                                // Self::bucketize only returns SubTree nodes
                                unreachable!()
                            };

                            Self::build(children)
                        })
                        .collect(),
                )
            }
        }
    }

    fn sort<const ABSOLUTE: bool>(children: &mut [Node<T>], dimension: usize) {
        let comparators: [_; PARAMETER_COUNT] = std::array::from_fn(|d| {
            move |leaf: &Node<T>| {
                let param = &leaf.parameters()[(dimension + d) % 7];
                let center = (param.start + param.end) / 2;

                if ABSOLUTE { center.abs() } else { center }
            }
        });

        children.sort_by(|a, b| {
            let mut cmp = comparators[0](a).cmp(&comparators[0](b));

            for func in &comparators[1..] {
                cmp = cmp.then(func(a).cmp(&func(b)));
            }

            cmp
        })
    }

    fn bucketize(nodes: Vec<Node<T>>) -> Vec<Node<T>> {
        let mut buckets = Vec::new();
        let mut children = Vec::new();
        let expected_children =
            6.0f64.powi(((nodes.len() as f64 - 0.01).ln() / f64::ln(6.0)) as i32) as usize;

        for child in nodes {
            children.push(child);

            if children.len() >= expected_children {
                buckets.push(Node::subtree(std::mem::take(&mut children)));
            }
        }

        if !children.is_empty() {
            buckets.push(Node::subtree(children));
        }

        buckets
    }

    #[inline(always)]
    fn cost(parameters: &ParameterSpace) -> i64 {
        parameters
            .iter()
            .map(|param| (param.end - param.start).abs())
            .sum()
    }
}
