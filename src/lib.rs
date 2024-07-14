//! Functions for vertical label placement that minimise the maximum absolute offset of any label
//! from its preferred position, while respecting limits on how high or low labels may be placed.
//! This crate serves as a reference implementation of the algorithm described in Kate Morley’s
//! article [Vertical label placement](https://iamkate.com/code/vertical-label-placement/).
//!
//! # Examples
//!
//! Placing labels, respecting a minimum separation:
//!
//! ```rust
//! # fn main() {
//! let preferred_positions = vec![-10, -1, 1, 10];
//!
//! let permitted_positions = vertical_label_placement::place(&preferred_positions, 10);
//!
//! assert_eq!([-15, -5, 5, 15], *permitted_positions);
//! # }
//! ```
//!
//! Placing labels, respecting a minimum separation and minimum and maximum positions:
//!
//! ```rust
//! # fn main() {
//! let preferred_positions = vec![-10, -1, 1, 10];
//!
//! let permitted_positions = vertical_label_placement::place_with_limits(
//!     &preferred_positions,
//!     10,
//!     0,
//!     100
//! );
//!
//! assert_eq!([0, 10, 20, 30], *permitted_positions);
//! # }
//! ```

use std::cmp::{max, min};

/// Places labels, respecting a minimum separation.
///
/// # Examples
///
/// ```rust
/// # fn main() {
/// let preferred_positions = vec![-10, -1, 1, 10];
///
/// let permitted_positions = vertical_label_placement::place(&preferred_positions, 10);
///
/// assert_eq!([-15, -5, 5, 15], *permitted_positions);
/// # }
/// ```
pub fn place(positions: &[i32], separation: i32) -> Vec<i32> {
    let mut clusters = ClusterList::new(separation, positions.len());

    for position in positions {
        let mut cluster = Cluster::new(*position);

        while let Some(previous) = clusters.pop_if_not_separate(cluster) {
            cluster = Cluster::merge(previous, cluster, separation);
        }

        clusters.push(cluster);
    }

    clusters.positions()
}

/// Places labels, respecting a minimum separation and minimum and maximum positions.
///
/// # Examples
///
/// ```rust
/// # fn main() {
/// let preferred_positions = vec![-10, -1, 1, 10];
///
/// let permitted_positive_positions = vertical_label_placement::place_with_limits(
///     &preferred_positions,
///     10,
///     0,
///     100
/// );
///
/// let permitted_negative_positions = vertical_label_placement::place_with_limits(
///     &preferred_positions,
///     10,
///     -100,
///     0
/// );
///
/// assert_eq!([0, 10, 20, 30], *permitted_positive_positions);
/// assert_eq!([-30, -20, -10, 0], *permitted_negative_positions);
/// # }
/// ```
///
/// Note that if the limits do not provide sufficient space for all the labels, only the maximum
/// limit will be respected:
///
/// ```rust
/// # fn main() {
/// let preferred_positions = vec![-10, -1, 1, 10];
///
/// let permitted_positions = vertical_label_placement::place_with_limits(
///     &preferred_positions,
///     10,
///     -10,
///     10
/// );
///
/// assert_eq!([-20, -10, 0, 10], *permitted_positions);
/// # }
/// ```
pub fn place_with_limits(positions: &[i32], separation: i32, min: i32, max: i32) -> Vec<i32> {
    let mut clusters = ClusterList::new(separation, positions.len());

    for position in positions {
        let mut cluster = Cluster::new(*position).limit(min, max);

        while let Some(previous) = clusters.pop_if_not_separate(cluster) {
            cluster = Cluster::merge(previous, cluster, separation).limit(min, max);
        }

        clusters.push(cluster);
    }

    clusters.positions()
}

/// Represents a set of neighbouring labels whose permitted positions are separated by exactly the
/// minimum separation.
#[derive(Copy, Clone)]
struct Cluster {
    /// The start position.
    start: i32,
    /// The end position.
    end: i32,
    /// The minimum offset.
    min_offset: i32,
    /// The maximum offset.
    max_offset: i32,
}

impl Cluster {
    /// Creates a new cluster containing a single position.
    fn new(position: i32) -> Self {
        Self {
            start: position,
            end: position,
            min_offset: 0,
            max_offset: 0,
        }
    }

