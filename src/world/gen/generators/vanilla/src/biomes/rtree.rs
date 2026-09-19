use crate::biomes::BiomeParameters;
use std::ops::Range;

pub const PARAMETER_COUNT: usize = 7;

pub type ClimateParameter = Range<i64>;

pub struct RTree<T: Clone> {
    root: Node<T>,
}

#[derive(Clone)]
enum Node<T: Clone> {
    SubTree {
        parameter_space: [Range<i64>; PARAMETER_COUNT],
        children: Vec<Node<T>>,
    },
    Leaf {
        parameter_space: [Range<i64>; PARAMETER_COUNT],
        inner: T,
    }
}

impl<T: Clone> RTree<T> {
    pub fn new(values: Vec<(BiomeParameters, T)>) -> Self {
        assert!(!values.is_empty());

        let leaves = values.into_iter()
            .map(|(parameters, value)| {
                Node::wrap(parameters.as_param_list(), value)
            })
            .collect::<Vec<_>>();

        Self {
            root: Self::build(leaves),
        }
    }

    fn build(mut children: Vec<Node<T>>) -> Node<T> {
        assert!(!children.is_empty());

        match children.len() {
            0 => unreachable!(),
            1 => children.into_iter().next().unwrap(),
            2..7 => {
                children.sort_by_key(|leaf| {
                    let mut total_magnitude = 0;

                    for param in leaf.parameter_space() {
                        total_magnitude += ((param.start + param.end) / 2).abs();
                    }

                    total_magnitude
                });

                Node::new_sub_tree(children)
            },
            _ => {
                let mut min_cost = i64::MAX;
                let mut min_dimension = 0;
                let mut min_buckets = None;

                for d in 0..7 {
                    Self::sort(&mut children, d, false);
                    let buckets = Self::bucketize(children.clone());
                    let mut total_cost = 0;

                    for bucket in &buckets {
                        total_cost += Self::cost(bucket.parameter_space())
                    }

                    if min_cost > total_cost {
                        min_cost = total_cost;
                        min_dimension = d;
                        min_buckets = Some(buckets);
                    }
                }

                let mut min_buckets = min_buckets.unwrap();
                Self::sort(&mut min_buckets, min_dimension, true);
                Node::new_sub_tree(
                    min_buckets
                        .into_iter()
                        .map(|b| {
                            let Node::SubTree { children, .. } = b else {
                                panic!("expected tree")
                            };

                            Self::build(children)
                        })
                        .collect(),
                )
            }
        }
    }

    fn sort(children: &mut Vec<Node<T>>, dimension: usize, absolute: bool) {
        let comparators: [_; PARAMETER_COUNT] = std::array::from_fn(|d| {
            move |leaf: &Node<T>| {
                let param = &leaf.parameter_space()[(dimension + d) % 7];
                let center = (param.start + param.end) / 2;
                if absolute {
                    center.abs()
                } else {
                    center
                }
            }
        });

        children.sort_by(|a, b| {
            let mut cmp = comparators[0](a).cmp(&comparators[0](b));

            for func in &comparators[1..] {
                cmp = cmp.then(
                    func(a).cmp(&func(b)),
                )
            }

            cmp
        })
    }

    fn bucketize(nodes: Vec<Node<T>>) -> Vec<Node<T>> {
        let mut buckets = Vec::new();
        let mut children = Vec::new();
        let expected_children_count = 6.0f64.powi(((nodes.len() as f64 - 0.01).ln() / f64::ln(6.0)) as i32) as usize;

        for child in nodes {
            children.push(child);

            if children.len() >= expected_children_count {
                buckets.push(Node::new_sub_tree(std::mem::replace(&mut children, Vec::new())));
            }
        }

        if !children.is_empty() {
            buckets.push(Node::new_sub_tree(children));
        }

        buckets
    }

    fn cost(parameter_space: &[ClimateParameter; 7]) -> i64 {
        let mut result = 0;

        for param in parameter_space {
            result += (param.end - param.start).abs();
        }

        result
    }

    pub fn search(&self, target: [i64; 7]) -> &T {
        let Node::Leaf { inner, .. } = self.root
            .search(&target, None) else {
            unreachable!()
        };

        inner
    }
}

impl<T: Clone> Node<T> {
    pub fn wrap(parameter_space: [ClimateParameter; PARAMETER_COUNT], inner: T) -> Self {
        Self::Leaf { parameter_space, inner }
    }

    pub fn new_sub_tree(children: Vec<Node<T>>) -> Self {
        let mut bounds: [ClimateParameter; PARAMETER_COUNT] = std::array::from_fn(|_| i64::MAX..i64::MIN);

        for child in &children {
            for d in 0..PARAMETER_COUNT {
                bounds[d] = param_span(
                    &bounds[d],
                    &child.parameter_space()[d],
                );
            }
        }

        Self::SubTree { parameter_space: bounds, children }
    }

    pub fn search<'a>(&'a self, target: &[i64; PARAMETER_COUNT], candidate: Option<&'a Node<T>>) -> &'a Node<T> {
        match self {
            Self::SubTree { children, .. } => {
                let mut min_distance = match candidate.as_ref() {
                    Some(candidate) => candidate.distance(target),
                    None => i64::MAX,
                };

                let mut closest_leaf = candidate;
                for child in children {
                    let child_distance = child.distance(target);
                    if min_distance > child_distance {
                        let leaf = child.search(target, closest_leaf);
                        let leaf_distance = if (leaf as *const _) == (child as *const _) {
                            child_distance
                        } else {
                            leaf.distance(target)
                        };

                        if min_distance > leaf_distance {
                            min_distance = leaf_distance;
                            closest_leaf = Some(leaf);
                        }
                    }
                }

                closest_leaf.unwrap()
            },
            Self::Leaf { .. } => self,
        }
    }

    pub fn distance(&self, target: &[i64; PARAMETER_COUNT]) -> i64 {
        self.parameter_space()
            .iter()
            .zip(target.iter())
            .map(|(param, target)|
                param_distance(param, &(*target..*target))
            )
            .sum()
    }

    pub fn parameter_space(&self) -> &[ClimateParameter; PARAMETER_COUNT] {
        match self {
            Self::SubTree { parameter_space, .. } => parameter_space,
            Self::Leaf { parameter_space, .. } => parameter_space,
        }
    }
}

fn param_span(a: &ClimateParameter, b: &ClimateParameter) -> ClimateParameter {
    a.start.min(b.start)..a.end.max(b.end)
}

fn param_distance(a: &ClimateParameter, b: &ClimateParameter) -> i64 {
    let above = b.start - a.end;
    let below = a.start - b.end;

    if above > 0 {
        above
    } else {
        below.max(0)
    }
}
