use crate::biomes::rtree::param::{
    ParameterSpace, ParameterValues, parameter_distance, parameter_span,
};
use std::array;

#[derive(PartialEq, Clone)]
pub struct Leaf<T: PartialEq> {
    parameters: ParameterSpace,
    value: T,
}

#[derive(Clone)]
pub enum Node<T: PartialEq> {
    Leaf(Leaf<T>),
    SubTree {
        parameters: ParameterSpace,
        children: Vec<Node<T>>,
    },
}

impl<T: PartialEq> PartialEq<Leaf<T>> for Node<T> {
    fn eq(&self, other: &Leaf<T>) -> bool {
        let Node::Leaf(leaf) = self else {
            return false;
        };

        leaf == other
    }
}

impl<T: PartialEq> Leaf<T> {
    #[inline]
    pub fn inner(&self) -> &T {
        &self.value
    }

    #[inline]
    fn distance(&self, target: ParameterValues) -> i64 {
        self.parameters
            .iter()
            .zip(target.iter())
            .map(|(a, b)| parameter_distance(a, &(*b..*b)))
            .sum()
    }
}

impl<T: PartialEq> Node<T> {
    pub fn leaf(parameters: ParameterSpace, value: T) -> Node<T> {
        Node::Leaf(Leaf { parameters, value })
    }

    pub fn subtree(children: Vec<Node<T>>) -> Node<T> {
        #[expect(clippy::reversed_empty_ranges)]
        let mut bounds: ParameterSpace = array::from_fn(|_| i64::MAX..i64::MIN);

        for child in &children {
            bounds
                .iter_mut()
                .zip(child.parameters().iter())
                .for_each(|(bound, parameter)| {
                    *bound = parameter_span(bound, parameter);
                });
        }

        Node::SubTree {
            parameters: bounds,
            children,
        }
    }

    pub fn search<'a>(
        &'a self,
        target: ParameterValues,
        candidate: Option<&'a Leaf<T>>,
    ) -> &'a Leaf<T> {
        match self {
            Self::SubTree { children, .. } => {
                let mut min_distance = candidate
                    .as_ref()
                    .map(|leaf| leaf.distance(target))
                    .unwrap_or(i64::MAX);

                let mut closest_leaf = candidate;
                for child in children {
                    let child_distance = child.distance(target);
                    if min_distance > child_distance {
                        let leaf = child.search(target, closest_leaf);
                        let leaf_distance = if child == leaf {
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
            }
            Self::Leaf(leaf) => leaf,
        }
    }

    #[inline]
    pub fn parameters(&self) -> &ParameterSpace {
        match self {
            Self::Leaf(Leaf { parameters, .. }) => parameters,
            Self::SubTree { parameters, .. } => parameters,
        }
    }

    #[inline]
    fn distance(&self, target: ParameterValues) -> i64 {
        match self {
            Self::Leaf(leaf) => leaf.distance(target),
            Self::SubTree { parameters, .. } => parameters
                .iter()
                .zip(target.iter())
                .map(|(a, b)| parameter_distance(a, &(*b..*b)))
                .sum(),
        }
    }
}