    /// Creates a new cluster by merging two neighbouring clusters.
    fn merge(mut first: Self, second: Self, separation: i32) -> Self {
        first.shift(second.start - first.end - separation);

        Self {
            start: first.start,
            end: second.end,
            min_offset: min(first.min_offset, second.min_offset),
            max_offset: max(first.max_offset, second.max_offset),
        }
        .balance()
    }

    /// Moves the cluster by an offset.
    fn shift(&mut self, offset: i32) {
        self.start += offset;
        self.end += offset;
        self.min_offset += offset;
        self.max_offset += offset;
    }

    /// Shifts the cluster to minimise the sum of `min_offset` and `max_offset`.
    ///
    /// This is equivalent to minimising the maximum absolute offset within the cluster.
    fn balance(mut self) -> Self {
        let imbalance = (self.min_offset + self.max_offset) / 2;

        if imbalance != 0 {
            self.shift(-imbalance);
        }

        self
    }

    /// Shifts the cluster to respect the limits.
    fn limit(mut self, min: i32, max: i32) -> Self {
        if self.start < min {
            self.shift(min - self.start);
        }

        if self.end > max {
            self.shift(max - self.end);
        }

        self
    }
}

/// Represents a list of clusters, providing stack-like access.
struct ClusterList {
    /// The vector of clusters.
    vec: Vec<Cluster>,
    /// The minimum separation.
    separation: i32,
    /// The requested capacity.
    capacity: usize,
}

impl ClusterList {
    /// Creates a new list of clusters.
    ///
    /// Providing a capacity equal to the number of labels prevents reallocation of vectors in
    /// `push()` and `positions()`.
    fn new(separation: i32, capacity: usize) -> Self {
        Self {
            vec: Vec::with_capacity(capacity),
            separation,
            capacity,
        }
    }

    /// Pops and returns the last cluster from the list if it is not sufficiently separated from the
    /// specified cluster, and otherwise returns `None`.
    fn pop_if_not_separate(&mut self, cluster: Cluster) -> Option<Cluster> {
        if let Some(previous) = self.vec.last() {
            if previous.end + self.separation > cluster.start {
                return self.vec.pop();
            }
        }

        None
    }

    /// Pushes a cluster onto the end of the list.
    fn push(&mut self, cluster: Cluster) {
        self.vec.push(cluster);
    }

    /// Transforms the list into a vector of permitted positions.
    fn positions(self) -> Vec<i32> {
        let mut positions = Vec::with_capacity(self.capacity);

        for cluster in self.vec {
            let mut position = cluster.start;
            while position <= cluster.end {
                positions.push(position);
                position += self.separation;
            }
        }

        positions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn already_separated() {
        assert_eq!([0], *place(&[0], 10));
        assert_eq!([0, 10], *place(&[0, 10], 10));
        assert_eq!([-10, 0, 10], *place(&[-10, 0, 10], 10));
    }

    #[test]
    fn already_separated_but_outside_limits() {
        assert_eq!([-10], *place_with_limits(&[-20], 10, -10, 10));
        assert_eq!([10], *place_with_limits(&[20], 10, -10, 10));
        assert_eq!([-10, 10], *place_with_limits(&[-20, 20], 10, -10, 10));
    }

    #[test]
    fn overflowing_limits() {
        assert_eq!([-20, -10, 0], *place_with_limits(&[0, 0, 0], 10, 0, 0));
    }

    #[test]
    fn one_cluster() {
        assert_eq!([-5, 5], *place(&[0, 0], 10));
        assert_eq!([-10, 0, 10], *place(&[0, 0, 0], 10));
        assert_eq!([-20, -10, 0], *place_with_limits(&[0, 0, 0], 10, -20, 0));
        assert_eq!([0, 10, 20], *place_with_limits(&[0, 0, 0], 10, 0, 20));
    }

    #[test]
    fn two_clusters() {
        assert_eq!(
            [-30, -20, -10, 10, 20, 30],
            *place(&[-20, -20, -20, 20, 20, 20], 10)
        );
    }

    #[test]
    fn cascading_merge() {
        assert_eq!([-5, 5, 15, 25, 35], *place(&[0, 10, 20, 30, 31], 10));
    }

    #[test]
    fn odd_separation() {
        assert_eq!([-3, 2], *place(&[0, 0], 5));
        assert_eq!([-5, 0, 5], *place(&[0, 0, 0], 5));
        assert_eq!([-8, -3, 2, 7], *place(&[0, 0, 0, 0], 5));
    }
}
